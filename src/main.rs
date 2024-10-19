use actix_web::{web::scope, middleware::Logger, web::Data, App, HttpServer};
use auth::ApiKeyMiddleware;
use std::sync::{Arc, Mutex};
use dotenv::dotenv;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;

mod auth;
mod clients;
mod langs;
mod messages;
mod routing;
mod security_headers;
mod tests;

use langs::Langs;
use messages::{remove_old_messages, Message};
use security_headers::SecurityHeaders;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect("Failed to read .env file"); 
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let config = aws_config::from_env().region(region_provider).load().await;
    let dynamodb_client = Client::new(&config);

    let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); 
        loop {
            interval.tick().await;
            remove_old_messages(messages_clone.clone());
        }
    });

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .wrap(SecurityHeaders)
            .app_data(Data::new(dynamodb_client.clone()))
            .app_data(Data::new(messages.clone()))
            .configure(routing::configure_client_routes)
            .configure(routing::configure_insecure_message_routes)
            .service(
                scope("/admin")
                    .wrap(ApiKeyMiddleware)
                    .configure(routing::configure_secure_message_routes),
            )
    })
    //.bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 7273))?
    .run()
    .await
}
