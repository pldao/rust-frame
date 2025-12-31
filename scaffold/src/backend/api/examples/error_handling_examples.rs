//! 错误处理示例
//!
//! 本文件展示了如何在 API 处理器中使用统一的错误码系统

use actix_web::{web, HttpResponse};
use crate::backend::AppState;
use crate::backend::errors::{
    ErrorCode, error_response, error_response_with_path,
    SuccessResponse, AppError, PaginatedData, paginated_response
};

// ============================================================================
// 基本错误处理示例
// ============================================================================

/// 示例1: 返回简单的错误响应
pub async fn example_simple_error() -> HttpResponse {
    let error_resp = error_response(
        ErrorCode::Unauthorized,
        "用户未登录"
    );
    HttpResponse::Unauthorized().json(error_resp)
}

/// 示例2: 返回带路径信息的错误响应
pub async fn example_error_with_path(path: String) -> HttpResponse {
    let error_resp = error_response_with_path(
        ErrorCode::TokenInvalid,
        "无效的token",
        path
    );
    HttpResponse::Unauthorized().json(error_resp)
}

/// 示例3: 返回成功响应
pub async fn example_success_response() -> HttpResponse {
    #[derive(serde::Serialize)]
    struct UserData {
        id: String,
        name: String,
    }

    let data = UserData {
        id: "123".to_string(),
        name: "张三".to_string(),
    };

    HttpResponse::Ok().json(SuccessResponse::new(data))
}

// ============================================================================
// 实际API场景示例
// ============================================================================

/// 示例4: 处理数据库错误
pub async fn example_database_operation(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
) -> HttpResponse {
    // 查询用户（可能返回数据库错误）
    let result = some_database_operation(&state.pg_client, &user_id).await;

    match result {
        Ok(user) => {
            HttpResponse::Ok().json(SuccessResponse::new(user))
        }
        Err(AppError::Database(err)) => {
            let error_resp = error_response(
                ErrorCode::DatabaseError,
                format!("查询用户失败: {}", err)
            );
            HttpResponse::InternalServerError().json(error_resp)
        }
        Err(AppError::NotFound(msg)) => {
            let error_resp = error_response(
                ErrorCode::NotFound,
                msg
            );
            HttpResponse::NotFound().json(error_resp)
        }
        Err(_) => {
            let error_resp = error_response(
                ErrorCode::InternalError,
                "未知错误"
            );
            HttpResponse::InternalServerError().json(error_resp)
        }
    }
}

/// 示例5: 验证请求参数
pub async fn example_validate_request(
    request: web::Json<CreateUserRequest>,
) -> HttpResponse {
    // 验证用户名
    if request.username.is_empty() {
        let error_resp = error_response(
            ErrorCode::MissingRequiredField,
            "用户名不能为空"
        );
        return HttpResponse::BadRequest().json(error_resp);
    }

    // 验证邮箱格式
    if !request.email.contains('@') {
        let error_resp = error_response(
            ErrorCode::InvalidFormat,
            "邮箱格式不正确"
        );
        return HttpResponse::BadRequest().json(error_resp);
    }

    // 处理业务逻辑...
    HttpResponse::Ok().json(SuccessResponse::new("创建成功"))
}

/// 示例6: 分页响应
pub async fn example_paginated_list(
    state: web::Data<AppState>,
    query: web::Query<PaginatedQuery>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 从数据库查询（假设）
    let items = vec!["item1", "item2", "item3"];
    let total = 100;

    let response = paginated_response(items, total, page, page_size);
    HttpResponse::Ok().json(response)
}

// ============================================================================
// 二维码登录场景示例
// ============================================================================

