use actix_web::web::ServiceConfig;
use crate::messages::*;
use crate::clients::*;

pub fn configure_client_routes(cfg: &mut ServiceConfig) {
    cfg.service(register_token);
    cfg.service(get_langs);
    cfg.service(get_all_tokens);
    cfg.service(add_langs);
    cfg.service(remove_langs);
    cfg.service(unregister_token);
}

pub fn configure_message_routes(cfg: &mut ServiceConfig) {
    cfg.service(add_en_message);
    cfg.service(get_en_messages);
    cfg.service(edit_en_message);
    cfg.service(delete_en_message);

    cfg.service(add_es_message);
    cfg.service(get_es_messages);
    cfg.service(edit_es_message);
    cfg.service(delete_es_message);

    cfg.service(add_fr_message);
    cfg.service(get_fr_messages);
    cfg.service(edit_fr_message);
    cfg.service(delete_fr_message);

    cfg.service(add_it_message);
    cfg.service(get_it_messages);
    cfg.service(edit_it_message);
    cfg.service(delete_it_message);

    cfg.service(add_po_message);
    cfg.service(get_po_messages);
    cfg.service(edit_po_message);
    cfg.service(delete_po_message);

    cfg.service(add_de_message);
    cfg.service(get_de_messages);
    cfg.service(edit_de_message);
    cfg.service(delete_de_message);
}

