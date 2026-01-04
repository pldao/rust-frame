use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_ws::Message;
use tracing::info;
use serde_json::json;
use crate::backend::ws_manager::WsManager;
use crate::backend::AppState;
use crate::backend::errors::{ErrorCode, error_response};
use sea_orm::{EntityTrait, DatabaseConnection};
use crate::backend::models::prelude::QrLoginSessions;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use chrono::Utc;

/// WebSocket处理：实时推送扫码登录状态
/// 
/// 路由: /ws/qr/{session_id}
/// 
/// 流程:
/// 1. Web端建立WebSocket连接
/// 2. 连接被添加到管理器中
/// 3. 保持连接，等待状态更新
/// 4. App端确认/拒绝后，服务器主动推送状态
/// 5. 推送完成后自动关闭连接
pub async fn ws_qr_status(
    req: HttpRequest,
    session_id: web::Path<String>,
    stream: web::Payload,
    ws_manager: web::Data<WsManager>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session_id = session_id.into_inner();
    
    info!("🔌 WebSocket connection request for session: {}", session_id);
    
    // 验证session是否存在
    let session_exists = QrLoginSessions::find()
        .filter(crate::backend::models::qr_login_sessions::Column::SessionId.eq(&session_id))
        .one(&state.pg_client)
        .await
        .ok()
        .flatten()
        .is_some();
    
    if !session_exists {
        info!("❌ Session not found: {}", session_id);
        let error_resp = error_response(
            ErrorCode::QRCodeNotFound,
            "Session not found"
        );
        return Ok(HttpResponse::NotFound().json(error_resp));
    }
    
    // 建立WebSocket连接
    let (response, session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    
    info!("✅ WebSocket connected for session: {}", session_id);
    
    // 将连接添加到管理器
    ws_manager.add_connection(session_id.clone(), session.clone()).await;
    
    // 发送连接成功消息 - 使用 serde_json 防止 XSS
    let mut session_clone = session.clone();
    let connect_message = json!({
        "status": "connected",
        "message": "Waiting for confirmation"
    });
    let _ = session_clone.text(connect_message.to_string()).await;
    
    // 启动心跳和消息处理任务
    let ws_manager_clone = ws_manager.clone();
    let session_id_clone = session_id.clone();
    let state_clone = state.clone();
    
    actix_web::rt::spawn(async move {
        let mut session = session;
        let mut heartbeat_interval = actix_web::rt::time::interval(std::time::Duration::from_secs(30));
        let mut timeout_check_interval = actix_web::rt::time::interval(std::time::Duration::from_secs(60));
        
        loop {
            tokio::select! {
                // 处理客户端消息
                Some(Ok(msg)) = msg_stream.recv() => {
                    match msg {
                        Message::Ping(bytes) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Message::Close(_) => {
                            info!("🔌 Client closed WebSocket for session: {}", session_id_clone);
                            break;
                        }
                        Message::Text(text) => {
                            info!("📩 Received message from client: {}", text);
                        }
                        _ => {}
                    }
                }
                // 心跳检测（30秒）
                _ = heartbeat_interval.tick() => {
                    if session.ping(b"").await.is_err() {
                        info!("❌ Heartbeat failed for session: {}", session_id_clone);
                        break;
                    }
                }
                // 超时检测（60秒）
                _ = timeout_check_interval.tick() => {
                    // 检查session是否过期
                    let expired = check_session_expired(&state_clone.pg_client, &session_id_clone).await;
                    if expired {
                        info!("⏰ Session expired, closing WebSocket: {}", session_id_clone);
                        let _ = session.text(r#"{"status":"expired","message":"QR code expired"}"#).await;
                        let _ = session.close(None).await;
                        break;
                    }
                }
            }
        }
        
        // 连接断开，从管理器中移除
        ws_manager_clone.remove_connection(&session_id_clone).await;
    });
    
    Ok(response)
}

/// 检查session是否过期
async fn check_session_expired(db: &DatabaseConnection, session_id: &str) -> bool {
    match QrLoginSessions::find()
        .filter(crate::backend::models::qr_login_sessions::Column::SessionId.eq(session_id))
        .one(db)
        .await
    {
        Ok(Some(session)) => {
            let now = Utc::now().naive_utc();
            // session已过期
            session.expires_at < now
        }
        _ => {
            // session不存在或查询失败，视为过期
            true
        }
    }
}
