use actix_web::{
    delete, patch, post,
    web::{Data, Json, Path},
    HttpResponse,
};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde::Serialize;

use crate::Langs;

#[allow(dead_code)]
#[derive(Serialize)]
struct TokenInfo {
    token: String,
    langs: Vec<Langs>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct RegistrationPayload {
    lang: Langs,
}

fn langs_to_set(langs: &[Langs]) -> AttributeValue {
    AttributeValue::Ss(langs.iter().map(|lang| lang.to_string()).collect())
}

fn set_to_langs(attr: &AttributeValue) -> Vec<Langs> {
    attr.as_ss().unwrap().iter().map(|s| Langs::from(s.clone())).collect()
}

// Endpoint to register a token
#[post("/api/register/{token}")]
pub async fn register_token(
    dynamodb_client: Data<Client>,
    token: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let request = dynamodb_client
        .put_item()
        .table_name("KoradiTokens")
        .item("Token", AttributeValue::S(token.to_string()));

    request
        .send()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to register token"))?;

    Ok(HttpResponse::Created().finish())
}

#[allow(dead_code)]
pub async fn get_langs(
    dynamodb_client: Data<Client>,
    token: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let request = dynamodb_client.get_item()
        .table_name("KoradiTokens")
        .key("Token", AttributeValue::S(token.to_string()));

    let result = request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get langs"))?;

    if let Some(item) = result.item {
        if let Some(langs_attr) = item.get("Langs") {
            let langs = set_to_langs(langs_attr);
            Ok(HttpResponse::Ok().json(langs))
        } else {
            Ok(HttpResponse::Ok().json(Vec::<Langs>::new()))
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

// pub async fn get_all_tokens(
//     dynamodb_client: Data<Client>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     let request = dynamodb_client.scan().table_name("KoradiTokens");

//     let result = request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get all tokens"))?;

//     let tokens: Vec<TokenInfo> = result
//         .items
//         .unwrap_or_default()
//         .into_iter()
//         .filter_map(|item| {
//             // Extract token
//             let token = item.get("Token").and_then(|v| v.as_s().ok())?.to_string();
//             let langs_attr = item.get("Langs")?;
//             let langs = set_to_langs(langs_attr);

//             Some(TokenInfo { token, langs })
//         })
//         .collect();

//     Ok(HttpResponse::Ok().json(tokens))
// }

// Endpoint to add langs associated with a token
#[patch("/api/add_langs/{token}")]
pub async fn add_langs(
    dynamodb_client: Data<Client>,
    token: Path<String>,
    body: Json<RegistrationPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    // Check if the token exists
    let get_request = dynamodb_client.get_item()
        .table_name("KoradiTokens")
        .key("Token", AttributeValue::S(token.to_string()));

    let get_result = get_request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to check token"))?;

    if get_result.item.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Exists add the language
    let request = dynamodb_client.update_item()
        .table_name("KoradiTokens")
        .key("Token", AttributeValue::S(token.to_string()))
        .update_expression("ADD Langs :lang")
        .expression_attribute_values(":lang", langs_to_set(&[body.lang.clone()]));

    request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to add lang"))?;

    Ok(HttpResponse::Ok().finish())
}

// Endpoint to remove langs associated with a token
#[patch("/api/remove_langs/{token}")]
pub async fn remove_langs(
    dynamodb_client: Data<Client>,
    token: Path<String>,
    body: Json<RegistrationPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    // Check if the token exists
    let get_request = dynamodb_client.get_item()
        .table_name("KoradiTokens")
        .key("Token", AttributeValue::S(token.to_string()));

    let get_result = get_request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to check token"))?;

    if let Some(item) = get_result.item {
        if let Some(langs_attr) = item.get("Langs") {
            let langs = set_to_langs(langs_attr);
            if !langs.contains(&body.lang) {
                return Ok(HttpResponse::NotFound().finish());
            }
        } else {
            return Ok(HttpResponse::NotFound().finish());
        }
    } else {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Exists remove the language
    let request = dynamodb_client.update_item()
        .table_name("KoradiTokens")
        .key("Token", AttributeValue::S(token.to_string()))
        .update_expression("DELETE Langs :lang")
        .expression_attribute_values(":lang", langs_to_set(&[body.lang.clone()]));

    request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to remove lang"))?;

    Ok(HttpResponse::Ok().finish())
}

// Endpoint to unregister token
#[delete("/api/unregister/{token}")]
pub async fn unregister_token(
    dynamodb_client: Data<Client>,
    token: Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    // Check if the token exists
    let get_request = dynamodb_client.get_item()
        .table_name("KoradiTokens")
        .key("Token", AttributeValue::S(token.to_string()));

    let get_result = get_request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to check token"))?;

    if get_result.item.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Exists delete the token
    let request = dynamodb_client.delete_item()
        .table_name("KoradiTokens")
        .key("Token", AttributeValue::S(token.to_string()));

    request.send().await.map_err(|_| actix_web::error::ErrorInternalServerError("Failed to unregister token"))?;

    Ok(HttpResponse::Ok().finish())
}
