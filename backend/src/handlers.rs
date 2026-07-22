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

/// Weather for the selected beaches AND cities (Open-Meteo, cached ~30 min).
#[get("/beach-weather")]
pub async fn get_beach_weather(
    db: web::Data<DatabaseConnection>,
    cache: web::Data<crate::weather::WeatherCache>,
) -> impl Responder {
    let places = crate::weather::selected_places(db.get_ref()).await;
    let spots = crate::weather::for_places(cache.get_ref(), &places).await;
    HttpResponse::Ok().json(serde_json::json!({ "spots": spots }))
}

// ---------------------------------------------------------------------------
// Weather cities: catalog + user selection (same pattern as tide spots)

#[derive(serde::Serialize)]
pub struct WeatherCity {
    key: &'static str,
    name: &'static str,
    selected: bool,
}

#[derive(Deserialize)]
pub struct WeatherCitiesPayload {
    pub cities: Vec<String>,
}

async fn weather_cities_response(db: &DatabaseConnection) -> Vec<WeatherCity> {
    let selected: std::collections::HashSet<&str> = crate::weather::selected_cities(db)
        .await
        .into_iter()
        .map(|c| c.key)
        .collect();
    crate::weather::CITIES
        .iter()
        .map(|c| WeatherCity {
            key: c.key,
            name: c.name,
            selected: selected.contains(c.key),
        })
        .collect()
}

#[get("/weather-cities")]
pub async fn get_weather_cities(db: web::Data<DatabaseConnection>) -> impl Responder {
    HttpResponse::Ok().json(weather_cities_response(db.get_ref()).await)
}

#[put("/weather-cities")]
pub async fn put_weather_cities(
    db: web::Data<DatabaseConnection>,
    payload: web::Json<WeatherCitiesPayload>,
) -> impl Responder {
    let tokens: Vec<String> = payload
        .into_inner()
        .cities
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    for t in &tokens {
        if !crate::weather::CITIES.iter().any(|c| c.key == *t) {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": format!("ville inconnue : {t}") }));
        }
    }
    if let Err(e) =
        crate::settings::set(db.get_ref(), crate::weather::CITIES_SETTING, &tokens.join(",")).await
    {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": e.to_string() }));
    }
    // No events to add/remove: weather is served live and the cache key
    // (selected place keys) changes with the selection.
    HttpResponse::Ok().json(weather_cities_response(db.get_ref()).await)
}

// ---------------------------------------------------------------------------
// Tide spots: catalog + user selection (the in-app dropdown)

#[derive(serde::Serialize)]
pub struct TideSpot {
    key: &'static str,
    name: &'static str,
    group: &'static str,
    selected: bool,
}

#[derive(Deserialize)]
pub struct TideSpotsPayload {
    pub spots: Vec<String>,
}

async fn tide_spots_response(db: &DatabaseConnection) -> Vec<TideSpot> {
    let selected: std::collections::HashSet<&str> = crate::tides::selected_ports(db)
        .await
        .into_iter()
        .map(|p| p.key)
        .collect();
    crate::tides::PORTS
        .iter()
        .map(|p| TideSpot {
            key: p.key,
            name: p.name,
            group: p.group,
            selected: selected.contains(p.key),
        })
        .collect()
}

#[get("/tide-spots")]
pub async fn get_tide_spots(db: web::Data<DatabaseConnection>) -> impl Responder {
    HttpResponse::Ok().json(tide_spots_response(db.get_ref()).await)
}

#[put("/tide-spots")]
pub async fn put_tide_spots(
    db: web::Data<DatabaseConnection>,
    snap: web::Data<Snapshot>,
    payload: web::Json<TideSpotsPayload>,
) -> impl Responder {
    let tokens: Vec<String> = payload
        .into_inner()
        .spots
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    // Every token must be a known spot key or group name
    for t in &tokens {
        let known = crate::tides::PORTS.iter().any(|p| p.key == *t || p.group == *t);
        if !known {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": format!("spot inconnu : {t}") }));
        }
    }

    let old: std::collections::HashSet<&str> = crate::tides::selected_ports(db.get_ref())
        .await
        .into_iter()
        .map(|p| p.key)
        .collect();
    let new_ports = crate::tides::ports_for_tokens(&tokens);
    let new_keys: std::collections::HashSet<&str> = new_ports.iter().map(|p| p.key).collect();

    if let Err(e) = crate::settings::set(db.get_ref(), crate::tides::SPOTS_SETTING, &tokens.join(",")).await {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": e.to_string() }));
    }

    // Drop tide events of deselected spots
    for port in crate::tides::PORTS.iter() {
        if old.contains(port.key) && !new_keys.contains(port.key) {
            let res = Event::delete_many()
                .filter(event::Column::Color.eq(crate::tides::TIDE_COLOR))
                .filter(event::Column::Title.starts_with(format!("🌊 {} — ", port.name)))
                .exec(db.get_ref())
                .await;
            if let Err(e) = res {
                log::warn!("could not delete tides for {}: {e}", port.name);
            }
        }
    }

    // Fetch tides right away for newly selected spots
    let added: Vec<&crate::tides::Port> = new_ports
        .into_iter()
        .filter(|p| !old.contains(p.key))
        .collect();
    if !added.is_empty() {
        let now = chrono::Utc::now().timestamp();
        let candidates = crate::tides::fetch(&added, now).await;
        let inserted = crate::seed::insert_new_events(db.get_ref(), candidates).await;
        log::info!("tide selection change: {inserted} events inserted for {} new spots", added.len());
    }

    snap.refresh(db.get_ref()).await;
    HttpResponse::Ok().json(tide_spots_response(db.get_ref()).await)
}
