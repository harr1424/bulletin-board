#[cfg(test)]

mod tests {
    use actix_web::{http::StatusCode, test, web::Data, App};
    use chrono::Utc;
    use serde_json::json;
    use std::{
        collections::{HashMap, HashSet},
        sync::{Arc, Mutex},
    };
    use uuid::Uuid;

    use crate::clients::*;
    use crate::langs::Langs;
    use crate::messages::*;

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
        let req = test::TestRequest::patch()
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
        let req = test::TestRequest::patch()
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

        let req = test::TestRequest::delete()
            .uri("/api/unregister/test_token")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let tokens = tokens.lock().unwrap();
        assert!(!tokens.contains_key("test_token"));
    }

    #[actix_rt::test]
    async fn test_add_message() {
        let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(messages.clone()))
                .service(add_message),
        )
        .await;

        let new_message = Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, world!".to_string(),
            lang: Langs::English,
        };

        let req = test::TestRequest::post()
            .uri("/api/messages")
            .set_json(&new_message)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = messages.lock().unwrap();
        assert!(messages.contains(&new_message));
    }

    #[actix_rt::test]
    async fn test_get_messages_by_lang() {
        let message = Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, world!".to_string(),
            lang: Langs::English,
        };
        let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(vec![message.clone()]));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(messages.clone()))
                .service(get_messages_by_lang),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/messages/English")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let returned_messages: Vec<Message> = test::read_body_json(resp).await;
        assert!(returned_messages.contains(&message));
    }

    #[actix_rt::test]
    async fn test_edit_message() {
        let message = Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, world!".to_string(),
            lang: Langs::English,
        };
        let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(vec![message.clone()]));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(messages.clone()))
                .service(edit_message),
        )
        .await;

        let edit = EditMessage {
            id: message.id,
            content: "Hello, Rust!".to_string(),
        };

        let req = test::TestRequest::patch()
            .uri("/api/messages")
            .set_json(&edit)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = messages.lock().unwrap();
        assert_eq!(messages.iter().find(|x| x.id == message.id).unwrap().content, "Hello, Rust!");
    }

    #[actix_rt::test]
    async fn test_delete_message() {
        let message = Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, world!".to_string(),
            lang: Langs::English,
        };
        let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(vec![message.clone()]));
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(messages.clone()))
                .service(delete_message),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/messages/{}", message.id))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = messages.lock().unwrap();
        assert!(messages.iter().find(|x| x.id == message.id).is_none());
    }
}
