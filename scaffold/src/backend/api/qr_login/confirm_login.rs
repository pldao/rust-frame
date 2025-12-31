use actix_web::{web, HttpResponse};
use serde::Deserialize;
use chrono::Utc;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use tracing::{info, warn};
use crate::backend::AppState;
use crate::backend::api::qr_login::handle_qr_session::{find_session_by_id, update_session_confirmed};
use crate::backend::models::users;
use crate::backend::utils::jwt::{Claims, create_jwt, verify_jwt};
use crate::backend::ws_manager::WsManager;
use crate::backend::errors::{ErrorCode, error_response, SuccessResponse};
use bcrypt::{hash, DEFAULT_COST};
use crate::backend::models::sea_orm_active_enums::UserRoleType;

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

    // 1. 验证App端token并检查admin权限
    let admin_claims = match verify_jwt(&request.app_token) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            let error_resp = error_response(
                ErrorCode::TokenInvalid,
                format!("Invalid app token: {}", e),
            );
            return HttpResponse::Unauthorized().json(error_resp);
        }
    };

    // 验证是否为admin权限
    let is_admin = admin_claims.role.as_ref().map_or(false, |r| r == &UserRoleType::Admin);
    if !is_admin {
        let error_resp = error_response(
            ErrorCode::PermissionDenied,
            "Admin permission required to confirm QR login",
        );
        return HttpResponse::Forbidden().json(error_resp);
    }

    info!("Admin user {} is confirming QR login", admin_claims.user_id);

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

    // 5. 从二维码数据中提取或生成用户ID
    // 二维码数据格式: {"session_id":"xxx","action":"login","user_id":"xxx"}（可选）
    // 如果没有user_id，则基于session_id生成
    let user_id = session.user_id.clone().unwrap_or_else(|| {
        // 使用session_id作为基础生成user_id
        format!("qr_user_{}", &session.session_id[0..8])
    });

    info!("Target user_id for login: {}", user_id);

    // 6. 查找或创建用户（扫码即注册）
    let user = match users::Entity::find()
        .filter(users::Column::UserId.eq(&user_id))
        .one(&state.pg_client)
        .await
    {
        Ok(Some(u)) => {
            info!("User {} exists, proceeding with login", user_id);
            u
        }
        Ok(None) => {
            // 用户不存在，自动创建（扫码即注册）
            info!("User {} does not exist, creating new user (scan-to-register)", user_id);

            // 生成默认密码：用户名@123456
            let default_password = format!("{}@123456", user_id);
            let password_hash = match hash(&default_password, DEFAULT_COST) {
                Ok(hash) => hash,
                Err(e) => {
                    let error_resp = error_response(
                        ErrorCode::InternalError,
                        format!("Failed to hash password: {}", e),
                    );
                    return HttpResponse::InternalServerError().json(error_resp);
                }
            };

            // 创建新用户
            let new_user = users::ActiveModel {
                user_id: Set(user_id.clone()),
                password_hash: Set(password_hash),
                email: Set(Some(format!("{}@qr-login.local", user_id))),
                role: Set(UserRoleType::User),
                is_active: Set(Some(true)),
                is_verified: Set(Some(false)),
                created_at: Set(Utc::now().naive_utc()),
                updated_at: Set(Utc::now().naive_utc()),
                ..Default::default()
            };

            match new_user.insert(&state.pg_client).await {
                Ok(user) => {
                    info!("✅ New user created successfully: {}", user_id);
                    user
                }
                Err(e) => {
                    let error_resp = error_response(
                        ErrorCode::DatabaseError,
                        format!("Failed to create user: {}", e),
                    );
                    return HttpResponse::InternalServerError().json(error_resp);
                }
            }
        }
        Err(e) => {
            let error_resp = error_response(
                ErrorCode::DatabaseError,
                format!("Database error: {}", e),
            );
            return HttpResponse::InternalServerError().json(error_resp);
        }
    };

    // 7. 生成Web端JWT token
    let web_claims = Claims {
        user_id: user.user_id.clone(),
        username: user.user_id.clone(),
        role: Some(user.role.clone()),
        exp: (Utc::now().timestamp() as usize + 60 * 60 * 24), // 24小时
    };
    let web_token = create_jwt(&web_claims);

    // 8. 更新会话状态
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

    // 9. 🔔 通过WebSocket推送状态更新
    ws_manager.notify_status(&request.session_id, "confirmed", Some(&web_token)).await;
    info!("✅ Login confirmed and WebSocket notified for session: {}", request.session_id);
    info!("✅ User {} logged in via QR code scan", user.user_id);

    #[derive(serde::Serialize)]
    struct ConfirmResponse {
        success: bool,
        message: String,
        user_id: String,
        auto_registered: bool,
    }

    // 判断是否为自动注册的用户
    let was_auto_registered = session.user_id.is_none();

    HttpResponse::Ok().json(SuccessResponse::new(ConfirmResponse {
        success: true,
        message: "Login confirmed successfully".to_string(),
        user_id: user.user_id.clone(),
        auto_registered: was_auto_registered,
    }))
}
