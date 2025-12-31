use actix_web::{web, HttpResponse};
use serde::Deserialize;
use chrono::Utc;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use tracing::info;
use crate::backend::AppState;
use crate::backend::api::qr_login::handle_qr_session::{find_session_by_id, update_session_confirmed};
use crate::backend::models::users;
use crate::backend::utils::jwt::{Claims, create_jwt, verify_jwt};
use crate::backend::ws_manager::WsManager;
use crate::backend::errors::{ErrorCode, error_response, SuccessResponse};

#[derive(Deserialize, Debug)]
pub struct ConfirmLoginRequest {
    pub session_id: String,
    pub app_token: String,
}

pub async fn confirm_login(
    state: web::Data<AppState>,
    ws_manager: web::Data<WsManager>,
    request: web::Json<ConfirmLoginRequest>,
) -> HttpResponse {
    info!("Received confirm login request for session: {}", request.session_id);
    
    // 1. 验证App端token
    let claims = match verify_jwt(&request.app_token) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            let error_resp = error_response(
                ErrorCode::TokenInvalid,
                format!("Invalid app token: {}", e),
            );
            return HttpResponse::Unauthorized().json(error_resp);
        }
    };

    // 2. 查找会话
    let session = match find_session_by_id(&state.pg_client, &request.session_id).await {
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

    // 3. 检查会话状态
    if session.status != "pending" && session.status != "scanned" {
        let error_resp = error_response(
            ErrorCode::ResourceConflict,
            "Session is not in valid state",
        );
        return HttpResponse::BadRequest().json(error_resp);
    }

    // 4. 检查是否过期
    let now = Utc::now().naive_utc();
    if session.expires_at < now {
        let error_resp = error_response(
            ErrorCode::QRCodeExpired,
            "Session expired",
        );
        return HttpResponse::BadRequest().json(error_resp);
    }
    
    // 5. 验证用户是否存在
    let user = match users::Entity::find()
        .filter(users::Column::UserId.eq(&claims.user_id))
        .one(&state.pg_client)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            let error_resp = error_response(
                ErrorCode::NotFound,
                "User not found",
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

    // 6. 生成Web端JWT token
    let web_claims = Claims {
        user_id: user.user_id.clone(),
        username: user.user_id.clone(),
        role: Some(user.role.clone()),
        exp: (Utc::now().timestamp() as usize + 60 * 60 * 24), // 24小时
    };
    let web_token = create_jwt(&web_claims);

    // 7. 更新会话状态
    if let Err(e) = update_session_confirmed(
        &state.pg_client,
        &request.session_id,
        &user.user_id,
        &web_token,
        &request.app_token,
    ).await {
        let error_resp = error_response(
            ErrorCode::DatabaseError,
            format!("Failed to update session: {}", e),
        );
        return HttpResponse::InternalServerError().json(error_resp);
    }
    
    // 8. 🔔 通过WebSocket推送状态更新
    ws_manager.notify_status(&request.session_id, "confirmed", Some(&web_token)).await;
    info!("✅ Login confirmed and WebSocket notified for session: {}", request.session_id);

    #[derive(serde::Serialize)]
    struct ConfirmResponse {
        success: bool,
        message: String,
    }

    HttpResponse::Ok().json(SuccessResponse::new(ConfirmResponse {
        success: true,
        message: "Login confirmed successfully".to_string(),
    }))
}
