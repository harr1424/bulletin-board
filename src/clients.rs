use actix_web::{
    web::{Json, Path, Data},
    HttpResponse, delete, get, patch, post
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::Langs;


#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct RegistrationPayload {
    lang: Langs,
}

// Endpoint to register a token
#[post("/api/register/{token}")]
pub async fn register_token(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
    token: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut tokens = tokens
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire lock on map"))?;

    let _ = tokens.entry(token.to_string()).or_insert_with(HashSet::new);

    Ok(HttpResponse::Created().finish())
}

// Endpoint to get langs associated with a token
#[get("/api/get_langs/{token}")]
pub async fn get_langs(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
    token: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let tokens = tokens
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire lock on map"))?;

    match tokens.get(&token.to_string()) {
        Some(langs) => Ok(HttpResponse::Ok().json(langs)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

// Endpoint to get all tokens and associated langs
#[get("/api/all_tokens")]
pub async fn get_all_tokens(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let tokens = tokens
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire lock on map"))?;

    Ok(HttpResponse::Ok().json(tokens.clone()))
}

// Endpoint to add langs associated with a token
#[patch("/api/add_langs/{token}")]
pub async fn add_langs(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
    token: Path<String>,
    body: Json<RegistrationPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut tokens = tokens
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire lock on map"))?;

    if !tokens.contains_key(&token.to_string()) {
        return Ok(HttpResponse::NotFound().finish());
    }

    let entry = tokens.entry(token.to_string()).or_insert_with(HashSet::new);
    entry.insert(body.lang.clone());

    Ok(HttpResponse::Ok().finish())
}

// Endpoint to remove langs associated with a token
#[patch("/api/remove_langs/{token}")]
pub async fn remove_langs(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
    token: Path<String>,
    body: Json<RegistrationPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut tokens = tokens
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire lock on map"))?;

    if !tokens.contains_key(&token.to_string()) {
        return Ok(HttpResponse::NotFound().finish());
    }

    let entry = tokens.entry(token.to_string()).or_insert_with(HashSet::new);
    entry.remove(&body.lang);

    Ok(HttpResponse::Ok().finish())
}

// Endpoint to unregister token
#[delete("/api/unregister/{token}")]
pub async fn unregister_token(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
    token: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    // remove token from hashmap
    let mut tokens = tokens.lock().unwrap();
    tokens.remove(&token.to_string());
    Ok(HttpResponse::Ok().finish())
}