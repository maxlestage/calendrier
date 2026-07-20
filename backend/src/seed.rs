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
use crate::{astro, astronomy, f1, tides, tmdb};

/// Static baseline: curated 2026 cinema releases + the 2026 F1 season as
/// offline fallback (fireworks/astrology entries are generated instead).
const SEED_JSON: &str = include_str!("../seed_events.json");

#[derive(Deserialize)]
pub struct SeedCandidate {
    /// "YYYY-MM-DD" civil day in France — also the dedup key with the title
    pub date: String,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    /// Precise UTC instant; None → all-day event (08:00Z–20:00Z)
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
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

/// Paris civil day of a stored UTC instant. A full moon at 00:57 Paris is
/// still the previous day in UTC, so a bare `start[..10]` would break the
/// dedup key against candidates dated in Paris civil days.
fn paris_day(start: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(&start.replace('Z', "+00:00")) {
        Ok(dt) => dt.with_timezone(&Paris).format("%Y-%m-%d").to_string(),
        Err(_) => start.chars().take(10).collect(),
    }
}

/// Dedup slot: all-day events key on their Paris civil day, timed events on
/// their exact instant (minute). Two same-titled timed events on one day —
/// e.g. two high tides — must not collapse into one, so day granularity is
/// wrong for them.
fn dedup_slot(start: &str, all_day: bool) -> String {
    if all_day {
        paris_day(start)
    } else {
        start.chars().take(16).collect() // YYYY-MM-DDTHH:MM
    }
}

pub async fn seed(db: &DatabaseConnection) {
    if std::env::var("SEED_DISABLED").is_ok_and(|v| v == "1" || v == "true") {
        log::info!("seeding disabled by SEED_DISABLED");
        return;
    }

    // Generated events first: when the F1 API responds, its precisely timed
    // races win over the all-day static fallback (color+date dedup below).
    let mut candidates: Vec<SeedCandidate> = Vec::new();
    let year = chrono::Utc::now().with_timezone(&Paris).year();
    for y in [year, year + 1] {
        candidates.extend(astro::seasons(y));
        candidates.extend(astro::moon_phases(y));
        candidates.extend(astro::fireworks(y));
        candidates.extend(astronomy::eclipses(y));
        candidates.extend(astronomy::solstices_equinoxes(y));
        candidates.extend(astronomy::meteor_showers(y));
        if let Some(races) = f1::fetch(y).await {
            candidates.extend(races);
        }
    }
    candidates.extend(tmdb::fetch().await);
    match serde_json::from_str::<Vec<SeedCandidate>>(SEED_JSON) {
        Ok(static_events) => candidates.extend(static_events),
        Err(e) => log::error!("invalid seed_events.json: {e}"),
    }

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
    // Latest stored tide instant per port name, to avoid re-querying the
    // (quota-limited) tide API while the horizon is still comfortable.
    let mut tide_horizon: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for ev in &existing {
        if ev.start.len() >= 10 {
            seen_titles.insert((normalize(&ev.title), dedup_slot(&ev.start, ev.all_day)));
            // F1 titles changed over time (static French list vs API), so
            // F1 events also dedup by color + Paris day.
            if ev.color.as_deref() == Some(f1::F1_COLOR) {
                f1_dates.insert(paris_day(&ev.start));
            }
            if ev.color.as_deref() == Some(tides::TIDE_COLOR) {
                // "🌊 Brest — Pleine mer" → port name before the em dash
                if let Some(port) = ev.title.split(" — ").next() {
                    let port = port.trim_start_matches("🌊").trim().to_string();
                    tide_horizon
                        .entry(port)
                        .and_modify(|cur| {
                            if ev.start > *cur {
                                *cur = ev.start.clone();
                            }
                        })
                        .or_insert_with(|| ev.start.clone());
                }
            }
        }
    }

    // Fetch tides only for ports whose stored horizon runs out within the
    // next half-window (keeps API usage low: roughly one refresh per horizon).
    let now = chrono::Utc::now();
    let refresh_before = (now + chrono::Duration::days(tides::horizon_days() / 2 + 1))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let ports_needing: Vec<&tides::Port> = tides::enabled_ports()
        .into_iter()
        .filter(|p| {
            tide_horizon
                .get(p.name)
                .map(|latest| latest < &refresh_before)
                .unwrap_or(true)
        })
        .collect();
    if !ports_needing.is_empty() {
        let tide_events = tides::fetch(&ports_needing, now.timestamp()).await;
        candidates.extend(tide_events);
    }

    let cutoff = three_months_ago();
    let mut inserted = 0usize;
    for c in candidates {
        // Timed events carry their exact UTC instant; all-day events use
        // 08:00–20:00 UTC, which stays within the same civil day in France
        // (UTC+1/+2) whatever the season, so they land on a single grid cell.
        let all_day = c.start.is_none();
        let start = c.start.unwrap_or_else(|| format!("{}T08:00:00Z", c.date));
        let end = c.end.unwrap_or_else(|| format!("{}T20:00:00Z", c.date));
        if end < cutoff {
            continue;
        }
        let is_f1 = c.color.as_deref() == Some(f1::F1_COLOR);
        let slot = dedup_slot(&start, all_day);
        let pday = paris_day(&start);
        let key = (normalize(&c.title), slot);
        // The color+date rule only guards the all-day static F1 fallback
        // against the API's timed events: several timed F1 sessions can
        // legitimately share a day (sprint + qualifying on Saturday).
        if seen_titles.contains(&key) || (is_f1 && all_day && f1_dates.contains(&pday)) {
            continue;
        }
        let active = event::ActiveModel {
            title: Set(c.title),
            description: Set(c.description),
            start: Set(start),
            end: Set(end),
            all_day: Set(all_day),
            color: Set(c.color),
            ..Default::default()
        };
        match active.insert(db).await {
            Ok(_) => {
                inserted += 1;
                seen_titles.insert(key);
                if is_f1 {
                    f1_dates.insert(pday);
                }
            }
            Err(e) => log::warn!("failed to insert a seed event: {e}"),
        }
    }
    if inserted > 0 {
        log::info!("seeded {inserted} themed events");
    }
}
