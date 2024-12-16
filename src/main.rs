#[warn(unused_doc_comments)]
mod modules;
mod utils;
mod midleware;
mod service;
use std::time::{SystemTime, UNIX_EPOCH};

use actix_cors::Cors;
use actix_web::rt::spawn;
use actix_web::web::scope;
use actix_web::{get, HttpResponse, Responder};
use actix_web::middleware::Logger;
use actix_web::{http::header, web, App, HttpServer};
use cronjob::CronJob;
use dotenv::dotenv;
use lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::BasicProperties;
use modules::post::post_handler::public_post_config;
use r2d2_redis::redis::Commands;
use serde_json::json;
use service::rabbitmq::{rabbit_connect, RabbitMqPool};
use service::redis::{redis_connect, RedisPool};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};


// struct AppState that include database pool 
pub struct AppState {
    db: Pool<Postgres>,
    redis:RedisPool,
    rabbit:RabbitMqPool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //initailize rust log
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    //reading the .env file
    dotenv().ok();
    let mut secondly =  CronJob::new("testing cron", schedular_test);
    secondly.seconds("1");
    //initialize env logger
    env_logger::init();
    //get database_url from env
    let database_url:String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    //create postgres pool
    let pool = match PgPoolOptions::new()
        .min_connections(5)
        .max_connections(50)
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

    let redis_conn =  redis_connect();
    let rabbit_conn = rabbit_connect();
    println!("🚀 Server started successfully");
    
    spawn(async move {
        println!("schedular start");
        secondly.start_job();  // Mulai cron job
    });
    //create server with actix web
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
            .app_data(web::Data::new(AppState { db: pool.clone() ,redis:redis_conn.clone(),rabbit:rabbit_conn.clone()}))
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                scope("/api")
                .service(test_rabbitmq)
                .service(test_redis)
                .service(api_health_check)
                .configure(public_post_config)
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

pub fn schedular_test(name:&str){
    let start_time = SystemTime::now();
    let elapsed = start_time.duration_since(UNIX_EPOCH).expect("Time went backwards");
    println!("Job executed at: {:?}", elapsed);
    println!("test schedular : {}", name);
}

#[get("/healthcheck")]
pub async fn api_health_check()-> impl Responder {
    let message : &str = "api healty ready to go 🚀🚀";
    HttpResponse::Ok().json(json!({"status":"success","message":message}))
}


#[get("/test-redis")]
pub async fn test_redis(data: web::Data<AppState>) -> impl Responder {
    let pool = &data.redis;

    // Ambil koneksi dari pool
    let mut conn = pool.get().expect("Failed to get Redis connection");

    // Lakukan operasi Redis
    let _: () = conn.set("key", "value").expect("Failed to set key");
    let value: String = conn.get("key").expect("Failed to get key");

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "value": value
    }))
}

#[get("/test-rabbitmq")]
pub async fn test_rabbitmq(data: web::Data<AppState>) -> impl Responder {
    let pool = &data.rabbit;

    // Ambil koneksi dari pool RabbitMQ
    let conn = pool.get().await.    expect("cant connect to pool");
    let channel = conn.create_channel().await.expect("Failed to create channel");

    // Membuat antrian
    channel.queue_declare(
            "test_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    // Mengirim pesan ke antrian
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

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Message sent to RabbitMQ"
    }))
}
