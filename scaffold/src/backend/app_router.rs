use actix_web::{App, HttpServer, web, middleware, http, Responder, HttpResponse, Scope};
use actix_cors::Cors;
use sea_orm::DbConn;
use tracing::info;
use crate::backend::AppState;
use crate::backend::middleware::auth_middleware::Auth;
use crate::backend::middleware::time::Timed;
use crate::backend::api::auth::auth_scope;
// use crate::backend::api::password::password_scope;
// use crate::backend::api::admin::admin_scope;
// use crate::backend::api::logs::logs_scope;
use crate::backend::api::code::code_scope;
use crate::backend::api::qr_login::{qr_login_scope, ws_qr_route};
use crate::backend::api::user::{user_scope, test_scope};
use crate::backend::ws_manager::WsManager;

pub async fn run_backend_server(
    pg_client: DbConn,
    backend_port: u16,
) -> std::io::Result<()> {
    info!("🌐 Starting HTTP server on 0.0.0.0:{}", backend_port);
    
    // 创建WebSocket管理器
    let ws_manager = WsManager::new();
    
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::default()
                      .allow_any_origin()
                      .allow_any_header()
                      .send_wildcard()
                      .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                      .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                      .allowed_header(http::header::CONTENT_TYPE)
                      .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(AppState { pg_client: pg_client.clone() }))
            .app_data(web::Data::new(ws_manager.clone()))
            // ==================== v1 API: 公开接口（不需要认证）====================
            .service(
                web::scope("/v1")
                    .wrap(Timed)
                    .route("/ping", web::get().to(router_hello))
                    .service(auth_scope())     // 用户注册/登录
                    .service(code_scope())     // 验证码
                    .service(qr_login_scope()) // 扫码登录（生成二维码、查询状态）
                    .service(test_scope())     // 测试接口（生成 token）
                    // WebSocket路由
                    .route("/ws/qr/{session_id}", ws_qr_route())
            )
            // ==================== v2 API: 需要认证的接口 ====================
            .service(
                web::scope("/v2")
                    .wrap(Timed)
                    .wrap(Auth)
                    .service(user_scope())     // 用户信息管理
                    // .service(admin_scope())    // 管理员接口
            )
    })
        .bind(("0.0.0.0", backend_port))?;
    
    info!("✅ Server listening on http://0.0.0.0:{}", backend_port);
    info!("");
    info!("� API Routes:");
    info!("  ├─ v1 (公开接口，无需认证):");
    info!("  │  ├─ 🏓 Health: http://localhost:{}/v1/ping", backend_port);
    info!("  │  ├─ �📡 QR Login: http://localhost:{}/v1/qr-login/generate", backend_port);
    info!("  │  ├─ 🔌 WebSocket: ws://localhost:{}/v1/ws/qr/{{session_id}}", backend_port);
    info!("  │  ├─ 🔐 Auth: http://localhost:{}/v1/auth/*", backend_port);
    info!("  │  ├─ 📧 Code: http://localhost:{}/v1/code/*", backend_port);
    info!("  │  └─ 🧪 Test: http://localhost:{}/v1/test/generate-token", backend_port);
    info!("  │");
    info!("  └─ v2 (需要认证):");
    info!("     └─ 👤 User: http://localhost:{}/v2/user/me", backend_port);
    info!("");
    
    server.run().await
}

pub async fn router_hello() -> impl Responder {
    info!("Hello World");
    HttpResponse::Ok().body("Pong")
}