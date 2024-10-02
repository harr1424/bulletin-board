use actix_web::{get, middleware::Logger, post, web::Data, App, HttpServer};
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
    En,
    Es,
    Fr,
    It,
    Po,
    De,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct RegistrationPayload {
    lang: Langs,
}

#[post("/api/register/{token}")]
pub async fn register_token(
    tokens: Data<Arc<Mutex<HashMap<String, HashSet<Langs>>>>>,
    token: Path<String>,
    body: Json<RegistrationPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut tokens = tokens
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to acquire lock on map"))?;

    let entry = tokens.entry(token.to_string()).or_insert_with(HashSet::new);
    entry.insert(body.lang.clone());

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(Data::new(tokens.clone()))
            .service(register_token)
            .service(get_langs)
            .service(add_langs)
            .service(remove_langs)
            .service(unregister_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, App};
    use serde_json::json;

    #[actix_rt::test]
    async fn test_register_token() {
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(register_token),
        )
        .await;

        let payload = json!({ "lang": "En" });
        let req = test::TestRequest::post()
            .uri("/api/register/test_token")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let tokens = tokens.lock().unwrap();
        assert!(tokens.contains_key("test_token"));
        assert!(tokens.get("test_token").unwrap().contains(&Langs::En));
    }

    #[actix_rt::test]
    async fn test_get_langs() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::En);
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
        assert!(langs.contains(&Langs::En));
    }

    #[actix_rt::test]
    async fn test_add_langs() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::En);
        map.insert("test_token".to_string(), set);
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(map));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(add_langs),
        )
        .await;

        let payload = json!({ "lang": "Es" });
        let req = test::TestRequest::post()
            .uri("/api/add_langs/test_token")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let tokens = tokens.lock().unwrap();
        assert!(tokens.get("test_token").unwrap().contains(&Langs::Es));
    }

    #[actix_rt::test]
    async fn test_remove_langs() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::En);
        map.insert("test_token".to_string(), set);
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> = Arc::new(Mutex::new(map));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(tokens.clone()))
                .service(remove_langs),
        )
        .await;

        let payload = json!({ "lang": "En" });
        let req = test::TestRequest::post()
            .uri("/api/remove_langs/test_token")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let tokens = tokens.lock().unwrap();
        assert!(!tokens.get("test_token").unwrap().contains(&Langs::En));
    }

    #[actix_rt::test]
    async fn test_unregister_token() {
        let mut map = HashMap::new();
        let mut set = HashSet::new();
        set.insert(Langs::En);
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