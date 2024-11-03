use actix_route_rate_limiter::{LimiterBuilder, RateLimiter};
use actix_web::{middleware::Logger, web::scope, web::Data, App, HttpServer};
use auth::ApiKeyMiddleware;
use chrono::Duration;
use dotenv::dotenv;
use std::{env, sync::{Arc, Mutex}};

mod auth;
mod langs;
mod messages;
mod routing;
mod security_headers;
mod tests;

use messages::{remove_old_messages, Message};
use security_headers::SecurityHeaders;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect("Failed to read .env file");
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let listen_addr = env::var("LISTEN").expect("LISTEN must be set");

    let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            remove_old_messages(messages_clone.clone());
        }
    });

    let limiter = LimiterBuilder::new()
        .with_duration(Duration::minutes(1))
        .with_num_requests(60)
        .build();

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .wrap(SecurityHeaders)
            .wrap(RateLimiter::new(Arc::clone(&limiter)))
            .app_data(Data::new(messages.clone()))
            .configure(routing::configure_insecure_message_routes)
            .service(
                scope("/admin")
                    .wrap(ApiKeyMiddleware)
                    .configure(routing::configure_secure_message_routes),
            )
    })
    .bind(listen_addr)?
    .run()
    .await
}
