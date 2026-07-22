//! Persistence of the last three months of events AND the settings table
//! (selected beaches/cities…) across dyno restarts.
//!
//! Heroku dynos have an ephemeral filesystem, so the SQLite file is wiped on
//! every restart. To survive that without an external database, the backup
//! payload is written (gzip + base64 JSON) to the `CALENDAR_BACKUP` config
//! var via the Heroku Platform API when the process shuts down (Heroku sends
//! SIGTERM before replacing a dyno), and read back from the environment when
//! an empty database boots.

use std::io::{Read, Write};

use base64::Engine;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set};
use serde::{Deserialize, Serialize};

use crate::entities::event::{self, Entity as Event};
use crate::entities::setting;
use crate::state::three_months_ago;

const BACKUP_VAR: &str = "CALENDAR_BACKUP";
/// Heroku limits the combined size of config vars (32 KB order of magnitude).
const PAYLOAD_WARN_BYTES: usize = 30_000;

#[derive(Serialize, Deserialize)]
pub struct Backup {
    pub events: Vec<event::Model>,
    /// Settings (selected tide spots, weather cities…) — absent in payloads
    /// written before this field existed.
    #[serde(default)]
    pub settings: Vec<setting::Model>,
}

pub fn encode(backup: &Backup) -> Result<String, String> {
    let json = serde_json::to_vec(backup).map_err(|e| e.to_string())?;
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(&json).map_err(|e| e.to_string())?;
    let gz = enc.finish().map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(gz))
}

pub fn decode(payload: &str) -> Result<Backup, String> {
    let gz = base64::engine::general_purpose::STANDARD
        .decode(payload.trim())
        .map_err(|e| e.to_string())?;
    let mut json = Vec::new();
    flate2::read::GzDecoder::new(&gz[..])
        .read_to_end(&mut json)
        .map_err(|e| e.to_string())?;
    // Current format first, then the legacy bare event list
    if let Ok(backup) = serde_json::from_slice::<Backup>(&json) {
        return Ok(backup);
    }
    serde_json::from_slice::<Vec<event::Model>>(&json)
        .map(|events| Backup { events, settings: Vec::new() })
        .map_err(|e| e.to_string())
}

/// Seed an empty database from the `CALENDAR_BACKUP` env var (set by the
/// previous dyno at shutdown). Only events from the last three months are
/// restored. A non-empty database is left untouched.
pub async fn restore_from_env(db: &DatabaseConnection) {
    let Ok(payload) = std::env::var(BACKUP_VAR) else {
        log::info!("no {BACKUP_VAR} set, starting fresh");
        return;
    };
    if payload.trim().is_empty() {
        return;
    }
    match Event::find().count(db).await {
        Ok(0) => {}
        Ok(n) => {
            log::info!("database already has {n} events, skipping restore");
            return;
        }
        Err(e) => {
            log::warn!("could not count events, skipping restore: {e}");
            return;
        }
    }
    let backup = match decode(&payload) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("could not decode {BACKUP_VAR}, skipping restore: {e}");
            return;
        }
    };
    // Settings first: the seed that runs right after restore needs the
    // selected beaches/cities to fetch tides and school vacations.
    let mut settings_restored = 0usize;
    for s in &backup.settings {
        match crate::settings::set(db, &s.key, &s.value).await {
            Ok(()) => settings_restored += 1,
            Err(e) => log::warn!("failed to restore setting {}: {e}", s.key),
        }
    }
    let cutoff = three_months_ago();
    let mut restored = 0usize;
    for ev in backup.events {
        // Recurring events are kept whatever their age: their occurrences
        // extend into the present.
        if ev.end < cutoff && ev.recurrence.is_none() {
            continue;
        }
        let active = event::ActiveModel {
            title: Set(ev.title),
            description: Set(ev.description),
            start: Set(ev.start),
            end: Set(ev.end),
            all_day: Set(ev.all_day),
            color: Set(ev.color),
            recurrence: Set(ev.recurrence),
            ..Default::default()
        };
        match active.insert(db).await {
            Ok(_) => restored += 1,
            Err(e) => log::warn!("failed to restore an event: {e}"),
        }
    }
    log::info!(
        "restored {restored} events and {settings_restored} settings from {BACKUP_VAR}"
    );
}

/// Write the snapshot to the app's `CALENDAR_BACKUP` config var through the
/// Heroku Platform API. Requires HEROKU_API_KEY and HEROKU_APP_NAME. Note:
/// changing a config var restarts the dyno, which is why this only runs at
/// shutdown, when the dyno is going away anyway.
pub async fn push_to_heroku(db: &DatabaseConnection, events: &[event::Model]) {
    let (Ok(key), Ok(app)) = (
        std::env::var("HEROKU_API_KEY"),
        std::env::var("HEROKU_APP_NAME"),
    ) else {
        log::info!("HEROKU_API_KEY/HEROKU_APP_NAME not set, skipping backup");
        return;
    };
    let api =
        std::env::var("HEROKU_API_URL").unwrap_or_else(|_| "https://api.heroku.com".into());
    let settings = setting::Entity::find().all(db).await.unwrap_or_default();
    let backup = Backup {
        events: events.to_vec(),
        settings,
    };
    let payload = match encode(&backup) {
        Ok(p) => p,
        Err(e) => {
            log::error!("could not encode backup: {e}");
            return;
        }
    };
    if payload.len() > PAYLOAD_WARN_BYTES {
        log::warn!(
            "backup payload is {} bytes, Heroku may reject it (config var size limit)",
            payload.len()
        );
    }
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("could not build HTTP client: {e}");
            return;
        }
    };
    let res = client
        .patch(format!("{api}/apps/{app}/config-vars"))
        .header("Accept", "application/vnd.heroku+json; version=3")
        .bearer_auth(key)
        .json(&serde_json::json!({ BACKUP_VAR: payload }))
        .send()
        .await;
    match res {
        Ok(r) if r.status().is_success() => {
            log::info!(
                "backed up {} events and {} settings to Heroku config var",
                backup.events.len(),
                backup.settings.len()
            )
        }
        Ok(r) => log::error!(
            "Heroku API returned {}: {}",
            r.status(),
            r.text().await.unwrap_or_default()
        ),
        Err(e) => log::error!("backup request failed: {e}"),
    }
}
