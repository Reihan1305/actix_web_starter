mod modules;
mod utils;
mod midleware;
use actix_cors::Cors;
use actix_web::{get, HttpResponse, Responder};
use actix_web::middleware::Logger;
use actix_web::{http::header, web, App, HttpServer};
use dotenv::dotenv;
use modules::auth::auth_handler::auth_config;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("🚀 Server started successfully");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .wrap(cors)
            .wrap(Logger::default())
            .service(api_health_check)
            .configure(auth_config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/healthcheck")]
pub async fn api_health_check()-> impl Responder {
    let message : &str = "api healty ready to go 🚀🚀";
    HttpResponse::Ok().json(json!({"status":"success","message":message}))
}