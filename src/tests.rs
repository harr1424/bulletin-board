#[cfg(test)]

mod tests {
    use actix_web::{
        web::{Json, Path},
        HttpResponse, delete, get, patch, post, http::StatusCode, test, web::Data, App
    };
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use std::{collections::{HashMap, HashSet},
                sync::{Arc, Mutex}};
    use uuid::Uuid;

    use crate::clients::*;
    use crate::langs::Langs;

    #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
    struct Message {
        id: Uuid,
        created: DateTime<Utc>,
        content: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
    struct EditMessage {
        id: Uuid,
        content: String,
    }

    #[actix_rt::test]
    async fn test_register_token() {
        let tokens: Arc<Mutex<HashMap<String, HashSet<Langs>>>> =
            Arc::new(Mutex::new(HashMap::new()));
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

    // Macro to generate add_message functions for different languages
    macro_rules! create_add_message_endpoint {
        ($repo:ident, $route:expr, $fn_name:ident) => {
            #[post($route)]
            pub async fn $fn_name(
                $repo: Data<Arc<Mutex<Vec<Message>>>>,
                body: Json<Message>,
            ) -> Result<HttpResponse, actix_web::Error> {
                let mut $repo = $repo.lock().unwrap();
                $repo.push(body.clone());
                Ok(HttpResponse::Ok().finish())
            }
        };
    }

    create_add_message_endpoint!(en_message_repo, "/api/messages/en", add_en_message);

    #[actix_rt::test]
    async fn test_add_en_message() {
        let en_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
        let app = test::init_service(
            App::new()
                .app_data(Data::new(en_message_repo.clone()))
                .service(add_en_message),
        )
        .await;

        let message = Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, World!".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/messages/en")
            .set_json(&message)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let repo = en_message_repo.lock().unwrap();
        assert_eq!(repo.len(), 1);
        assert_eq!(repo[0], message);
    }

    // Macro to generate get_messages functions for different languages
    macro_rules! create_get_messages_endpoint {
        ($repo:ident, $route:expr, $fn_name:ident) => {
            #[get($route)]
            pub async fn $fn_name(
                $repo: Data<Arc<Mutex<Vec<Message>>>>,
            ) -> Result<HttpResponse, actix_web::Error> {
                let $repo = $repo.lock().unwrap();
                Ok(HttpResponse::Ok().json($repo.clone()))
            }
        };
    }

    create_get_messages_endpoint!(en_message_repo, "/api/messages/en", get_en_messages);

    #[actix_rt::test]
    async fn test_get_en_messages() {
        let en_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(vec![Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, World!".to_string(),
        }]));
        let app = test::init_service(
            App::new()
                .app_data(Data::new(en_message_repo.clone()))
                .service(get_en_messages),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/messages/en")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let response_body: Vec<Message> = test::read_body_json(resp).await;
        let repo = en_message_repo.lock().unwrap();
        assert_eq!(response_body, *repo);
    }

    // Macro to generate edit_message functions for different languages
    macro_rules! create_edit_message_endpoint {
        ($repo:ident, $route:expr, $fn_name:ident) => {
            #[patch($route)]
            pub async fn $fn_name(
                $repo: Data<Arc<Mutex<Vec<Message>>>>,
                body: Json<EditMessage>,
            ) -> Result<HttpResponse, actix_web::Error> {
                let mut $repo = $repo.lock().unwrap();
                let index = $repo.iter().position(|x| x.id == body.id).unwrap();
                $repo[index].content = body.content.clone();
                Ok(HttpResponse::Ok().finish())
            }
        };
    }

    create_edit_message_endpoint!(en_message_repo, "/api/messages/en", edit_en_message);

    #[actix_rt::test]
    async fn test_edit_en_message() {
        let message_id = Uuid::new_v4();
        let en_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(vec![Message {
            id: message_id,
            created: Utc::now(),
            content: "Hello, World!".to_string(),
        }]));
        let app = test::init_service(
            App::new()
                .app_data(Data::new(en_message_repo.clone()))
                .service(edit_en_message),
        )
        .await;

        let edit_message = EditMessage {
            id: message_id,
            content: "Hello, Universe!".to_string(),
        };

        let req = test::TestRequest::patch()
            .uri("/api/messages/en")
            .set_json(&edit_message)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let repo = en_message_repo.lock().unwrap();
        assert_eq!(repo[0].content, edit_message.content);
    }

    // Macro to generate delete_message functions for different languages
    macro_rules! create_delete_message_endpoint {
        ($repo:ident, $route:expr, $fn_name:ident) => {
            #[delete($route)]
            pub async fn $fn_name(
                $repo: Data<Arc<Mutex<Vec<Message>>>>,
                id: Path<Uuid>,
            ) -> Result<HttpResponse, actix_web::Error> {
                let mut $repo = $repo.lock().unwrap();
                let index = $repo.iter().position(|x| x.id == *id).unwrap();
                $repo.remove(index);
                Ok(HttpResponse::Ok().finish())
            }
        };
    }

    create_delete_message_endpoint!(en_message_repo, "/api/messages/en/{id}", delete_en_message);

    #[actix_rt::test]
    async fn test_delete_en_message() {
        let message_id = Uuid::new_v4();
        let en_message_repo: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(vec![Message {
            id: message_id,
            created: Utc::now(),
            content: "Hello, World!".to_string(),
        }]));
        let app = test::init_service(
            App::new()
                .app_data(Data::new(en_message_repo.clone()))
                .service(delete_en_message),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/messages/en/{}", message_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let repo = en_message_repo.lock().unwrap();
        assert!(repo.is_empty());
    }
}