use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::backend::utils::jwt::{create_jwt, Claims};
use crate::backend::models::sea_orm_active_enums::UserRoleType;
use crate::backend::errors::SuccessResponse;
use chrono::Utc;

/// 生成测试 Token 的请求参数
#[derive(Debug, Deserialize)]
pub struct GenerateTokenRequest {
    pub user_id: String,
    pub username: String,
    #[serde(default)]
    pub role: Option<String>,
}

/// 生成测试 Token 的响应
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub expires_at: String,
}

/// 生成测试 JWT Token
///
/// # 测试接口
///
/// 此接口用于生成测试用的 JWT token，方便开发和测试。
///
/// **注意**: 此接口仅用于开发测试环境，生产环境中应该禁用。
///
/// ## 请求示例
/// ```bash
/// curl -X POST http://localhost:8080/v1/test/generate-token \
///   -H "Content-Type: application/json" \
///   -d '{
///     "user_id": "test_user_001",
///     "username": "alice",
///     "role": "Admin"
///   }'
/// ```
///
/// ## 成功响应 (200)
/// ```json
/// {
///   "code": 0,
///   "message": "success",
///   "data": {
///     "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9...",
///     "user_id": "test_user_001",
///     "username": "alice",
///     "role": "Admin",
///     "expires_at": "2025-01-05T12:00:00Z"
///   }
/// }
/// ```
pub async fn generate_test_token(
    req: HttpRequest,
    params: web::Json<GenerateTokenRequest>,
) -> impl Responder {
    info!("🧪 生成测试 Token: user_id={}, username={}", params.user_id, params.username);

    // 解析角色
    let role = match params.role.as_deref() {
        Some("Admin") | Some("admin") => Some(UserRoleType::Admin),
        Some("User") | Some("user") => Some(UserRoleType::User),
        _ => Some(UserRoleType::User), // 默认为 User
    };

    // 计算过期时间（24小时后）
    let expires_at = Utc::now() + chrono::Duration::hours(24);
    let exp_timestamp = expires_at.timestamp() as usize;

    // 创建 Claims
    let claims = Claims {
        user_id: params.user_id.clone(),
        username: params.username.clone(),
        role: role.clone(),
        exp: exp_timestamp,
    };

    // 生成 JWT token
    let token = create_jwt(&claims);

    info!("✅ Token 生成成功: {}...", &token[..50]);

    let response = TokenResponse {
        token: token.clone(),
        user_id: params.user_id.clone(),
        username: params.username.clone(),
        role: format!("{:?}", role.unwrap_or(UserRoleType::User)),
        expires_at: expires_at.to_rfc3339(),
    };

    HttpResponse::Ok().json(SuccessResponse::new(response))
}

/// 生成默认测试 Token（快速测试）
///
/// 这是一个便捷接口，使用默认值生成测试 token。
///
/// ## 请求示例
/// ```bash
/// curl -X POST http://localhost:8080/v1/test/generate-token/default
/// ```
///
/// ## 响应
/// 与 `generate_test_token` 相同
pub async fn generate_default_test_token() -> impl Responder {
    info!("🧪 生成默认测试 Token");

    let claims = Claims {
        user_id: "test_user_001".to_string(),
        username: "alice".to_string(),
        role: Some(UserRoleType::Admin),
        exp: (Utc::now().timestamp() as usize + 86400), // 24小时后过期
    };

    let token = create_jwt(&claims);

    info!("✅ 默认 Token 生成成功");

    let response = TokenResponse {
        token,
        user_id: "test_user_001".to_string(),
        username: "alice".to_string(),
        role: "Admin".to_string(),
        expires_at: (Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
    };

    HttpResponse::Ok().json(SuccessResponse::new(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_generate_default_token() {
        let req = test::TestRequest::default().to_http_request();
        let response = generate_default_test_token().await;

        // 基本检查
        match response.respond_to(&req) {
            Ok(resp) => {
                assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
            }
            Err(e) => panic!("Response error: {:?}", e),
        }
    }
}
