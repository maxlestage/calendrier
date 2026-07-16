//! Pre-loaded themed events (cinema releases, fireworks, astrology, F1),
//! embedded in the binary and inserted at startup when missing. Events whose
//! date is older than the three-month window are not (re)inserted, matching
//! the backup retention policy.

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::entities::event::{self, Entity as Event};
use crate::state::three_months_ago;

const SEED_JSON: &str = include_str!("../seed_events.json");

#[derive(Deserialize)]
struct SeedEvent {
    /// "YYYY-MM-DD"
    date: String,
    title: String,
    description: Option<String>,
    color: Option<String>,
}

pub async fn seed(db: &DatabaseConnection) {
    let seeds: Vec<SeedEvent> = match serde_json::from_str(SEED_JSON) {
        Ok(s) => s,
        Err(e) => {
            log::error!("invalid seed_events.json: {e}");
            return;
        }
    };
    let cutoff = three_months_ago();
    let mut inserted = 0usize;
    for s in seeds {
        // 08:00–20:00 UTC stays within the same civil day in France (UTC+1/+2)
        // whatever the season, so all-day events land on a single grid cell.
        let start = format!("{}T08:00:00Z", s.date);
        let end = format!("{}T20:00:00Z", s.date);
        if end < cutoff {
            continue;
        }
        let already = Event::find()
            .filter(event::Column::Title.eq(s.title.clone()))
            .filter(event::Column::Start.eq(start.clone()))
            .count(db)
            .await;
        match already {
            Ok(0) => {}
            Ok(_) => continue,
            Err(e) => {
                log::warn!("seed lookup failed, skipping: {e}");
                continue;
            }
        }
        let active = event::ActiveModel {
            title: Set(s.title),
            description: Set(s.description),
            start: Set(start),
            end: Set(end),
            all_day: Set(true),
            color: Set(s.color),
            ..Default::default()
        };
        match active.insert(db).await {
            Ok(_) => inserted += 1,
            Err(e) => log::warn!("failed to insert a seed event: {e}"),
        }
    }
    if inserted > 0 {
        log::info!("seeded {inserted} themed events");
    }
}
