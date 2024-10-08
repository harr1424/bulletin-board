use actix_web::web::ServiceConfig;
use crate::messages::*;
use crate::clients::*;

pub fn configure_client_routes(cfg: &mut ServiceConfig) {
    cfg.service(register_token);
    cfg.service(get_langs);         // TODO unregster this once DynamoDB is set up
    cfg.service(get_all_tokens);  // TODO unregster this once DynamoDB is set up
    cfg.service(add_langs);
    cfg.service(remove_langs);
    cfg.service(unregister_token);
}

pub fn configure_secure_message_routes(cfg: &mut ServiceConfig) {
    cfg.service(add_message);
    cfg.service(edit_message);
    cfg.service(delete_message);
}

pub fn configure_insecure_message_routes(cfg: &mut ServiceConfig) {
    cfg.service(get_messages_by_lang);
}
