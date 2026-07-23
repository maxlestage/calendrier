//! Tiny key/value settings persisted in the database (survive restarts via
//! the same SQLite file and, on Heroku, via the CALENDAR_BACKUP mechanism —
//! events *and* settings are backed up).

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};

use crate::entities::setting::{self, Entity as Setting};

/// Notification preferences (read by the web to build the iOS reminder
/// schedule). Stored as JSON in the `notif_prefs` setting.
pub const NOTIF_PREFS_SETTING: &str = "notif_prefs";

fn default_hour() -> u8 {
    7
}
fn default_lead() -> u16 {
    15
}
fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
pub struct NotifPrefs {
    /// Hour (0–23, device-local) of the daily morning briefing
    #[serde(default = "default_hour")]
    pub morning_hour: u8,
    /// Minutes before a timed event to fire its reminder
    #[serde(default = "default_lead")]
    pub lead_min: u16,
    #[serde(default = "default_true")]
    pub morning_briefing: bool,
    #[serde(default = "default_true")]
    pub event_reminders: bool,
}

impl Default for NotifPrefs {
    fn default() -> Self {
        NotifPrefs {
            morning_hour: default_hour(),
            lead_min: default_lead(),
            morning_briefing: true,
            event_reminders: true,
        }
    }
}

impl NotifPrefs {
    /// Clamp to sane ranges before storing.
    pub fn sanitized(mut self) -> Self {
        if self.morning_hour > 23 {
            self.morning_hour = default_hour();
        }
        self.lead_min = self.lead_min.clamp(0, 24 * 60);
        self
    }
}

pub async fn notif_prefs(db: &DatabaseConnection) -> NotifPrefs {
    match get(db, NOTIF_PREFS_SETTING).await {
        Some(json) => serde_json::from_str(&json).unwrap_or_default(),
        None => NotifPrefs::default(),
    }
}

pub async fn get(db: &DatabaseConnection, key: &str) -> Option<String> {
    match Setting::find_by_id(key.to_string()).one(db).await {
        Ok(Some(row)) => Some(row.value),
        Ok(None) => None,
        Err(e) => {
            log::warn!("could not read setting {key}: {e}");
            None
        }
    }
}

pub async fn set(db: &DatabaseConnection, key: &str, value: &str) -> Result<(), sea_orm::DbErr> {
    let active = setting::ActiveModel {
        key: Set(key.to_string()),
        value: Set(value.to_string()),
    };
    // SQLite upsert: try insert, fall back to update
    match Setting::insert(active.clone())
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(setting::Column::Key)
                .update_column(setting::Column::Value)
                .to_owned(),
        )
        .exec(db)
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => active.update(db).await.map(|_| ()),
    }
}
