use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;

use crate::entities::event::{self, Entity as Event};
use crate::state::Snapshot;

#[derive(Deserialize)]
pub struct EventPayload {
    pub title: String,
    pub description: Option<String>,
    pub start: String,
    pub end: String,
    #[serde(default)]
    pub all_day: bool,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct RangeQuery {
    /// ISO 8601 lower bound (inclusive) on event end
    pub from: Option<String>,
    /// ISO 8601 upper bound (exclusive) on event start
    pub to: Option<String>,
}

#[get("/events")]
pub async fn list_events(
    db: web::Data<DatabaseConnection>,
    query: web::Query<RangeQuery>,
) -> impl Responder {
    let mut select = Event::find().order_by_asc(event::Column::Start);
    // ISO 8601 UTC strings compare lexicographically in chronological order,
    // so string comparison is a valid range filter here.
    if let Some(from) = &query.from {
        select = select.filter(event::Column::End.gte(from.clone()));
    }
    if let Some(to) = &query.to {
        select = select.filter(event::Column::Start.lt(to.clone()));
    }
    match select.all(db.get_ref()).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[get("/events/{id}")]
pub async fn get_event(db: web::Data<DatabaseConnection>, path: web::Path<i32>) -> impl Responder {
    match Event::find_by_id(path.into_inner()).one(db.get_ref()).await {
        Ok(Some(ev)) => HttpResponse::Ok().json(ev),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "event not found" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[post("/events")]
pub async fn create_event(
    db: web::Data<DatabaseConnection>,
    snap: web::Data<Snapshot>,
    payload: web::Json<EventPayload>,
) -> impl Responder {
    let payload = payload.into_inner();
    let active = event::ActiveModel {
        title: Set(payload.title),
        description: Set(payload.description),
        start: Set(payload.start),
        end: Set(payload.end),
        all_day: Set(payload.all_day),
        color: Set(payload.color),
        ..Default::default()
    };
    match active.insert(db.get_ref()).await {
        Ok(ev) => {
            snap.refresh(db.get_ref()).await;
            HttpResponse::Created().json(ev)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[put("/events/{id}")]
pub async fn update_event(
    db: web::Data<DatabaseConnection>,
    snap: web::Data<Snapshot>,
    path: web::Path<i32>,
    payload: web::Json<EventPayload>,
) -> impl Responder {
    let id = path.into_inner();
    let existing = match Event::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(ev)) => ev,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({ "error": "event not found" }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": e.to_string() }))
        }
    };
    let payload = payload.into_inner();
    let mut active: event::ActiveModel = existing.into();
    active.title = Set(payload.title);
    active.description = Set(payload.description);
    active.start = Set(payload.start);
    active.end = Set(payload.end);
    active.all_day = Set(payload.all_day);
    active.color = Set(payload.color);
    match active.update(db.get_ref()).await {
        Ok(ev) => {
            snap.refresh(db.get_ref()).await;
            HttpResponse::Ok().json(ev)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[delete("/events/{id}")]
pub async fn delete_event(
    db: web::Data<DatabaseConnection>,
    snap: web::Data<Snapshot>,
    path: web::Path<i32>,
) -> impl Responder {
    match Event::delete_by_id(path.into_inner()).exec(db.get_ref()).await {
        Ok(res) if res.rows_affected > 0 => {
            snap.refresh(db.get_ref()).await;
            HttpResponse::NoContent().finish()
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({ "error": "event not found" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// Manual export of the in-memory snapshot (last three months), as JSON.
#[get("/export")]
pub async fn export_events(snap: web::Data<Snapshot>) -> impl Responder {
    HttpResponse::Ok().json(snap.events())
}
