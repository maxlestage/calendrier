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
    /// "weekly" | "monthly" | "yearly" — omitted/null for one-shot events
    #[serde(default)]
    pub recurrence: Option<String>,
}

fn normalize_recurrence(r: Option<String>) -> Result<Option<String>, HttpResponse> {
    match r.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(v @ ("weekly" | "monthly" | "yearly")) => Ok(Some(v.to_string())),
        Some(other) => Err(HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": format!("récurrence inconnue : {other}") }))),
    }
}

#[derive(Deserialize)]
pub struct RangeQuery {
    /// ISO 8601 lower bound (inclusive) on event end
    pub from: Option<String>,
    /// ISO 8601 upper bound (exclusive) on event start
    pub to: Option<String>,
    /// Title search (substring, case-insensitive for ASCII)
    pub q: Option<String>,
}

fn parse_iso(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(&s.replace('Z', "+00:00"))
        .ok()
        .map(|d| d.with_timezone(&chrono::Utc))
}

/// Expand a recurring event into its occurrences overlapping [from, to).
/// Occurrences reuse the base event's id: editing/deleting one edits the
/// whole series.
fn expand_recurring(
    ev: &event::Model,
    from: Option<&str>,
    to: Option<&str>,
) -> Vec<event::Model> {
    let Some(rule) = ev.recurrence.as_deref() else {
        return vec![ev.clone()];
    };
    let (Some(base_start), Some(base_end)) = (parse_iso(&ev.start), parse_iso(&ev.end)) else {
        return vec![ev.clone()];
    };
    let now = chrono::Utc::now();
    let win_from = from
        .and_then(parse_iso)
        .unwrap_or_else(|| now - chrono::Duration::days(90));
    let win_to = to
        .and_then(parse_iso)
        .unwrap_or_else(|| now + chrono::Duration::days(400));
    let duration = base_end - base_start;
    let fmt = |d: chrono::DateTime<chrono::Utc>| d.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut out = Vec::new();
    // Occurrence n is computed from the base each time (no cumulative
    // drift; monthly clamps the 31st to shorter months' last day).
    for n in 0..600u32 {
        let start = match rule {
            "weekly" => base_start + chrono::Duration::weeks(n as i64),
            "monthly" => match base_start.checked_add_months(chrono::Months::new(n)) {
                Some(d) => d,
                None => break,
            },
            "yearly" => match base_start.checked_add_months(chrono::Months::new(12 * n)) {
                Some(d) => d,
                None => break,
            },
            _ => return vec![ev.clone()],
        };
        if start >= win_to {
            break;
        }
        let end = start + duration;
        if end >= win_from {
            let mut occ = ev.clone();
            occ.start = fmt(start);
            occ.end = fmt(end);
            out.push(occ);
        }
    }
    out
}

