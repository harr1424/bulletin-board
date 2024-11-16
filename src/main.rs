use actix_route_rate_limiter::{LimiterBuilder, RateLimiter};
use actix_web::{middleware::Logger, web::scope, web::Data, App, HttpServer};
use auth::ApiKeyMiddleware;
use chrono::Duration;
use dotenv::dotenv;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    env,
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

mod auth;
mod backup;
mod langs;
mod messages;
mod routing;
mod security_headers;
mod tests;

use backup::{BackupConfig, BackupSystem};
use messages::{remove_old_messages, Message};
use security_headers::SecurityHeaders;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect("Failed to read .env file");
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let insecure_listen_addr = env::var("LISTEN_HTTP").expect("LISTEN_HTTP must be set");
    let secure_listen_addr = env::var("LISTEN_HTTPS").expect("LISTEN_HTTPS must be set");
    let cert_path = env::var("TLS_CERT_PATH").expect("TLS_CERT_PATH must be set");
    let key_path = env::var("TLS_KEY_PATH").expect("TLS_KEY_PATH must be set");
    let rustls_config = load_rustls_config(&cert_path, &key_path)?;

    let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
    let background_messages_clone = messages.clone();
    let tls_messages_clone = messages.clone();
    let backup_messages_clone = messages.clone();

    if let Err(e) = configure_backup_system(backup_messages_clone.clone()).await {
        log::error!("Failed to configure backup system: {}", e);
    }

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            remove_old_messages(background_messages_clone.clone());
        }
    });

    let limiter = LimiterBuilder::new()
        .with_duration(Duration::minutes(1))
        .with_num_requests(60)
        .build();

    let tls_limiter = LimiterBuilder::new()
        .with_duration(Duration::minutes(1))
        .with_num_requests(3)
        .build();

    let insecure_app_factory = move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .wrap(SecurityHeaders)
            .wrap(RateLimiter::new(Arc::clone(&limiter)))
            .app_data(Data::new(messages.clone()))
            .configure(routing::configure_insecure_message_routes)
    };

    let secure_app_factory = move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .wrap(SecurityHeaders)
            .wrap(RateLimiter::new(Arc::clone(&tls_limiter)))
            .app_data(Data::new(tls_messages_clone.clone()))
            .service(
                scope("/admin")
                    .wrap(ApiKeyMiddleware)
                    .configure(routing::configure_secure_message_routes),
            )
    };

    let http_server = HttpServer::new(insecure_app_factory.clone())
        .bind(insecure_listen_addr)?
        .run();

    let https_server = HttpServer::new(secure_app_factory)
        .bind_rustls(&secure_listen_addr, rustls_config)?
        .run();

    futures_util::try_join!(http_server, https_server)?;

    Ok(())
}

fn load_rustls_config(cert_path: &str, key_path: &str) -> std::io::Result<ServerConfig> {
    let cert_file = &mut BufReader::new(File::open(cert_path)?);
    let cert_chain = certs(cert_file)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid cert"))?
        .into_iter()
        .map(Certificate)
        .collect();

    let key_file = &mut BufReader::new(File::open(key_path)?);
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid key"))?
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No private key found",
        ));
    }

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(config)
}

async fn configure_backup_system(
    messages: Arc<Mutex<Vec<Message>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = BackupConfig::from_env()?;
    let backup_system = BackupSystem::new(messages.clone(), config).await?;

    {
        let mut messages_guard = messages.lock().unwrap();
        if messages_guard.is_empty() {
            match backup_system.restore_latest_backup().await {
                Ok(restored_messages) => {
                    *messages_guard = restored_messages;
                    log::info!("Successfully restored messages from latest backup");
                }
                Err(e) => {
                    log::error!("Failed to restore from backup: {}", e);
                }
            }
        }
    }

    backup_system.start_backup_task().await;

    Ok(())
}
