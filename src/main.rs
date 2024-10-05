use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use chrono::Duration;
use std::collections::HashSet;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::langs::Langs;
use crate::messages::{Message, remove_old_messages};

mod langs;
mod clients;
mod messages;
mod routing;
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(HashMap::new()));
    let en_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let es_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let fr_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let it_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let po_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let de_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));

    let en_repo_clone = en_message_repo.clone();
    let es_repo_clone = es_message_repo.clone();
    let fr_repo_clone = fr_message_repo.clone();
    let it_repo_clone = it_message_repo.clone();
    let po_repo_clone = po_message_repo.clone();
    let de_repo_clone = de_message_repo.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60 * 60)); // Run hourly
        loop {
            interval.tick().await;
            let max_age = Duration::weeks(1);
            remove_old_messages(en_repo_clone.clone(), max_age);
            remove_old_messages(es_repo_clone.clone(), max_age);
            remove_old_messages(fr_repo_clone.clone(), max_age);
            remove_old_messages(it_repo_clone.clone(), max_age);
            remove_old_messages(po_repo_clone.clone(), max_age);
            remove_old_messages(de_repo_clone.clone(), max_age);
        }
    });

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(Data::new(tokens.clone()))
            .app_data(Data::new(en_message_repo.clone()))
            .app_data(Data::new(es_message_repo.clone()))
            .app_data(Data::new(fr_message_repo.clone()))
            .app_data(Data::new(it_message_repo.clone()))
            .app_data(Data::new(po_message_repo.clone()))
            .app_data(Data::new(de_message_repo.clone()))
            .configure(routing::configure_client_routes)
            .configure(routing::configure_message_routes)
    })
    //.bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 7273))?
    .run()
    .await
}