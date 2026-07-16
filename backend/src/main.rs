mod entities;
mod handlers;
mod migration;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use migration::Migrator;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://calendar.db?mode=rwc".into());
    let db = Database::connect(&db_url)
        .await
        .expect("failed to connect to database");
    Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    log::info!("listening on http://{host}:{port}");

    let db_data = web::Data::new(db);
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .wrap(Cors::permissive())
            .service(
                web::scope("/api")
                    .service(handlers::list_events)
                    .service(handlers::get_event)
                    .service(handlers::create_event)
                    .service(handlers::update_event)
                    .service(handlers::delete_event),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
