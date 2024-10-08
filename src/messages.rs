use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpResponse,
};
use chrono::{DateTime, Duration, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::langs::Langs;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub id: Uuid,
    pub created: DateTime<Utc>,
    pub content: String,
    pub lang: Langs
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct EditMessage {
    pub id: Uuid,
    pub content: String,
}

// Endpoint to post a new message to the shared message repo
#[post("/api/messages")]
pub async fn add_message(
    repo: Data<Arc<Mutex<Vec<Message>>>>,
    body: Json<Message>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut repo = repo.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
    })?;
    repo.push(body.clone());
    Ok(HttpResponse::Ok().finish())
}

// Endpoint to get a message by language
#[get("/api/messages/{lang}")]
pub async fn get_messages_by_lang(
    repo: Data<Arc<Mutex<Vec<Message>>>>,
    lang: Path<Langs>,
) -> Result<HttpResponse, actix_web::Error> {
    let repo = repo.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
    })?;
    let messages: Vec<Message> = repo
        .iter()
        .filter(|x| x.lang == *lang)
        .cloned()
        .collect();
    Ok(HttpResponse::Ok().json(messages))
}

// Endpoint to edit a message
#[patch("/api/messages")]
pub async fn edit_message(
    repo: Data<Arc<Mutex<Vec<Message>>>>,
    body: Json<EditMessage>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut repo = repo.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
    })?;

    if let Some(index) = repo.iter().position(|x| x.id == body.id) {
        repo[index].content = body.content.clone();
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

// Endpoint to delete a message by id
#[delete("/api/messages/{id}")]
pub async fn delete_message(
    repo: Data<Arc<Mutex<Vec<Message>>>>,
    id: Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut repo = repo.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
    })?;

    if let Some(index) = repo.iter().position(|x| x.id == *id) {
        repo.remove(index);
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
// Function to iterate over a Arc<Mutex<Vec<Message>>> and remove any messages exceeding a certain age
pub fn remove_old_messages(repo: Arc<Mutex<Vec<Message>>>, max_age: Duration) {
    let mut repo = repo.lock().unwrap();
    let now = Utc::now();
    repo.retain(|msg| now.signed_duration_since(msg.created) < max_age);
}
