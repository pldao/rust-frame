/// 应用程序常量配置
///
/// 这个模块集中管理所有的魔法数字和配置常量，使代码更易于维护和修改。

/// QR 码相关常量
pub mod qr_code {
    /// QR 码过期时间（秒）- 默认 5 分钟
    pub const TTL_SECONDS: u64 = 300;

    /// QR 码图片最小尺寸
    pub const MIN_IMAGE_SIZE: u32 = 300;

    /// QR 码图片最大尺寸
    pub const MAX_IMAGE_SIZE: u32 = 300;

    /// QR 码默认容错级别
    pub const ERROR_CORRECTION_LEVEL: qrcode::EcLevel = qrcode::EcLevel::M;
}

/// JWT 相关常量
pub mod jwt {
    /// JWT 续签阈值（秒）- 剩余时间少于此时自动续签
    pub const RENEWAL_THRESHOLD_SECONDS: usize = 3600; // 1 小时

    /// JWT 默认有效期（秒）- 24 小时
    pub const DEFAULT_EXPIRATION_SECONDS: usize = 86400;

    /// 测试 Token 有效期（秒）- 24 小时
    pub const TEST_TOKEN_EXPIRATION_SECONDS: usize = 86400;
}

/// CORS 相关常量
pub mod cors {
    /// CORS 预检请求缓存时间（秒）- 1 小时
    pub const MAX_AGE: usize = 3600;
}

/// WebSocket 相关常量
pub mod websocket {
    /// WebSocket 连接超时时间（秒）- 5 分钟
    pub const SESSION_TIMEOUT_SECONDS: u64 = 300;
}

/// 邮件相关常量
pub mod email {
    /// 验证码长度
    pub const CODE_LENGTH: usize = 6;

    /// 验证码有效期（秒）- 5 分钟
    pub const CODE_TTL_SECONDS: u64 = 300;

    /// 邮件发送频率限制（秒）- 60 秒内只能发送一次
    pub const RATE_LIMIT_SECONDS: u64 = 60;
}

/// HTTP 相关常量
pub mod http {
    /// Authorization header 名称
    pub const AUTH_HEADER: &str = "Authorization";

    /// Bearer token 前缀
    pub const BEARER_PREFIX: &str = "Bearer ";

    /// Bearer token 前缀（小写变体）
    pub const BEARER_PREFIX_LOWER: &str = "bearer ";
}

/// 会话状态相关常量
pub mod session {
    /// 会话状态：已创建
    pub const STATUS_CREATED: &str = "created";

    /// 会话状态：已扫描
    pub const STATUS_SCANNED: &str = "scanned";

    /// 会话状态：已确认
    pub const STATUS_CONFIRMED: &str = "confirmed";

    /// 会话状态：已拒绝
    pub const STATUS_REJECTED: &str = "rejected";

    /// 会话状态：已过期
    pub const STATUS_EXPIRED: &str = "expired";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_are_positive() {
        assert!(qr_code::TTL_SECONDS > 0);
        assert!(jwt::RENEWAL_THRESHOLD_SECONDS > 0);
        assert!(cors::MAX_AGE > 0);
    }

    #[test]
    fn test_image_size_constraints() {
        assert!(qr_code::MIN_IMAGE_SIZE <= qr_code::MAX_IMAGE_SIZE);
    }
}
