use actix_web::{get, middleware::Logger, post, web::Data, App, HttpServer};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashSet;
use actix_web::{
    web::{Json, Path},
    HttpResponse,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
enum Langs {
    English,
    Spanish,
    French,
    Italian,
    Portuguese,
    German,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct RegistrationPayload {
    lang: Langs,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Message {
    id: Uuid,
    created: DateTime<Utc>,
    content: String,
}

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

// endpoint to get langs associated with a token
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

// endpoint to get all tokens and associated langs
#[get("/api/all_tokens")]
pub async fn get_all_tokens(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let tokens = tokens
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire lock on map"))?;

    Ok(HttpResponse::Ok().json(tokens.clone()))
}

// endpoint to add langs associated with a token
#[post("/api/add_langs/{token}")]
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

// endpoint to remove langs associated with a token
#[post("/api/remove_langs/{token}")]
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

// endpoint to unregister token
#[post("/api/unregister/{token}")]
pub async fn unregister_token(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
    token: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    // remove token from hashmap
    let mut tokens = tokens.lock().unwrap();
    tokens.remove(&token.to_string());
    Ok(HttpResponse::Ok().finish())
}

// #[post("/api/messages/en")]
// pub async fn add_en_message(
//     en_message_repo: Data<Arc<Mutex<Vec<Message>>>>,
//     body: Json<Message>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     let mut en_message_repo = en_message_repo.lock().unwrap();
//     en_message_repo.push(body.clone());
//     Ok(HttpResponse::Ok().finish())
// }

// Macro to generate add_message functions for different languages
macro_rules! create_add_message_endpoint {
    ($repo:ident, $route:expr, $fn_name:ident) => {
        #[post($route)]
        pub async fn $fn_name(
            $repo : Data<Arc<Mutex<Vec<Message>>>>,
            body: Json<Message>,
        ) -> Result<HttpResponse, actix_web::Error> {
            let mut $repo  = $repo .lock().unwrap();
            $repo .push(body.clone());
            Ok(HttpResponse::Ok().finish())
        }
    };
}

create_add_message_endpoint!(en_message_repo, "/api/messages/en", add_en_message);

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
            .service(register_token)
            .service(get_langs)
            .service(get_all_tokens)
            .service(add_langs)
            .service(remove_langs)
            .service(unregister_token)
    })
    //.bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 7273))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, App};
    use serde_json::json;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    #[actix_rt::test]
    async fn test_register_token() {
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(register_token),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/register/test_token")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let tokens = tokens.lock().unwrap();
        assert!(tokens.contains_key("test_token"));
        assert!(tokens.get("test_token").unwrap().is_empty());
    }

    #[actix_rt::test]
    async fn test_get_langs() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::English);
        map.insert("test_token".to_string(), set);
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(map));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(get_langs),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/get_langs/test_token")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let langs: HashSet<Langs> = test::read_body_json(resp).await;
        assert!(langs.contains(&Langs::English));
    }

    #[actix_rt::test]
    async fn test_add_langs() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::English);
        map.insert("test_token".to_string(), set);
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(map));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(add_langs),
        )
        .await;

        let payload = json!({ "lang": "Spanish" });
        let req = test::TestRequest::post()
            .uri("/api/add_langs/test_token")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let tokens = tokens.lock().unwrap();
        assert!(tokens.get("test_token").unwrap().contains(&Langs::Spanish));
    }

    #[actix_rt::test]
    async fn test_remove_langs() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::English);
        map.insert("test_token".to_string(), set);
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(map));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(remove_langs),
        )
        .await;

        let payload = json!({ "lang": "English" });
        let req = test::TestRequest::post()
            .uri("/api/remove_langs/test_token")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let tokens = tokens.lock().unwrap();
        assert!(!tokens.get("test_token").unwrap().contains(&Langs::English));
    }

    #[actix_rt::test]
    async fn test_unregister_token() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::English);
        map.insert("test_token".to_string(), set);
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(map));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(unregister_token),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/unregister/test_token")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let tokens = tokens.lock().unwrap();
        assert!(!tokens.contains_key("test_token"));
    }
}