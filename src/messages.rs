use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpResponse,
};
use chrono::{DateTime, Duration, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::langs::Langs;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Expiration {
    Hour = 60 * 60,
    Day = 60 * 60 * 24,
    Week = 60 * 60 * 24 * 7,
    Quarter = 60 * 60 * 24 * 7 * 12,
    Year = 60 * 60 * 24 * 365,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub id: Uuid,
    pub created: DateTime<Utc>,
    pub content: String,
    pub lang: Langs,
    pub expires: Expiration,
    pub title: String,
    pub image_url: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub image_mime_type: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct NewMessage {
    pub content: String,
    pub lang: Langs,
    pub expires: Expiration,
    pub title: String,
    pub image_url: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub image_mime_type: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct EditMessage {
    pub id: Uuid,
    pub content: String,
    pub title: String,
    pub image_url: Option<String>,
}

#[post("/api/messages")]
pub async fn add_message(
    repo: Data<Arc<Mutex<Vec<Message>>>>,
    body: Json<NewMessage>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut repo = repo.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
    })?;

    let new_message = Message {
        id: Uuid::new_v4(),
        created: Utc::now(),
        content: body.content.clone(),
        lang: body.lang.clone(),
        expires: body.expires.clone(),
        title: body.title.clone(),
        image_url: body.image_url.clone(),
        image_data: body.image_data.clone(),
        image_mime_type: body.image_mime_type.clone(),
    };
    repo.push(new_message);
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/messages/{lang}")]
pub async fn get_messages_by_lang(
    repo: Data<Arc<Mutex<Vec<Message>>>>,
    lang: Path<Langs>,
) -> Result<HttpResponse, actix_web::Error> {
    let repo = repo.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
    })?;
    let messages: Vec<Message> = repo.iter().filter(|x| x.lang == *lang).cloned().collect();
    Ok(HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .json(messages))
}

#[patch("/api/messages")]
pub async fn edit_message(
    repo: Data<Arc<Mutex<Vec<Message>>>>,
    body: Json<EditMessage>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut repo = repo.lock().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
    })?;

    if let Some(index) = repo.iter().position(|x| x.id == body.id) {
        repo[index].title = body.title.clone();
        repo[index].content = body.content.clone();
        repo[index].image_url = body.image_url.clone();
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

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
pub fn remove_old_messages(repo: Arc<Mutex<Vec<Message>>>) {
    let mut repo = repo.lock().unwrap();
    let now = Utc::now();
    repo.retain(|msg| {
        now.signed_duration_since(msg.created) < Duration::seconds(msg.expires as i64)
    });
}
