use std::env;
use r2d2_redis::{r2d2::Pool, RedisConnectionManager};

pub type RedisPool = Pool<RedisConnectionManager>;

pub fn redis_connect() -> RedisPool{
    let redis_hostname=env::var("REDIS_HOSTNAME").expect("hostname empty please fill");
    let redis_password = env::var("REDIS_PASSWORD").unwrap_or_default();

    let conn_url = format!("redis://{}@{}",redis_password,redis_hostname);

    let manager: RedisConnectionManager = RedisConnectionManager::new(conn_url).expect("Invalid connection URL");

    Pool::builder()
        .min_idle(Some(5))
        .max_size(50) 
        .build(manager)
        .expect("Failed to create Redis connection pool")
}