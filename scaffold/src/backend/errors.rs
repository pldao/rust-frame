use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
use thiserror::Error;

/// 统一的错误码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 成功
    Success = 0,

    // 客户端错误 1000-1999
    // 认证相关 1001-1099
    Unauthorized = 1001,
    TokenMissing = 1002,
    TokenInvalid = 1003,
    TokenExpired = 1004,
    LoginFailed = 1005,
    PermissionDenied = 1006,

    // 请求相关 1100-1199
    BadRequest = 1100,
    InvalidParams = 1101,
    MissingRequiredField = 1102,
    InvalidFormat = 1103,
    RateLimitExceeded = 1104,

    // 资源相关 1200-1299
    NotFound = 1200,
    ResourceExpired = 1201,
    ResourceAlreadyExists = 1202,
    ResourceConflict = 1203,

    // 二维码登录相关 1300-1399
    QRCodeNotFound = 1300,
    QRCodeExpired = 1301,
    QRCodePending = 1302,
    QRCodeScanned = 1303,
    QRCodeRejected = 1304,

    // 邮件相关 1400-1499
    EmailSendFailed = 1400,
    EmailCodeInvalid = 1401,
    EmailCodeExpired = 1402,
    EmailRateLimitExceeded = 1403,

    // 服务器错误 2000-2999
    InternalError = 2000,
    DatabaseError = 2001,
    NetworkError = 2002,
    ConfigurationError = 2003,
    ServiceUnavailable = 2004,
}

impl ErrorCode {
    /// 获取错误码的默认消息
    pub fn default_message(&self) -> &str {
        match self {
            ErrorCode::Success => "操作成功",
            ErrorCode::Unauthorized => "未授权",
            ErrorCode::TokenMissing => "token不能为空",
            ErrorCode::TokenInvalid => "无效的token",
            ErrorCode::TokenExpired => "token已过期",
            ErrorCode::LoginFailed => "登录失败",
            ErrorCode::PermissionDenied => "权限不足",

            ErrorCode::BadRequest => "错误的请求",
            ErrorCode::InvalidParams => "无效的参数",
            ErrorCode::MissingRequiredField => "缺少必填字段",
            ErrorCode::InvalidFormat => "格式错误",
            ErrorCode::RateLimitExceeded => "请求过于频繁，请稍后再试",

            ErrorCode::NotFound => "资源不存在",
            ErrorCode::ResourceExpired => "资源已过期",
            ErrorCode::ResourceAlreadyExists => "资源已存在",
            ErrorCode::ResourceConflict => "资源冲突",

            ErrorCode::QRCodeNotFound => "二维码不存在",
            ErrorCode::QRCodeExpired => "二维码已过期",
            ErrorCode::QRCodePending => "等待扫码",
            ErrorCode::QRCodeScanned => "已扫码，等待确认",
            ErrorCode::QRCodeRejected => "用户拒绝登录",

            ErrorCode::EmailSendFailed => "邮件发送失败",
            ErrorCode::EmailCodeInvalid => "验证码错误",
            ErrorCode::EmailCodeExpired => "验证码已过期",
            ErrorCode::EmailRateLimitExceeded => "邮件发送过于频繁，请稍后再试",

            ErrorCode::InternalError => "内部服务器错误",
            ErrorCode::DatabaseError => "数据库错误",
            ErrorCode::NetworkError => "网络错误",
            ErrorCode::ConfigurationError => "配置错误",
            ErrorCode::ServiceUnavailable => "服务暂时不可用",
        }
    }

    /// 获取错误码对应的 HTTP 状态码
    pub fn http_status_code(&self) -> u16 {
        match self {
            ErrorCode::Success => 200,

            ErrorCode::Unauthorized
            | ErrorCode::TokenMissing
            | ErrorCode::TokenInvalid
            | ErrorCode::TokenExpired
            | ErrorCode::LoginFailed
            | ErrorCode::PermissionDenied => 401,

            ErrorCode::BadRequest
            | ErrorCode::InvalidParams
            | ErrorCode::MissingRequiredField
            | ErrorCode::InvalidFormat
            | ErrorCode::RateLimitExceeded
            | ErrorCode::ResourceAlreadyExists
            | ErrorCode::ResourceConflict => 400,

            ErrorCode::NotFound
            | ErrorCode::QRCodeNotFound => 404,

            ErrorCode::ResourceExpired
            | ErrorCode::QRCodeExpired
            | ErrorCode::EmailCodeExpired => 400,

            ErrorCode::QRCodePending
            | ErrorCode::QRCodeScanned
            | ErrorCode::QRCodeRejected => 200,

            ErrorCode::EmailSendFailed
            | ErrorCode::EmailCodeInvalid
            | ErrorCode::EmailRateLimitExceeded => 400,

            ErrorCode::InternalError
            | ErrorCode::DatabaseError
            | ErrorCode::NetworkError
            | ErrorCode::ConfigurationError
            | ErrorCode::ServiceUnavailable => 500,
        }
    }
}

/// 统一的 API 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.msg)
    }
}

impl std::error::Error for ErrorResponse {}

/// 应用错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("认证错误: {0}")]
    Auth(String),

    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("JWT错误: {0}")]
    Jwt(String),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("自定义错误: {code:?} - {message}")]
    Custom { code: ErrorCode, message: String },
}

