use std::sync::RwLock;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::entities::event::{self, Entity as Event};

/// ISO 8601 UTC timestamp of three months (90 days) ago. Stored datetimes are
/// ISO 8601 UTC strings, so lexicographic comparison is chronological.
pub fn three_months_ago() -> String {
    (chrono::Utc::now() - chrono::Duration::days(90))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// In-memory snapshot of the last three months of events, kept in sync with
/// the database after every mutation. On shutdown it is serialized to a
/// Heroku config var (see `backup`), which survives dyno replacement —
/// unlike the dyno filesystem where the SQLite file lives.
pub struct Snapshot(RwLock<Vec<event::Model>>);

impl Snapshot {
    pub fn new() -> Self {
        Snapshot(RwLock::new(Vec::new()))
    }

    pub async fn refresh(&self, db: &DatabaseConnection) {
        match Event::find()
            .filter(event::Column::End.gte(three_months_ago()))
            .order_by_asc(event::Column::Start)
            .all(db)
            .await
        {
            Ok(events) => {
                log::debug!("snapshot refreshed: {} events", events.len());
                *self.0.write().unwrap() = events;
            }
            Err(e) => log::warn!("snapshot refresh failed: {e}"),
        }
    }

    pub fn events(&self) -> Vec<event::Model> {
        self.0.read().unwrap().clone()
    }
}
