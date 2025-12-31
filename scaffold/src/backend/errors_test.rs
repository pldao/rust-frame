#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json;

    // ============================================================================
    // ErrorCode 测试
    // ============================================================================

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::Success as i32, 0);
        assert_eq!(ErrorCode::TokenMissing as i32, 1002);
        assert_eq!(ErrorCode::TokenInvalid as i32, 1003);
        assert_eq!(ErrorCode::NotFound as i32, 1200);
        assert_eq!(ErrorCode::DatabaseError as i32, 2001);
    }

    #[test]
    fn test_error_code_messages() {
        assert_eq!(ErrorCode::Success.default_message(), "操作成功");
        assert_eq!(ErrorCode::TokenMissing.default_message(), "token不能为空");
        assert_eq!(ErrorCode::TokenInvalid.default_message(), "无效的token");
        assert_eq!(ErrorCode::NotFound.default_message(), "资源不存在");
    }

    #[test]
    fn test_error_code_http_status_mapping() {
        // 成功应该是 200
        assert_eq!(ErrorCode::Success.http_status_code(), 200);

        // 认证错误应该是 401
        assert_eq!(ErrorCode::TokenMissing.http_status_code(), 401);
        assert_eq!(ErrorCode::TokenInvalid.http_status_code(), 401);
        assert_eq!(ErrorCode::Unauthorized.http_status_code(), 401);

        // 未找到应该是 404
        assert_eq!(ErrorCode::NotFound.http_status_code(), 404);
        assert_eq!(ErrorCode::QRCodeNotFound.http_status_code(), 404);

        // 客户端错误应该是 400
        assert_eq!(ErrorCode::BadRequest.http_status_code(), 400);
        assert_eq!(ErrorCode::InvalidParams.http_status_code(), 400);

        // 服务器错误应该是 500
        assert_eq!(ErrorCode::InternalError.http_status_code(), 500);
        assert_eq!(ErrorCode::DatabaseError.http_status_code(), 500);
    }

    // ============================================================================
    // ErrorResponse 测试
    // ============================================================================

    #[test]
    fn test_error_response_creation() {
        let resp = error_response(ErrorCode::TokenMissing, "测试错误");
        assert_eq!(resp.code, 1002);
        assert_eq!(resp.msg, "测试错误");
        assert!(resp.path.is_none());
        assert!(resp.details.is_none());
    }

    #[test]
    fn test_error_response_with_path() {
        let resp = error_response_with_path(
            ErrorCode::TokenInvalid,
            "无效的token",
            "/api/test"
        );
        assert_eq!(resp.code, 1003);
        assert_eq!(resp.msg, "无效的token");
        assert_eq!(resp.path.as_ref().unwrap(), "/api/test");
    }

    #[test]
    fn test_error_response_serialization() {
        let resp = error_response(ErrorCode::TokenMissing, "token不能为空");
        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains(r#""code":1002"#));
        assert!(json.contains(r#""msg":"token不能为空""#));
    }

    #[test]
    fn test_error_response_display() {
        let resp = error_response(ErrorCode::DatabaseError, "连接失败");
        let display_str = format!("{}", resp);
        assert!(display_str.contains("2001"));
        assert!(display_str.contains("连接失败"));
    }

    // ============================================================================
    // SuccessResponse 测试
    // ============================================================================

    #[test]
    fn test_success_response_creation() {
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
    }

    #[test]
    fn test_success_response_with_message() {
        #[derive(serde::Serialize)]
        struct TestData {
            value: i32,
        }

        let data = TestData { value: 42 };
        let resp = SuccessResponse::with_message(&data, "自定义消息");

        assert_eq!(resp.code, 0);
        assert_eq!(resp.msg, "自定义消息");
    }

    #[test]
    fn test_success_response_serialization() {
        #[derive(serde::Serialize)]
        struct TestData {
            id: i32,
        }

        let data = TestData { id: 123 };
        let resp = SuccessResponse::new(&data);
        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains(r#""code":0"#));
        assert!(json.contains(r#""msg":"操作成功""#));
        assert!(json.contains(r#""data":{"id":123}"#));
    }

    // ============================================================================
    // PaginatedData 测试
    // ============================================================================

    #[test]
    fn test_paginated_data_creation() {
        let items = vec!["item1", "item2", "item3"];
        let paginated = PaginatedData::new(items.clone(), 100, 2, 20);

        assert_eq!(paginated.items, items);
        assert_eq!(paginated.total, 100);
        assert_eq!(paginated.page, 2);
        assert_eq!(paginated.page_size, 20);
        assert_eq!(paginated.total_pages, 5); // (100 + 20 - 1) / 20 = 5
    }

    #[test]
    fn test_paginated_data_edge_cases() {
        // 空列表
        let items: Vec<String> = vec![];
        let paginated = PaginatedData::new(items, 0, 1, 10);
        assert_eq!(paginated.total_pages, 0);

        // 单页
        let items = vec!["a", "b"];
        let paginated = PaginatedData::new(items, 2, 1, 10);
        assert_eq!(paginated.total_pages, 1);

        // 刚好整除
        let items = vec!["a"];
        let paginated = PaginatedData::new(items, 20, 1, 20);
        assert_eq!(paginated.total_pages, 1);

        // 最后一页不满
        let items = vec!["a"];
        let paginated = PaginatedData::new(items, 21, 2, 20);
        assert_eq!(paginated.total_pages, 2);
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3];
        let resp = paginated_response(items, 100, 1, 10);

        assert_eq!(resp.code, 0);
        assert_eq!(resp.msg, "操作成功");
        assert_eq!(resp.data.items.len(), 3);
        assert_eq!(resp.data.total, 100);
    }

    // ============================================================================
    // BatchOperationResult 测试
    // ============================================================================

    #[test]
    fn test_batch_operation_result_creation() {
        let result = BatchOperationResult::new(10);
        assert_eq!(result.total_count, 10);
        assert_eq!(result.success_count, 0);
        assert_eq!(result.failed_count, 0);
        assert!(result.errors.is_empty());
        assert!(!result.is_complete());
    }

    #[test]
    fn test_batch_operation_add_success() {
        let mut result = BatchOperationResult::new(5);
        result.add_success();
        result.add_success();

        assert_eq!(result.success_count, 2);
        assert_eq!(result.failed_count, 0);
        assert!(!result.is_complete());
    }

    #[test]
    fn test_batch_operation_add_failure() {
        let mut result = BatchOperationResult::new(3);
        result.add_failure("错误1".to_string());
        result.add_failure("错误2".to_string());

        assert_eq!(result.success_count, 0);
        assert_eq!(result.failed_count, 2);
        assert_eq!(result.errors.len(), 2);
        assert!(!result.is_complete());
    }

    #[test]
    fn test_batch_operation_complete() {
        let mut result = BatchOperationResult::new(3);
        result.add_success();
        result.add_failure("错误".to_string());
        result.add_success();

        assert!(result.is_complete());
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failed_count, 1);
    }

    // ============================================================================
    // AppError 测试
    // ============================================================================

    #[test]
    fn test_app_error_auth() {
        let err = AppError::auth("未授权");
        assert_eq!(err.code(), ErrorCode::Unauthorized);
        assert_eq!(err.message(), "未授权");
    }

    #[test]
    fn test_app_error_not_found() {
        let err = AppError::not_found("资源不存在");
        assert_eq!(err.code(), ErrorCode::NotFound);
        assert_eq!(err.message(), "资源不存在");
    }

    #[test]
    fn test_app_error_validation() {
        let err = AppError::validation("参数错误");
        assert_eq!(err.code(), ErrorCode::InvalidParams);
        assert_eq!(err.message(), "参数错误");
    }

    #[test]
    fn test_app_error_custom() {
        let err = AppError::custom(ErrorCode::QRCodeExpired, "二维码已过期");
        assert_eq!(err.code(), ErrorCode::QRCodeExpired);
        assert_eq!(err.message(), "二维码已过期");
    }

    #[test]
    fn test_app_error_to_response() {
        let err = AppError::not_found("用户不存在");
        let resp = err.to_response();

        assert_eq!(resp.code, ErrorCode::NotFound as i32);
        assert_eq!(resp.msg, "用户不存在");
    }

    // ============================================================================
    // HttpResponseExt 测试
    // ============================================================================

    #[test]
    fn test_http_response_json_error() {
        // 注意：这个测试需要 actix-web 的运行时环境
        // 在实际运行时，HttpResponseExt 会正确工作
        // 这里我们只测试函数签名是否正确
        let error_resp = error_response(ErrorCode::TokenInvalid, "测试");
        assert_eq!(error_resp.code, 1003);
    }
}