#[get("/events")]
pub async fn list_events(
    db: web::Data<DatabaseConnection>,
    query: web::Query<RangeQuery>,
) -> impl Responder {
    // One-shot events: plain range query. ISO 8601 UTC strings compare
    // lexicographically in chronological order, so string comparison is a
    // valid range filter here.
    let mut select = Event::find()
        .filter(event::Column::Recurrence.is_null())
        .order_by_asc(event::Column::Start);
    if let Some(from) = &query.from {
        select = select.filter(event::Column::End.gte(from.clone()));
    }
    if let Some(to) = &query.to {
        select = select.filter(event::Column::Start.lt(to.clone()));
    }
    if let Some(q) = &query.q {
        select = select.filter(event::Column::Title.contains(q.clone()));
    }
    let mut events = match select.all(db.get_ref()).await {
        Ok(events) => events,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": e.to_string() }))
        }
    };
    // Recurring events: fetched without a range (their base may be long
    // before the window) and expanded into occurrences.
    let mut recurring = Event::find().filter(event::Column::Recurrence.is_not_null());
    if let Some(q) = &query.q {
        recurring = recurring.filter(event::Column::Title.contains(q.clone()));
    }
    match recurring.all(db.get_ref()).await {
        Ok(list) => {
            for ev in &list {
                events.extend(expand_recurring(ev, query.from.as_deref(), query.to.as_deref()));
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": e.to_string() }))
        }
    }
    events.sort_by(|a, b| a.start.cmp(&b.start));
    HttpResponse::Ok().json(events)
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
    let recurrence = match normalize_recurrence(payload.recurrence) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    let active = event::ActiveModel {
        title: Set(payload.title),
        description: Set(payload.description),
        start: Set(payload.start),
        end: Set(payload.end),
        all_day: Set(payload.all_day),
        color: Set(payload.color),
        recurrence: Set(recurrence),
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
    let recurrence = match normalize_recurrence(payload.recurrence) {
        Ok(r) => r,
        Err(resp) => return resp,
    };
    let mut active: event::ActiveModel = existing.into();
    active.title = Set(payload.title);
    active.description = Set(payload.description);
    active.start = Set(payload.start);
    active.end = Set(payload.end);
    active.all_day = Set(payload.all_day);
    active.color = Set(payload.color);
    active.recurrence = Set(recurrence);
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

// ---------------------------------------------------------------------------
// Device backup: the web app (iOS webview/PWA) keeps a copy of this state in
// the phone's persistent storage and pushes it back if a fresh dyno booted
// without any server-side backup.

#[derive(serde::Serialize, Deserialize)]
pub struct StatePayload {
    #[serde(default)]
    pub events: Vec<event::Model>,
    #[serde(default)]
    pub settings: Vec<crate::entities::setting::Model>,
}

/// Snapshot + settings, the unit the device stores locally.
#[get("/state")]
pub async fn get_state(
    db: web::Data<DatabaseConnection>,
    snap: web::Data<Snapshot>,
) -> impl Responder {
    let settings = crate::entities::setting::Entity::find()
        .all(db.get_ref())
        .await
        .unwrap_or_default();
    HttpResponse::Ok().json(StatePayload {
        events: snap.events(),
        settings,
    })
}

/// Restore a device backup: settings first (upsert), then events
/// (deduplicated — safe to import over freshly seeded data).
#[post("/import")]
pub async fn import_state(
    db: web::Data<DatabaseConnection>,
    snap: web::Data<Snapshot>,
    payload: web::Json<StatePayload>,
) -> impl Responder {
    let payload = payload.into_inner();
    let mut settings_set = 0usize;
    for s in &payload.settings {
        if s.key.trim().is_empty() {
            continue;
        }
        match crate::settings::set(db.get_ref(), &s.key, &s.value).await {
            Ok(()) => settings_set += 1,
            Err(e) => log::warn!("import: could not set {}: {e}", s.key),
        }
    }
    let inserted = crate::seed::import_events(db.get_ref(), payload.events).await;
    // The restored selection may call for tides/vacations that a blank dyno
    // never fetched
    crate::holidays::sync_vacations(db.get_ref()).await;
    snap.refresh(db.get_ref()).await;
    log::info!("device restore: {inserted} events, {settings_set} settings");
    HttpResponse::Ok().json(serde_json::json!({
        "events_inserted": inserted,
        "settings_set": settings_set,
    }))
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
    snap: web::Data<Snapshot>,
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
    // Weather itself is served live (the cache key follows the selection),
    // but school vacations are events and track the zones of the selected
    // places — keep them in sync.
    crate::holidays::sync_vacations(db.get_ref()).await;
    snap.refresh(db.get_ref()).await;
    HttpResponse::Ok().json(weather_cities_response(db.get_ref()).await)
}

// ---------------------------------------------------------------------------
// Notification preferences (used by the web to build the iOS reminders)

#[get("/prefs")]
pub async fn get_prefs(db: web::Data<DatabaseConnection>) -> impl Responder {
    HttpResponse::Ok().json(crate::settings::notif_prefs(db.get_ref()).await)
}

#[put("/prefs")]
pub async fn put_prefs(
    db: web::Data<DatabaseConnection>,
    payload: web::Json<crate::settings::NotifPrefs>,
) -> impl Responder {
    let prefs = payload.into_inner().sanitized();
    match serde_json::to_string(&prefs) {
        Ok(json) => {
            if let Err(e) =
                crate::settings::set(db.get_ref(), crate::settings::NOTIF_PREFS_SETTING, &json).await
            {
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "error": e.to_string() }));
            }
            HttpResponse::Ok().json(prefs)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ---------------------------------------------------------------------------
// ICS feed — subscribe from the native iOS/Android/desktop calendar

fn ics_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
        .replace('\r', "")
}

/// Paris civil day of an ISO UTC instant, compact ICS form (YYYYMMDD).
fn ics_paris_date(iso: &str, plus_days: i64) -> Option<String> {
    let dt = parse_iso(iso)?;
    Some(
        (dt.with_timezone(&chrono_tz::Europe::Paris).date_naive()
            + chrono::Duration::days(plus_days))
        .format("%Y%m%d")
        .to_string(),
    )
}

/// The whole calendar as an iCalendar feed. Subscribing to
/// `/api/calendar.ics` from the iPhone's native Calendar (Réglages →
/// Apps → Calendrier → Comptes → Autre → Ajouter un cal. avec abonnement)
/// mirrors every event — tides included — with native notifications.
#[get("/calendar.ics")]
pub async fn calendar_ics(db: web::Data<DatabaseConnection>) -> impl Responder {
    let events = match Event::find()
        .order_by_asc(event::Column::Start)
        .all(db.get_ref())
        .await
    {
        Ok(evs) => evs,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": e.to_string() }))
        }
    };
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let mut ics = String::from(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Calendrier//FR\r\n\
         CALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\nX-WR-CALNAME:Calendrier\r\n\
         X-WR-TIMEZONE:Europe/Paris\r\n",
    );
    for ev in &events {
        let compact =
            |iso: &str| iso.replace(['-', ':'], "");
        ics.push_str("BEGIN:VEVENT\r\n");
        ics.push_str(&format!("UID:{}@calendrier\r\n", ev.id));
        ics.push_str(&format!("DTSTAMP:{stamp}\r\n"));
        if ev.all_day {
            let (Some(d0), Some(d1)) = (
                ics_paris_date(&ev.start, 0),
                ics_paris_date(&ev.end, 1),
            ) else {
                ics.push_str("END:VEVENT\r\n");
                continue;
            };
            ics.push_str(&format!("DTSTART;VALUE=DATE:{d0}\r\n"));
            ics.push_str(&format!("DTEND;VALUE=DATE:{d1}\r\n"));
        } else {
            ics.push_str(&format!("DTSTART:{}\r\n", compact(&ev.start)));
            ics.push_str(&format!("DTEND:{}\r\n", compact(&ev.end)));
        }
        if let Some(rule) = ev.recurrence.as_deref() {
            let freq = match rule {
                "weekly" => "WEEKLY",
                "monthly" => "MONTHLY",
                "yearly" => "YEARLY",
                _ => "",
            };
            if !freq.is_empty() {
                ics.push_str(&format!("RRULE:FREQ={freq}\r\n"));
            }
        }
        ics.push_str(&format!("SUMMARY:{}\r\n", ics_escape(&ev.title)));
        if let Some(desc) = ev.description.as_deref() {
            if !desc.is_empty() {
                ics.push_str(&format!("DESCRIPTION:{}\r\n", ics_escape(desc)));
            }
        }
        ics.push_str("END:VEVENT\r\n");
    }
    ics.push_str("END:VCALENDAR\r\n");
    HttpResponse::Ok()
        .content_type("text/calendar; charset=utf-8")
        .body(ics)
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

    // School vacations follow the zones of the selected places
    crate::holidays::sync_vacations(db.get_ref()).await;

    snap.refresh(db.get_ref()).await;
    HttpResponse::Ok().json(tide_spots_response(db.get_ref()).await)
}
