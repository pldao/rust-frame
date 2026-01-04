-- 创建扫码登录会话表
CREATE TABLE IF NOT EXISTS qr_login_sessions (
    id BIGSERIAL PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    user_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    web_token TEXT,
    app_token TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_qr_login_user FOREIGN KEY (user_id) 
        REFERENCES users(user_id) 
        ON UPDATE NO ACTION 
        ON DELETE CASCADE
);

-- 为常用查询字段添加索引
CREATE INDEX idx_qr_login_session_id ON qr_login_sessions(session_id);
CREATE INDEX idx_qr_login_status ON qr_login_sessions(status);
CREATE INDEX idx_qr_login_expires_at ON qr_login_sessions(expires_at);


