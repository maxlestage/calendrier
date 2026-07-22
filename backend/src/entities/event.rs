use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    /// ISO 8601 datetime (UTC)
    pub start: String,
    /// ISO 8601 datetime (UTC)
    pub end: String,
    pub all_day: bool,
    pub color: Option<String>,
    /// "weekly" | "monthly" | "yearly" — None for one-shot events.
    /// serde(default) keeps pre-recurrence CALENDAR_BACKUP payloads readable.
    #[serde(default)]
    pub recurrence: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
