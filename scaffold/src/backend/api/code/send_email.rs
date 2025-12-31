use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sea_orm::DbConn;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use lettre::Message;
use lettre::Transport;
use crate::backend::api::code::handle_email_code::insert_email_code;
use crate::backend::AppState;
use crate::backend::errors::{ErrorCode, error_response, SuccessResponse};
use crate::config::lazy_config::get_mailer;

#[derive(Deserialize, Debug)]
pub struct EmailRequest {
    pub username: String,
    pub email: String,
}

fn generate_code(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

async fn send_email(email: &str, code: &str) -> Result<(), String> {
    let email_content = format!(
        "Hello,\n\nYour verification code is: {}\n\nThis code is valid for 10 minutes.",
        code
    );


    let email = Message::builder()
        .from("pureblackalex@163.com".parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Your Verification Code")
        .body(email_content)
        .map_err(|e| e.to_string())?;

    get_mailer().send(&email).map_err(|e| e.to_string())?;

    Ok(())
}


pub async fn send_email_code(
    state: web::Data<AppState>,
    request: web::Json<EmailRequest>,
) -> HttpResponse {
    let client: &DbConn = &state.pg_client;

    let code = generate_code(6); // 生成 6 位验证码

    // 存储验证码到 PostgreSQL
    if let Err(e) = insert_email_code(client, &request.username, &request.email, &code, 600).await {
        let error_resp = error_response(
            ErrorCode::DatabaseError,
            format!("Failed to store code: {}", e),
        );
        return HttpResponse::InternalServerError().json(error_resp);
    }

    // 发送邮件
    if let Err(e) = send_email(&request.email, &code).await {
        let error_resp = error_response(
            ErrorCode::EmailSendFailed,
            format!("Failed to send email: {}", e),
        );
        return HttpResponse::InternalServerError().json(error_resp);
    }

    #[derive(serde::Serialize)]
    struct EmailResponse {
        message: String,
    }

    HttpResponse::Ok().json(SuccessResponse::new(EmailResponse {
        message: "Email code sent successfully".to_string(),
    }))
}