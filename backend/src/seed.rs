//! Themed events (cinema releases, fireworks, astrology, F1) checked and
//! (re)generated at every startup for the current and next civil year:
//!
//! - fireworks and zodiac seasons: fixed French dates, generated per year
//! - moon phases: computed (Meeus) per year — see `astro`
//! - F1: fetched from the Jolpica API per year, static 2026 list as fallback
//! - cinema: curated static list, plus TMDB when TMDB_API_KEY is set
//!
//! Events older than the three-month retention window are not (re)inserted.
//! Set SEED_DISABLED=1 to turn all of this off.

use std::collections::HashSet;

use chrono::Datelike;
use chrono_tz::Europe::Paris;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::Deserialize;

use crate::entities::event::{self, Entity as Event};
use crate::state::three_months_ago;
use crate::{astro, f1, tmdb};

/// Static baseline: curated 2026 cinema releases + the 2026 F1 season as
/// offline fallback (fireworks/astrology entries are generated instead).
const SEED_JSON: &str = include_str!("../seed_events.json");

#[derive(Deserialize)]
pub struct SeedCandidate {
    /// "YYYY-MM-DD"
    pub date: String,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

/// Dedup key: lowercase alphanumeric title + date, so cosmetic differences
/// (accents kept, punctuation/spacing/case dropped) don't create duplicates.
fn normalize(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

pub async fn seed(db: &DatabaseConnection) {
    if std::env::var("SEED_DISABLED").is_ok_and(|v| v == "1" || v == "true") {
        log::info!("seeding disabled by SEED_DISABLED");
        return;
    }

    let mut candidates: Vec<SeedCandidate> = match serde_json::from_str(SEED_JSON) {
        Ok(s) => s,
        Err(e) => {
            log::error!("invalid seed_events.json: {e}");
            Vec::new()
        }
    };

    let year = chrono::Utc::now().with_timezone(&Paris).year();
    for y in [year, year + 1] {
        candidates.extend(astro::seasons(y));
        candidates.extend(astro::moon_phases(y));
        candidates.extend(astro::fireworks(y));
        if let Some(races) = f1::fetch(y).await {
            candidates.extend(races);
        }
    }
    candidates.extend(tmdb::fetch().await);

    // Index existing events once for dedup
    let existing = match Event::find().all(db).await {
        Ok(events) => events,
        Err(e) => {
            log::warn!("could not list events, skipping seed: {e}");
            return;
        }
    };
    let mut seen_titles: HashSet<(String, String)> = HashSet::new();
    let mut f1_dates: HashSet<String> = HashSet::new();
    for ev in &existing {
        if ev.start.len() >= 10 {
            let date = ev.start[..10].to_string();
            seen_titles.insert((normalize(&ev.title), date.clone()));
            // F1 titles changed over time (static French list vs API), so
            // F1 events also dedup by color + date.
            if ev.color.as_deref() == Some(f1::F1_COLOR) {
                f1_dates.insert(date);
            }
        }
    }

    let cutoff = three_months_ago();
    let mut inserted = 0usize;
    for c in candidates {
        // 08:00–20:00 UTC stays within the same civil day in France
        // (UTC+1/+2) whatever the season, so all-day events land on a
        // single grid cell.
        let start = format!("{}T08:00:00Z", c.date);
        let end = format!("{}T20:00:00Z", c.date);
        if end < cutoff {
            continue;
        }
        let is_f1 = c.color.as_deref() == Some(f1::F1_COLOR);
        let key = (normalize(&c.title), c.date.clone());
        if seen_titles.contains(&key) || (is_f1 && f1_dates.contains(&c.date)) {
            continue;
        }
        let active = event::ActiveModel {
            title: Set(c.title),
            description: Set(c.description),
            start: Set(start),
            end: Set(end),
            all_day: Set(true),
            color: Set(c.color),
            ..Default::default()
        };
        match active.insert(db).await {
            Ok(_) => {
                inserted += 1;
                seen_titles.insert(key);
                if is_f1 {
                    f1_dates.insert(c.date);
                }
            }
            Err(e) => log::warn!("failed to insert a seed event: {e}"),
        }
    }
    if inserted > 0 {
        log::info!("seeded {inserted} themed events");
    }
}
