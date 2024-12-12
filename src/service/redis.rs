use std::env;
use r2d2_redis::{r2d2::Pool, RedisConnectionManager};

pub type RedisPool = Pool<RedisConnectionManager>;

pub fn connect() -> RedisPool{
    let redis_hostname=env::var("REDIS_HOSTNAME").expect("hostname empty please fill");
    let redis_password = env::var("REDIS_PASSWORD").unwrap_or_default();

    let uri_scheme = match env::var("IS_TLS"){
        Ok(_)=>"rediss",
        Err(_)=>"redis"
    };

    let conn_url = format!("{}://{}@{}",uri_scheme,redis_password,redis_hostname);

    let manager = RedisConnectionManager::new(conn_url).expect("Invalid connection URL");

    
    // Buat pool dengan r2d2
    Pool::builder()
        .max_size(15) 
        .build(manager)
        .expect("Failed to create Redis connection pool")
    // redis::Client::open(conn_url)
    // .expect("invalid connection url")
    // .get_connection()
    // .expect("failed to connect redis")

}