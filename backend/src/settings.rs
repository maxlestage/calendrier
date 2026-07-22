//! Tiny key/value settings persisted in the database (survive restarts via
//! the same SQLite file and, on Heroku, via the CALENDAR_BACKUP mechanism
//! only for events — settings are cheap to re-pick if lost).

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::entities::setting::{self, Entity as Setting};

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
