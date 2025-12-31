use actix_web::{HttpResponse, web};
use serde::Deserialize;
use tracing::info;
use crate::backend::AppState;
use crate::backend::errors::SuccessResponse;

#[derive(Deserialize, Debug)]
pub struct PhoneRequest {
    #[allow(dead_code)]
    pub phone: String,
}

pub async fn send_phone_code(
    _state: web::Data<AppState>,
    request: web::Json<PhoneRequest>,
) -> HttpResponse {
    info!("Received email request: {:?}", request);
    println!("Received email request: {:?}", request);

    #[derive(serde::Serialize)]
    struct PhoneResponse {
        message: String,
    }

    HttpResponse::Ok().json(SuccessResponse::new(PhoneResponse {
        message: "phone code sent successfully".to_string(),
    }))
}