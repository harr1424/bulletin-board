use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpResponse,
};
use chrono::{DateTime, Duration, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    id: Uuid,
    created: DateTime<Utc>,
    content: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
struct EditMessage {
    id: Uuid,
    content: String,
}
// Macro to generate add_message functions for different languages
macro_rules! create_add_message_endpoint {
    ($repo:ident, $route:expr, $fn_name:ident) => {
        #[post($route)]
        pub async fn $fn_name(
            $repo: Data<Arc<Mutex<Vec<Message>>>>,
            body: Json<String>,
        ) -> Result<HttpResponse, actix_web::Error> {
            let mut $repo = $repo.lock().map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
            })?;
            let new_message = Message {
                id: Uuid::new_v4(),
                created: Utc::now(),
                content: body.clone(),
            };
            $repo.push(new_message);
            Ok(HttpResponse::Ok().finish())
        }
    };
}

create_add_message_endpoint!(en_message_repo, "/api/messages/en", add_en_message);
create_add_message_endpoint!(es_message_repo, "/api/messages/es", add_es_message);
create_add_message_endpoint!(fr_message_repo, "/api/messages/fr", add_fr_message);
create_add_message_endpoint!(it_message_repo, "/api/messages/it", add_it_message);
create_add_message_endpoint!(po_message_repo, "/api/messages/po", add_po_message);
create_add_message_endpoint!(de_message_repo, "/api/messages/de", add_de_message);

// Macro to generate get_messages functions for different languages
macro_rules! create_get_messages_endpoint {
    ($repo:ident, $route:expr, $fn_name:ident) => {
        #[get($route)]
        pub async fn $fn_name(
            $repo: Data<Arc<Mutex<Vec<Message>>>>,
        ) -> Result<HttpResponse, actix_web::Error> {
            let $repo = $repo.lock().map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
            })?;
            Ok(HttpResponse::Ok().json($repo.clone()))
        }
    };
}

create_get_messages_endpoint!(en_message_repo, "/api/messages/en", get_en_messages);
create_get_messages_endpoint!(es_message_repo, "/api/messages/es", get_es_messages);
create_get_messages_endpoint!(fr_message_repo, "/api/messages/fr", get_fr_messages);
create_get_messages_endpoint!(it_message_repo, "/api/messages/it", get_it_messages);
create_get_messages_endpoint!(po_message_repo, "/api/messages/po", get_po_messages);
create_get_messages_endpoint!(de_message_repo, "/api/messages/de", get_de_messages);

// Macro to generate edit_message functions for different languages
macro_rules! create_edit_message_endpoint {
    ($repo:ident, $route:expr, $fn_name:ident) => {
        #[patch($route)]
        pub async fn $fn_name(
            $repo: Data<Arc<Mutex<Vec<Message>>>>,
            body: Json<EditMessage>,
        ) -> Result<HttpResponse, actix_web::Error> {
            let mut $repo = $repo.lock().map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
            })?;
            let index = $repo.iter().position(|x| x.id == body.id).unwrap();
            $repo[index].content = body.content.clone();
            Ok(HttpResponse::Ok().finish())
        }
    };
}

create_edit_message_endpoint!(en_message_repo, "/api/messages/en", edit_en_message);
create_edit_message_endpoint!(es_message_repo, "/api/messages/es", edit_es_message);
create_edit_message_endpoint!(fr_message_repo, "/api/messages/fr", edit_fr_message);
create_edit_message_endpoint!(it_message_repo, "/api/messages/it", edit_it_message);
create_edit_message_endpoint!(po_message_repo, "/api/messages/po", edit_po_message);
create_edit_message_endpoint!(de_message_repo, "/api/messages/de", edit_de_message);

// Macro to generate delete_message functions for different languages
macro_rules! create_delete_message_endpoint {
    ($repo:ident, $route:expr, $fn_name:ident) => {
        #[delete($route)]
        pub async fn $fn_name(
            $repo: Data<Arc<Mutex<Vec<Message>>>>,
            id: Path<Uuid>,
        ) -> Result<HttpResponse, actix_web::Error> {
            let mut $repo = $repo.lock().map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to acquire lock on message repo")
            })?;
            
            if let Some(index) = $repo.iter().position(|x| x.id == *id) {
                $repo.remove(index);
                Ok(HttpResponse::Ok().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
    };
}

create_delete_message_endpoint!(en_message_repo, "/api/messages/en/{id}", delete_en_message);
create_delete_message_endpoint!(es_message_repo, "/api/messages/es/{id}", delete_es_message);
create_delete_message_endpoint!(fr_message_repo, "/api/messages/fr/{id}", delete_fr_message);
create_delete_message_endpoint!(it_message_repo, "/api/messages/it/{id}", delete_it_message);
create_delete_message_endpoint!(po_message_repo, "/api/messages/po/{id}", delete_po_message);
create_delete_message_endpoint!(de_message_repo, "/api/messages/de/{id}", delete_de_message);

// Function to iterate over a Arc<Mutex<Vec<Message>>> and remove any messages exceeding a certain age
pub fn remove_old_messages(repo: Arc<Mutex<Vec<Message>>>, max_age: Duration) {
    let mut repo = repo.lock().unwrap();
    let now = Utc::now();
    repo.retain(|msg| now.signed_duration_since(msg.created) < max_age);
}
