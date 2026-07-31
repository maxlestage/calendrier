mod astro;
mod astronomy;
mod backup;
mod entities;
mod f1;
mod handlers;
mod holidays;
mod migration;
mod schedule;
mod seed;
mod settings;
mod state;
mod tides;
mod tmdb;
mod weather;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, App, HttpServer};
use migration::Migrator;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

/// The SPA shell (index.html) as an HTML response with `Cache-Control: no-cache`
/// so browsers always revalidate it and pick up new deploys immediately.
fn shell_response(html: &str) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"))
        .body(html.to_owned())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // On Heroku, DATABASE_URL is a postgres:// URL provided by the addon;
    // locally we fall back to a SQLite file.
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://calendar.db?mode=rwc".into());
    let db = Database::connect(&db_url)
        .await
        .expect("failed to connect to database");
    Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

    // If a previous dyno saved a backup config var, seed the (empty) database
    // from it, then load the last three months into the in-memory snapshot.
    backup::restore_from_env(&db).await;
    seed::seed(&db).await;
    // Repaint any former violet events to the new stainless-steel colour.
    seed::recolor_violet_to_steel(&db).await;
    let snapshot = web::Data::new(state::Snapshot::new());
    snapshot.refresh(&db).await;
    let weather_cache = web::Data::new(weather::WeatherCache::new());

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Directory of the built frontend (frontend/dist). When present, the
    // backend serves the SPA itself so a single process serves everything.
    let static_dir =
        std::env::var("STATIC_DIR").unwrap_or_else(|_| "frontend/dist".into());
    let index_html: Option<String> =
        std::fs::read_to_string(std::path::Path::new(&static_dir).join("index.html")).ok();
    let serve_static = index_html.is_some();
    if serve_static {
        log::info!("serving frontend from {static_dir}");
    } else {
        log::info!("no frontend build found at {static_dir}, running API-only");
    }

    log::info!("listening on http://{host}:{port}");

    let db_data = web::Data::new(db);
    let server = {
        let db_data = db_data.clone();
        let snapshot = snapshot.clone();
        let weather_cache = weather_cache.clone();
        let index_html = index_html.clone();
        HttpServer::new(move || {
        let mut app = App::new()
            .app_data(db_data.clone())
            .app_data(snapshot.clone())
            .app_data(weather_cache.clone())
            .wrap(Cors::permissive())
            .service(
                web::scope("/api")
                    .service(handlers::list_events)
                    .service(handlers::get_event)
                    .service(handlers::create_event)
                    .service(handlers::create_events_batch)
                    .service(handlers::parse_schedule)
                    .service(handlers::update_event)
                    .service(handlers::delete_event)
                    .service(handlers::export_events)
                    .service(handlers::get_state)
                    .service(handlers::import_state)
                    .service(handlers::get_tide_spots)
                    .service(handlers::put_tide_spots)
                    .service(handlers::get_beach_weather)
                    .service(handlers::get_weather_cities)
                    .service(handlers::put_weather_cities)
                    .service(handlers::calendar_ics)
                    .service(handlers::event_ics)
                    .service(handlers::export_csv)
                    .service(handlers::print_html)
                    .service(handlers::get_prefs)
                    .service(handlers::put_prefs),
            );
        if let Some(html) = index_html.clone() {
            // The HTML shell must never be cached, or phones keep serving a stale
            // app (old colours/code) after a deploy. It's served from memory with
            // Cache-Control: no-cache for "/" and every SPA route; hashed assets
            // under /assets/ change name each build, so they stay cacheable.
            let root_get = html.clone();
            let root_head = html.clone();
            let spa_html = html;
            app = app
                .service(
                    web::resource("/")
                        .route(web::get().to(move || {
                            let body = root_get.clone();
                            async move { shell_response(&body) }
                        }))
                        .route(web::head().to(move || {
                            let body = root_head.clone();
                            async move { shell_response(&body) }
                        })),
                )
                .service(
                    Files::new("/", &static_dir).default_handler(move |req: ServiceRequest| {
                        let body = spa_html.clone();
                        async move {
                            let (req, _) = req.into_parts();
                            Ok(ServiceResponse::new(req, shell_response(&body)))
                        }
                    }),
                );
        }
        app
        })
        .bind((host, port))?
        .run()
    };

    // Blocks until shutdown. Heroku sends SIGTERM before replacing a dyno;
    // actix stops gracefully and we get a chance to persist the snapshot.
    server.await?;

    log::info!("shutting down, backing up events and settings");
    snapshot.refresh(db_data.get_ref()).await;
    backup::push_to_heroku(db_data.get_ref(), &snapshot.events()).await;
    Ok(())
}
