use actix_web::{web, HttpResponse};
use chrono::Utc;
use tracing::info;
use serde_json::json;
use crate::backend::AppState;
use crate::backend::api::qr_login::handle_qr_session::find_session_by_id;
use crate::backend::errors::{ErrorCode, error_response};

pub async fn check_login_status(
    state: web::Data<AppState>,
    session_id: web::Path<String>,
) -> HttpResponse {
    info!("Checking login status for session: {}", session_id);
    
    // 查找会话
    let session = match find_session_by_id(&state.pg_client, &session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            let error_resp = error_response(
                ErrorCode::QRCodeNotFound,
                "Session not found",
            );
            return HttpResponse::NotFound().json(error_resp);
        }
        Err(e) => {
            let error_resp = error_response(
                ErrorCode::DatabaseError,
                format!("Database error: {}", e),
            );
            return HttpResponse::InternalServerError().json(error_resp);
        }
    };
    
    // 检查是否过期
    let now = Utc::now().naive_utc();
    if session.expires_at < now && session.status == "pending" {
        return HttpResponse::Ok().json(json!({
            "status": "expired",
            "web_token": null,
            "message": "QR code expired"
        }));
    }

    // 根据状态返回 - 使用 serde_json 防止 XSS 和注入攻击
    match session.status.as_str() {
        "pending" => {
            HttpResponse::Ok().json(json!({
                "status": "pending",
                "web_token": null,
                "message": "Waiting for scan"
            }))
        }
        "scanned" => {
            HttpResponse::Ok().json(json!({
                "status": "scanned",
                "web_token": null,
                "message": "Scanned, waiting for confirmation"
            }))
        }
        "confirmed" => {
            let web_token = session.web_token.unwrap_or_default();
            HttpResponse::Ok().json(json!({
                "status": "confirmed",
                "web_token": web_token,
                "message": "Login successful"
            }))
        }
        "rejected" => {
            HttpResponse::Ok().json(json!({
                "status": "rejected",
                "web_token": null,
                "message": "Login rejected by user"
            }))
        }
        _ => {
            HttpResponse::Ok().json(json!({
                "status": "expired",
                "web_token": null,
                "message": "QR code expired"
            }))
        }
    }
}
