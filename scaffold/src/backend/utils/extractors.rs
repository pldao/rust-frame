use actix_web::HttpMessage;
use crate::backend::config::http;
use crate::backend::errors::AppError;

/// 从 HTTP 请求中提取 JWT Token
///
/// 此函数提供了一个统一的方式从 Authorization header 中提取 Bearer token
/// 避免了在多个地方重复相同的逻辑
///
/// # Arguments
/// * `req` - HTTP 请求引用
///
/// # Returns
/// * `Ok(String)` - 成功提取的 token
/// * `Err(AppError)` - 提取失败时的错误信息
///
/// # Examples
/// ```rust
/// use actix_web::test;
///
/// let req = test::TestRequest::default()
///     .insert_header(("Authorization", "Bearer my_token"))
///     .to_http_request();
///
/// let token = extract_token_from_request(&req)?;
/// assert_eq!(token, "my_token");
/// ```
pub fn extract_token_from_request(req: &impl HttpMessage) -> Result<String, AppError> {
    req.headers()
        .get(http::AUTH_HEADER)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| {
            header_str
                .strip_prefix(http::BEARER_PREFIX)
                .or_else(|| header_str.strip_prefix(http::BEARER_PREFIX_LOWER))
        })
        .ok_or_else(|| {
            AppError::auth("Missing or invalid Authorization header. Expected format: Bearer <token>")
        })
        .map(|s| s.to_string())
}

/// 检查请求路径是否在排除列表中
///
/// 用于中间件判断某些路径是否需要跳过认证
///
/// # Arguments
/// * `path` - 请求路径
/// * `excluded_paths` - 排除的路径列表
///
/// # Returns
/// * `true` - 路径在排除列表中
/// * `false` - 路径不在排除列表中
pub fn is_excluded_path(path: &str, excluded_paths: &[&str]) -> bool {
    excluded_paths.iter().any(|excluded| path.starts_with(excluded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_extract_token_valid() {
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "Bearer test_token_123"))
            .to_http_request();

        let result = extract_token_from_request(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_token_123");
    }

    #[test]
    fn test_extract_token_lowercase_bearer() {
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "bearer test_token_123"))
            .to_http_request();

        let result = extract_token_from_request(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_token_123");
    }

    #[test]
    fn test_extract_token_missing_header() {
        let req = test::TestRequest::default()
            .to_http_request();

        let result = extract_token_from_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_invalid_format() {
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "InvalidFormat test_token"))
            .to_http_request();

        let result = extract_token_from_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_excluded_path() {
        let excluded = vec!["/ping", "/qr-login", "/ws"];

        assert!(is_excluded_path("/ping", &excluded));
        assert!(is_excluded_path("/qr-login/generate", &excluded));
        assert!(is_excluded_path("/ws/qr/123", &excluded));
        assert!(!is_excluded_path("/user/me", &excluded));
        assert!(!is_excluded_path("/v2/protected", &excluded));
    }
}
