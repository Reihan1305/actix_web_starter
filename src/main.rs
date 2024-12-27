mod modules;
mod utils;
mod midleware;
mod service;
use std::{
    collections::HashMap,
    env::{
        var_os,
        set_var,
        var
    }
};
use actix_cors::Cors;
use actix_web::{http::header,middleware::Logger,{
    get, 
    HttpResponse, 
    Responder, 
    web::{self,scope}, 
    App, 
    HttpServer
}};
use lapin::{
    BasicProperties,
    types::FieldTable,
    options::{
    BasicPublishOptions, 
    QueueDeclareOptions
    }
};

use r2d2_redis::redis::Commands;
use serde_json::json;
use modules::post::post_handler::public_post_config;

/// Shared state for Actix App
pub struct AppState {
    db: sqlx::Pool<sqlx::Postgres>,
    redis: service::redis::RedisPool,
    rabbit: service::rabbitmq::RabbitMqPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    // Setup logger and environment variables
    env_logger::init();
    if var_os("RUST_LOG").is_none() {
        set_var("RUST_LOG", "actix_web=info");
    }

    //get port from env
    let port: u16 = var("PORT")
                    .expect("cant get port from env")
                    .parse::<u16>()
                    .expect("cant convert port to u16");

    // get database_url from env
    let database_url: String = var("DATABASE_URL")
                               .expect("cant get db url from env");

    // create initial pool database
    let pool: sqlx::Pool<sqlx::Postgres> = match sqlx::postgres::PgPoolOptions::new()
        .min_connections(5)
        .max_connections(50)
        .connect(&database_url)
        .await {
            Ok(pg_pool)=> {
                println!("✅ Connection to the database is successful!");
                pg_pool
            },
            Err(err) => {
            println!("failed to connect database: {}",err);
            std::process::exit(1)
            }
        };

    // Create redis connection pool
    let redis_conn: r2d2_redis::r2d2::Pool<r2d2_redis::RedisConnectionManager> = service::redis::redis_connect();

    // Create rabbitmq connection pool
    let rabbit_conn: deadpool_lapin::Pool = service::rabbitmq::rabbit_connect();

    // print the status server and the port
    println!("🚀 Server started successfully at port {:?}",port);
    
    // Start Actix server
    HttpServer::new(move || {
        //configure the cors
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
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                redis: redis_conn.clone(),
                rabbit: rabbit_conn.clone(),
            }))
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                scope("/api")
                    .service(api_health_check)
                    .configure(public_post_config),
            )
    })
    .bind(("0.0.0.0",port))?
    .run()
    .await
}

/// Health Check Endpoint
#[get("/healthcheck")]
pub async fn api_health_check(data: web::Data<AppState>) -> impl Responder {
    // let mut message = String::new();
    
    let mut error_messages:Vec<HashMap<String,String>> = vec![];
    // Database Health Check
    match sqlx::query("SELECT 1;").fetch_one(&data.db).await {
        Ok(_) => {
            println!("✅ Database healthy");
        }
        Err(err) => {
            let mut error_message:HashMap<String,String> = HashMap::new();
            error_message.insert("database".to_string(), format!("cant connect to database {}",err).to_string());
            error_messages.push(error_message);
        }
    }

    // Redis Health Check
    match data.redis.get() {
        Ok(mut conn) => {
            let _: () = conn.set("testing_redis", "yoo").expect("Failed to set Redis key");
            let redis_value: String = conn.get("testing_redis").expect("Failed to get Redis key");
            println!("✅ Redis healthy: {}", redis_value);
        }
        Err(err) => {
            let mut error_message:HashMap<String,String> = HashMap::new();
            error_message.insert("redis".to_string(), format!("cant connect to redis {}",err).to_string());
            error_messages.push(error_message);
        }
    }

    // RabbitMQ Health Check
    match data.rabbit.get().await {
        Ok(conn) => {
            let channel = conn.create_channel().await.expect("Failed to create channel");
                channel
                .queue_declare("test_queue", QueueDeclareOptions::default(), FieldTable::default())
                .await
                .expect("Failed to declare queue");
            let _ = channel
                .basic_publish(
                    "",
                    "test_queue",
                    BasicPublishOptions::default(),
                    b"Hello, RabbitMQ!",
                    BasicProperties::default(),
                )
                .await
                .expect("Failed to publish message");
                println!("✅ RabbitMQ is healthy");
        }
        Err(err) => {
            let mut error_message:HashMap<String,String> = HashMap::new();
            error_message.insert("redis".to_string(), format!("cant connect to redis {}",err).to_string());
            error_messages.push(error_message);
        }
    }
    if error_messages.len() >0{
        return HttpResponse::BadRequest().json(json!({"error":error_messages}));
    }
    HttpResponse::Ok().json(json!({ "status": "success", "message": "API healthy and ready to go 🚀🚀" }))
}
