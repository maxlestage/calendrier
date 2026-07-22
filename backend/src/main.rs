mod astro;
mod astronomy;
mod backup;
mod entities;
mod f1;
mod handlers;
mod migration;
mod seed;
mod settings;
mod state;
mod tides;
mod tmdb;
mod weather;

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, App, HttpServer};
use migration::Migrator;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

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
    let serve_static = std::path::Path::new(&static_dir).join("index.html").exists();
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
                    .service(handlers::update_event)
                    .service(handlers::delete_event)
                    .service(handlers::export_events)
                    .service(handlers::get_tide_spots)
                    .service(handlers::put_tide_spots)
                    .service(handlers::get_beach_weather),
            );
        if serve_static {
            let index = std::path::Path::new(&static_dir).join("index.html");
            app = app.service(
                Files::new("/", &static_dir)
                    .index_file("index.html")
                    // SPA fallback: unknown paths get index.html
                    .default_handler(move |req: ServiceRequest| {
                        let index = index.clone();
                        async move {
                            let (req, _) = req.into_parts();
                            let file = NamedFile::open_async(index).await?;
                            let res = file.into_response(&req);
                            Ok(ServiceResponse::new(req, res))
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

    log::info!("shutting down, backing up events");
    snapshot.refresh(db_data.get_ref()).await;
    backup::push_to_heroku(&snapshot.events()).await;
    Ok(())
}