impl AppError {
    /// 获取错误码
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::Auth(msg) if msg.contains("token不能为空") => ErrorCode::TokenMissing,
            AppError::Auth(msg) if msg.contains("无效的token") => ErrorCode::TokenInvalid,
            AppError::Auth(_) => ErrorCode::Unauthorized,
            AppError::Database(_) => ErrorCode::DatabaseError,
            AppError::Jwt(_) => ErrorCode::TokenInvalid,
            AppError::Validation(_) => ErrorCode::InvalidParams,
            AppError::NotFound(_) => ErrorCode::NotFound,
            AppError::Internal(_) => ErrorCode::InternalError,
            AppError::Custom { code, .. } => *code,
        }
    }

    /// 获取错误消息
    pub fn message(&self) -> String {
        match self {
            AppError::Auth(msg)
            | AppError::Jwt(msg)
            | AppError::Validation(msg)
            | AppError::NotFound(msg)
            | AppError::Internal(msg) => msg.clone(),
            AppError::Database(err) => err.to_string(),
            AppError::Custom { message, .. } => message.clone(),
        }
    }

    /// 创建认证错误
    pub fn auth(msg: impl Into<String>) -> Self {
        AppError::Auth(msg.into())
    }

    /// 创建 JWT 错误
    pub fn jwt(msg: impl Into<String>) -> Self {
        AppError::Jwt(msg.into())
    }

    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        AppError::Validation(msg.into())
    }

    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    /// 创建内部错误
    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::Internal(msg.into())
    }

    /// 创建自定义错误
    pub fn custom(code: ErrorCode, msg: impl Into<String>) -> Self {
        AppError::Custom {
            code,
            message: msg.into(),
        }
    }

    /// 创建二维码错误
    pub fn qr_not_found() -> Self {
        AppError::Custom {
            code: ErrorCode::QRCodeNotFound,
            message: ErrorCode::QRCodeNotFound.default_message().to_string(),
        }
    }

    pub fn qr_expired() -> Self {
        AppError::Custom {
            code: ErrorCode::QRCodeExpired,
            message: ErrorCode::QRCodeExpired.default_message().to_string(),
        }
    }

    /// 创建邮件发送错误
    pub fn email_send_failed(msg: impl Into<String>) -> Self {
        AppError::Custom {
            code: ErrorCode::EmailSendFailed,
            message: msg.into(),
        }
    }

    /// 转换为错误响应
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.code() as i32,
            msg: self.message(),
            path: None,
            details: None,
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let error_response = self.to_response();
        HttpResponse::build(self.status_code()).json(error_response)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        self.code()
            .http_status_code()
            .try_into()
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// 用于构建错误响应的辅助函数
pub fn error_response(code: ErrorCode, msg: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        code: code as i32,
        msg: msg.into(),
        path: None,
        details: None,
    }
}

pub fn error_response_with_path(
    code: ErrorCode,
    msg: impl Into<String>,
    path: impl Into<String>,
) -> ErrorResponse {
    ErrorResponse {
        code: code as i32,
        msg: msg.into(),
        path: Some(path.into()),
        details: None,
    }
}

/// 成功响应
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

impl<T: Serialize> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            code: ErrorCode::Success as i32,
            msg: ErrorCode::Success.default_message().to_string(),
            data,
        }
    }

    pub fn with_message(data: T, msg: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Success as i32,
            msg: msg.into(),
            data,
        }
    }
}

/// 分页响应数据结构
#[derive(Debug, Serialize)]
pub struct PaginatedData<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T: Serialize> PaginatedData<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };

        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// 分页响应类型
pub type PaginatedResponse<T> = SuccessResponse<PaginatedData<T>>;

/// 创建分页响应
pub fn paginated_response<T: Serialize>(
    items: Vec<T>,
    total: u64,
    page: u64,
    page_size: u64,
) -> PaginatedResponse<T> {
    SuccessResponse::new(PaginatedData::new(items, total, page, page_size))
}

/// 批量操作响应
#[derive(Debug, Serialize)]
pub struct BatchOperationResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub total_count: usize,
    pub errors: Vec<String>,
}

impl BatchOperationResult {
    pub fn new(total_count: usize) -> Self {
        Self {
            success_count: 0,
            failed_count: 0,
            total_count,
            errors: Vec::new(),
        }
    }

    pub fn add_success(&mut self) {
        self.success_count += 1;
    }

    pub fn add_failure(&mut self, error: String) {
        self.failed_count += 1;
        self.errors.push(error);
    }

    pub fn is_complete(&self) -> bool {
        self.success_count + self.failed_count == self.total_count
    }
}

/// HTTP 响应扩展trait，提供便捷的响应构建方法
pub trait HttpResponseExt {
    fn json_error(code: ErrorCode, msg: impl Into<String>) -> HttpResponse;
    fn json_error_with_path(code: ErrorCode, msg: impl Into<String>, path: impl Into<String>) -> HttpResponse;
    fn json_success<T: Serialize>(data: T) -> HttpResponse;
}

impl HttpResponseExt for HttpResponse {
    fn json_error(code: ErrorCode, msg: impl Into<String>) -> HttpResponse {
        let status = code.http_status_code();
        HttpResponse::build(status.try_into().unwrap()).json(error_response(code, msg))
    }

    fn json_error_with_path(code: ErrorCode, msg: impl Into<String>, path: impl Into<String>) -> HttpResponse {
        let status = code.http_status_code();
        HttpResponse::build(status.try_into().unwrap()).json(error_response_with_path(code, msg, path))
    }

    fn json_success<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Ok().json(SuccessResponse::new(data))
    }
}

#[cfg(test)]
mod tests {
    include!("errors_test.rs");
}