/// 示例7: 二维码登录状态检查
pub async fn example_qr_status_check(
    state: web::Data<AppState>,
    session_id: web::Path<String>,
) -> HttpResponse {
    // 查询会话
    let session = find_qr_session(&state.pg_client, &session_id).await;

    match session {
        Ok(Some(session)) => {
            // 检查是否过期
            if session.is_expired() {
                let error_resp = error_response(
                    ErrorCode::QRCodeExpired,
                    "二维码已过期"
                );
                return HttpResponse::BadRequest().json(error_resp);
            }

            // 返回状态
            HttpResponse::Ok().json(SuccessResponse::new(session.status))
        }
        Ok(None) => {
            let error_resp = error_response(
                ErrorCode::QRCodeNotFound,
                "二维码不存在"
            );
            HttpResponse::NotFound().json(error_resp)
        }
        Err(e) => {
            let error_resp = error_response(
                ErrorCode::DatabaseError,
                format!("数据库错误: {}", e)
            );
            HttpResponse::InternalServerError().json(error_resp)
        }
    }
}

// ============================================================================
// 使用 ? 运算符的简化写法
// ============================================================================

/// 示例8: 使用 ? 运算符简化错误处理
pub async fn example_with_question_mark(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    // 使用 ? 运算符自动转换错误
    let user = find_user_by_id(&state.pg_client, &user_id).await?
        .ok_or_else(|| AppError::not_found("用户不存在"))?;

    // 返回成功响应
    Ok(HttpResponse::Ok().json(SuccessResponse::new(user)))
}

// ============================================================================
// 辅助类型和函数（模拟）
// ============================================================================

#[derive(serde::Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(serde::Deserialize)]
struct PaginatedQuery {
    page: Option<u64>,
    page_size: Option<u64>,
}

// 模拟数据库操作
async fn some_database_operation(
    _db: &sea_orm::DbConn,
    _id: &str,
) -> Result<String, AppError> {
    // 实际实现中，这里会查询数据库
    Ok("user data".to_string())
}

async fn find_user_by_id(
    _db: &sea_orm::DbConn,
    _id: &str,
) -> Result<Option<String>, AppError> {
    // 实际实现中，这里会查询数据库
    Ok(Some("user data".to_string()))
}

struct QrSession {
    status: String,
}

impl QrSession {
    fn is_expired(&self) -> bool {
        false
    }
}

async fn find_qr_session(
    _db: &sea_orm::DbConn,
    _id: &str,
) -> Result<Option<QrSession>, AppError> {
    // 实际实现中，这里会查询数据库
    Ok(Some(QrSession {
        status: "pending".to_string(),
    }))
}

// ============================================================================
// 使用扩展trait的简化写法
// ============================================================================

use crate::backend::errors::HttpResponseExt;

/// 示例9: 使用 HttpResponseExt trait 简化响应构建
pub async fn example_using_extension_trait() -> HttpResponse {
    // 直接使用扩展方法
    HttpResponse::json_error(ErrorCode::Unauthorized, "未授权")
}

/// 示例10: 使用扩展trait构建成功响应
pub async fn example_success_using_trait() -> HttpResponse {
    #[derive(serde::Serialize)]
    struct ResponseData {
        message: String,
    }

    let data = ResponseData {
        message: "操作成功".to_string(),
    };

    HttpResponse::json_success(data)
}

// ============================================================================
// 最佳实践建议
// ============================================================================

/*
1. **始终使用统一的错误码**
   - 不要使用硬编码的数字
   - 使用 ErrorCode 枚举确保类型安全

2. **提供清晰的错误消息**
   - 错误消息应该对用户友好
   - 包含足够的信息帮助调试

3. **正确使用 HTTP 状态码**
   - ErrorCode 会自动映射到正确的状态码
   - 不要手动指定错误的状态码

4. **使用 ? 运算符简化代码**
   - 对于可以返回 AppError 的函数，使用 ? 运算符
   - 让错误自动传播和转换

5. **记录错误日志**
   - 使用 tracing::error! 记录错误
   - 包含足够的上下文信息

6. **保持一致的响应格式**
   - 所有成功响应使用 SuccessResponse
   - 所有错误响应使用 error_response 系列函数
*/
