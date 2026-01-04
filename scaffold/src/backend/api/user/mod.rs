mod get_me;
mod generate_test_token;

use actix_web::{Scope, web};

use crate::backend::api::user::get_me::get_current_user;
use crate::backend::api::user::generate_test_token::{generate_test_token, generate_default_test_token};

pub fn user_scope() -> Scope {
    web::scope("/user")
        .route("/me", web::get().to(get_current_user))
}

pub fn test_scope() -> Scope {
    web::scope("/test")
        .route("/generate-token", web::post().to(generate_test_token))
        .route("/generate-token/default", web::post().to(generate_default_test_token))
}
