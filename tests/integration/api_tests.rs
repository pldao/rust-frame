/// QR登录API集成测试
///
/// 这些测试需要测试数据库运行
///
/// 运行方式：
/// ```bash
/// export TEST_DATABASE_URL="postgres://test_user:test_password@localhost:5433/test_db"
/// cargo test --test integration
/// ```

#[cfg(test)]
mod tests {
    // 注意：这些是集成测试的框架
    // 实际运行时需要真实的数据库连接

    #[test]
    fn test_error_codes() {
        // 测试错误码系统
        use rust_frame::backend::errors::{ErrorCode, error_response};

        // 测试错误码值
        assert_eq!(ErrorCode::Success as i32, 0);
        assert_eq!(ErrorCode::TokenMissing as i32, 1002);
        assert_eq!(ErrorCode::NotFound as i32, 1200);

        // 测试错误响应
        let resp = error_response(ErrorCode::TokenInvalid, "测试错误");
        assert_eq!(resp.code, 1003);
        assert_eq!(resp.msg, "测试错误");

        println!("✓ 错误码系统测试通过");
    }

    #[test]
    fn test_success_response() {
        use rust_frame::backend::errors::SuccessResponse;
        use serde_json::to_string;

        #[derive(serde::Serialize)]
        struct TestData {
            id: i32,
            name: String,
        }

        let data = TestData {
            id: 1,
            name: "测试".to_string(),
        };

        let resp = SuccessResponse::new(&data);
        assert_eq!(resp.code, 0);
        assert_eq!(resp.msg, "操作成功");

        let json = to_string(&resp).unwrap();
        assert!(json.contains(r#""code":0"#));
        assert!(json.contains(r#""msg":"操作成功""#));

        println!("✓ 成功响应测试通过");
    }

    #[test]
    fn test_paginated_response() {
        use rust_frame::backend::errors::{PaginatedData, paginated_response};

        let items = vec!["item1", "item2", "item3"];
        let paginated = PaginatedData::new(items.clone(), 100, 2, 20);

        assert_eq!(paginated.total, 100);
        assert_eq!(paginated.page, 2);
        assert_eq!(paginated.page_size, 20);
        assert_eq!(paginated.total_pages, 5);

        let resp = paginated_response(items, 100, 1, 10);
        assert_eq!(resp.code, 0);

        println!("✓ 分页响应测试通过");
    }

    #[test]
    fn test_batch_operation_result() {
        use rust_frame::backend::errors::BatchOperationResult;

        let mut result = BatchOperationResult::new(5);
        assert_eq!(result.total_count, 5);
        assert!(!result.is_complete());

        result.add_success();
        result.add_success();
        result.add_failure("错误1".to_string());

        assert_eq!(result.success_count, 2);
        assert_eq!(result.failed_count, 1);
        assert!(!result.is_complete());

        result.add_success();
        result.add_success();

        assert!(result.is_complete());

        println!("✓ 批量操作结果测试通过");
    }

    // 注意：以下测试需要真实的数据库连接
    // 在实际环境中，需要设置测试数据库并取消注释

    /*
    #[tokio::test]
    async fn test_qr_login_flow() {
        use actix_web::{test, web, App};
        use rust_frame::backend::app_router::configure_app;
        use rust_frame::backend::AppState;
        use sea_orm::Database;

        let database_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set");

        let db = Database::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let app_state = AppState { pg_client: db };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .configure(configure_app)
        ).await;

        // 测试生成二维码
        let req = test::TestRequest::post()
            .uri("/qr-login/generate")
            .set_json(&serde_json::json!({"client_info": "test"}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["code"], 0);
        assert!(body["data"]["session_id"].is_string());

        println!("✓ QR登录流程测试通过");
    }
    */
}
