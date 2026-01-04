use actix_web::{HttpRequest, HttpResponse, Responder};
use serde_json::json;
use tracing::info;

use crate::backend::utils::jwt::{verify_jwt, Claims};
use crate::backend::utils::extractors::extract_token_from_request;
use crate::backend::errors::{ErrorCode, error_response};

/// 从请求中提取并验证用户信息
///
/// 这是一个辅助函数，用于从请求中提取 JWT token 并验证
/// 返回包含用户信息的 Claims
fn extract_user_from_request(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    // 使用共享的 token 提取函数
    let token = extract_token_from_request(req).map_err(|err| {
        HttpResponse::Unauthorized().json(error_response(
            ErrorCode::TokenMissing,
            err.message(),
        ))
    })?;

    // 验证 token 并解析用户信息
    verify_jwt(&token)
        .map(|data| data.claims)
        .map_err(|_| {
            HttpResponse::Unauthorized().json(error_response(
                ErrorCode::TokenInvalid,
                "Invalid or expired token",
            ))
        })
}

/// 获取当前用户信息
///
/// # JWT 认证测试接口
///
/// 此接口用于测试 JWT 认证是否正常工作。它会：
/// 1. 验证请求中的 JWT token
/// 2. 解析 token 中的用户信息
/// 3. 返回完整的用户数据
///
/// ## 请求示例
/// ```bash
/// curl -X GET http://localhost:8080/v2/user/me \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN"
/// ```
///
/// ## 成功响应 (200)
/// ```json
/// {
///   "code": 0,
///   "message": "success",
///   "path": "/v2/user/me",
///   "data": {
///     "user_id": "user123",
///     "username": "test_user",
///     "role": "Admin",
///     "exp": 1735689600
///   }
/// }
/// ```
pub async fn get_current_user(req: HttpRequest) -> impl Responder {
    info!("🔍 GET /v2/user/me - Fetching current user info");

    // 提取并验证用户信息
    match extract_user_from_request(&req) {
        Ok(claims) => {
            info!("✅ User authenticated: {} ({})", claims.username, claims.user_id);

            HttpResponse::Ok().json(json!({
                "code": 0,
                "message": "success",
                "path": "/v2/user/me",
                "data": {
                    "user_id": claims.user_id,
                    "username": claims.username,
                    "role": claims.role,
                    "exp": claims.exp
                }
            }))
        }
        Err(error_resp) => {
            info!("❌ Authentication failed");
            error_resp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use crate::backend::utils::jwt::{create_jwt, Claims};
    use crate::backend::models::sea_orm_active_enums::UserRoleType;
    use chrono::Utc;

    #[test]
    fn test_extract_user_from_valid_request() {
        // 创建测试用的 JWT claims
        let claims = Claims {
            user_id: "test_user_123".to_string(),
            username: "alice".to_string(),
            role: Some(UserRoleType::Admin),
            exp: (Utc::now().timestamp() as usize + 3600),
        };

        // 生成 token
        let token = create_jwt(&claims);

        // 创建测试请求
        let req = test::TestRequest::default()
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_http_request();

        // 测试提取用户信息
        let extracted_claims = extract_user_from_request(&req);
        assert!(extracted_claims.is_ok());

        let extracted = extracted_claims.unwrap();
        assert_eq!(extracted.user_id, claims.user_id);
        assert_eq!(extracted.username, claims.username);
        assert_eq!(extracted.role, claims.role);
    }

    #[test]
    fn test_extract_user_from_missing_token() {
        // 创建没有 token 的请求
        let req = test::TestRequest::default()
            .to_http_request();

        let result = extract_user_from_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_user_from_invalid_token() {
        // 创建无效 token 的请求
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "Bearer invalid_token_12345"))
            .to_http_request();

        let result = extract_user_from_request(&req);
        assert!(result.is_err());
    }
}
