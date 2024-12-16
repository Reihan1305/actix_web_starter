#[warn(unused_doc_comments)]

mod modules;
mod utils;
mod midleware;
mod service;

use std::time::SystemTime;
use actix_cors::Cors;
use actix_web::{get, HttpResponse, Responder};
use actix_web::{http::header, web, App, HttpServer};
use actix_web::middleware::Logger;
use actix_web::rt::spawn;
use actix_web::web::scope;
use cronjob::CronJob;
use dotenv::dotenv;
use lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::BasicProperties;
use r2d2_redis::redis::Commands;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use modules::post::post_handler::public_post_config;
use service::rabbitmq::{rabbit_connect, RabbitMqPool};
use service::redis::{redis_connect, RedisPool};

pub struct AppState {
    db: Pool<Postgres>,
    redis:RedisPool,
    rabbit:RabbitMqPool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    dotenv().ok();
    let mut secondly =  CronJob::new("testing cron", schedular_test);
    secondly.seconds("1");
    env_logger::init();
    let database_url:String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

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
        println!("🚀 schedular start 🚀");
        secondly.start_job();
    });
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
    let elapsed = start_time.elapsed().expect("Time went backwards");
    println!("Job executed at: {:?}", elapsed);
    println!("test schedular : {}", name);
}

#[get("/healthcheck")]
pub async fn api_health_check(
    data:web::Data<AppState>
)-> impl Responder {
    let mut message : String = String::from("");
    match sqlx::query("SELECT 1;").fetch_one(&data.db).await {
        Ok(_) => {
            message.push_str("database is healty");
            println!("Database healthy")
        },
        Err(err) =>{
            let error_message = format!("database cant connect : {}", err);
            message.push_str(&error_message);
            println!("{}",error_message)
        }
    }

    let redis_pool = &data.redis;
    let conn_redis = redis_pool.get();

    match conn_redis {
        Ok(mut conn)=>{
            let _:() = conn.set("testing_redis", "yoo").expect("failed to set key");
            let redis_value :String = conn.get("testing_redis").expect("failed to get key");
            message.push_str(", redis healty");
            println!("redis healty {}",redis_value)
        },
        Err(err)=>{
            let error_message = format!(", redis cant connect : {}", err);
            message.push_str(&error_message);
            println!("{}",error_message)
        }

    }

    let rabbit_pool = &data.rabbit;
    let rabbit_conn = rabbit_pool.get().await;

    match rabbit_conn{
        Ok(conn)=>{
            let channel = conn.create_channel().await.expect("failed to create channel");
            channel.queue_declare("test queue", QueueDeclareOptions::default(), FieldTable::default()).await.expect("failed to declare queue");
            let _ = channel.basic_publish("","test_queue",BasicPublishOptions::default(),
             b"Hello, RabbitMQ!",
             BasicProperties::default(),
         ).await.expect("Failed to publish message");  
         message.push_str(", rabbit mq connect success");
         println!("rabbit mq testing success")
        },
        Err(err)=>{
            let error_message = format!(", rabbit cant connect : {}", err);
            message.push_str(&error_message);
            println!("{}",error_message)
        }
    }
    message.push_str(", api healty ready to go 🚀🚀");
    HttpResponse::Ok().json(json!({"status":"success","message":message}))
}
