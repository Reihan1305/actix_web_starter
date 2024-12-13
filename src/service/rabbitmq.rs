use deadpool_lapin::{Config, Manager, Pool, Timeouts};
use lapin::ConnectionProperties;
use std::{env, time::Duration};

pub type RabbitMqPool = Pool;

pub fn rabbit_connect() -> RabbitMqPool {
    let rabbitmq_url = env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set");

    let config = Config {
        url: Some(rabbitmq_url),
        ..Default::default()
    };

    let manager = Manager::new(config.url.unwrap(),ConnectionProperties::default());

    Pool::builder(manager)
    .max_size(15)
    .timeouts(Timeouts{
        wait:Some(Duration::from_secs(60)),
        create:Some(Duration::from_secs(60)),
        recycle:Some(Duration::from_secs(60))
    })
    .runtime(deadpool_lapin::Runtime::Tokio1)
    .build()
    .expect("failed to create rabbit pool")
}   
