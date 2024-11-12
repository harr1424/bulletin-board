#[cfg(test)]

mod tests {
    use actix_web::{http::StatusCode, test, web::Data, App};
    use chrono::Utc;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    use crate::langs::Langs;
    use crate::messages::*;



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
            expires: Expiration::Week,
            title: "Test".to_string(),
            image_url: None,
            image_data: None,
            image_mime_type: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/messages")
            .set_json(&new_message)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = messages.lock().unwrap();
        assert!(messages.iter().any(|msg| {
            msg.content == new_message.content
                && msg.lang == new_message.lang
                && msg.expires == new_message.expires
        }));
    }

    #[actix_rt::test]
    async fn test_get_messages_by_lang() {
        let message = Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, world!".to_string(),
            lang: Langs::English,
            expires: Expiration::Week,
            title: "Test".to_string(),
            image_url: None,
            image_data: None,
            image_mime_type: None,
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
            expires: Expiration::Week,
            title: "Test".to_string(),
            image_url: None,
            image_data: None,
            image_mime_type: None,
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
            title: "Test".to_string(),
            image_url: None
        };

        let req = test::TestRequest::patch()
            .uri("/api/messages")
            .set_json(&edit)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = messages.lock().unwrap();
        assert_eq!(
            messages
                .iter()
                .find(|x| x.id == message.id)
                .unwrap()
                .content,
            "Hello, Rust!"
        );
    }

    #[actix_rt::test]
    async fn test_delete_message() {
        let message = Message {
            id: Uuid::new_v4(),
            created: Utc::now(),
            content: "Hello, world!".to_string(),
            lang: Langs::English,
            expires: Expiration::Week,
            title: "Test".to_string(),
            image_url: None,
            image_data: None,
            image_mime_type: None,
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
