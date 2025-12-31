# WebSocket 连接生命周期管理

## 📊 完整流程图

```
1. 建立连接
   │
   ├─→ Web端: new WebSocket('ws://localhost:8080/v1/ws/qr/{session_id}')
   │
   └─→ 服务器: ws_qr_status() handler
       │
       └─→ add_connection() ✅ 添加到 HashMap
           📊 Active connections: 1

2. 保持连接（等待扫码）
   │
   ├─→ 心跳 (每30秒)
   │   └─→ Ping/Pong
   │
   └─→ 监听客户端消息

3. 清理连接（4种情况）
   │
   ├─→ 情况A：登录成功（正常流程）
   │   │
   │   ├─→ App调用 POST /v1/qr-login/confirm
   │   │
   │   ├─→ notify_status()
   │   │   ├─ connections.remove() ✅ 第1次移除
   │   │   ├─ 发送消息: {"status":"confirmed","web_token":"..."}
   │   │   └─ session.close() 关闭连接
   │   │
   │   └─→ ws_status异步任务检测到Close
   │       └─ remove_connection() 🔄 第2次尝试移除（连接已不存在，静默返回）
   │
   ├─→ 情况B：Session超时（主动清理）⏰ 新增
   │   │
   │   ├─→ 每60秒检查session是否过期
   │   │
   │   ├─→ check_session_expired() 返回true
   │   │   └─ 数据库查询：expires_at < now
   │   │
   │   ├─→ 发送过期消息: {"status":"expired"}
   │   │
   │   ├─→ session.close() 关闭连接
   │   │
   │   └─→ remove_connection() ✅ 移除连接
   │
   ├─→ 情况C：客户端主动断开
   │   │
   │   ├─→ 客户端: ws.close()
   │   │
   │   └─→ ws_status检测到Message::Close
   │       └─ remove_connection() ✅ 移除连接
   │
   └─→ 情况D：连接异常
       │
       ├─→ 心跳失败 或 网络错误
       │
       └─→ ws_status异步任务退出
           └─ remove_connection() ✅ 移除连接

最终状态
   │
   └─→ 📊 Active connections: 0
```

---

## 🔍 代码分析

### 1. 添加连接

```rust
// ws_status.rs
pub async fn ws_qr_status(...) -> Result<HttpResponse, Error> {
    // 建立WebSocket连接
    let (response, session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    
    // ✅ 添加到管理器
    ws_manager.add_connection(session_id.clone(), session.clone()).await;
    
    // 启动异步任务处理消息
    actix_web::rt::spawn(async move {
        loop {
            // 监听消息和心跳
        }
        
        // ✅ 任务结束时移除连接（情况B、C）
        ws_manager_clone.remove_connection(&session_id_clone).await;
    });
    
    Ok(response)
}
```

### 2. 推送并移除（情况A）

```rust
// ws_manager.rs
pub async fn notify_status(&self, session_id: &str, status: &str, web_token: Option<&str>) {
    let mut connections = self.connections.write().await;
    
    // ✅ 从HashMap中取出并移除（第1次移除）
    if let Some(mut session) = connections.remove(session_id) {
        drop(connections); // 释放锁
        
        // 发送消息
        session.text(message).await;
        
        // 关闭连接（触发异步任务退出）
        session.close(None).await;
    }
}
```

### 3. 清理检查（所有情况）

```rust
// ws_manager.rs
pub async fn remove_connection(&self, session_id: &str) {
    let mut connections = self.connections.write().await;
    
    if connections.remove(session_id).is_some() {
        // ✅ 连接存在，移除成功
        info!("🔌 WebSocket disconnected");
    }
    // 如果连接不存在（情况A的第2次调用），静默返回
}
```

---

## ✅ 清理保证

### 情况A：登录成功
```
notify_status() remove ──┐
                         ├─→ ✅ 连接被移除
ws_status remove ────────┘    （第2次调用无操作）
```

### 情况B：客户端断开
```
客户端 close ────→ ws_status检测 ──→ ✅ remove_connection()
```

### 情况C：异常/超时
```
心跳失败/错误 ──→ 异步任务退出 ──→ ✅ remove_connection()
```

---

## 🎯 关键设计

### 1. **双重保护**
- `notify_status` 主动移除（推送场景）
- 异步任务退出时兜底移除（所有场景）

### 2. **幂等操作**
```rust
// 多次调用 remove_connection 是安全的
connections.remove(session_id)  // 第1次返回 Some
connections.remove(session_id)  // 第2次返回 None（静默）
```

### 3. **锁优化**
```rust
if let Some(mut session) = connections.remove(session_id) {
    drop(connections); // ✅ 立即释放锁
    session.text(...).await; // 不阻塞其他连接
}
```

---

## 🧪 测试验证

### 查看日志

**正常流程（情况A）：**
```
✅ WebSocket connected for session: xxx
📊 Active connections: 1
🔔 Pushing status update to session xxx: confirmed
✅ Status pushed and connection closed for session: xxx
🔌 Client closed WebSocket for session: xxx
📊 Active connections: 0  ← 第2次remove时连接已不存在，不输出日志
```

**客户端断开（情况B）：**
```
✅ WebSocket connected for session: xxx
📊 Active connections: 1
🔌 Client closed WebSocket for session: xxx
🔌 WebSocket disconnected for session: xxx
📊 Active connections: 0
```

**心跳失败（情况C）：**
```
✅ WebSocket connected for session: xxx
📊 Active connections: 1
❌ Heartbeat failed for session: xxx
🔌 WebSocket disconnected for session: xxx
📊 Active connections: 0
```

---

## ⏰ 超时机制（重要！）

### 为什么需要超时清理？

**问题场景：**
```
1. Web端建立WebSocket连接
2. Session有效期：5分钟
3. 用户打开页面但不扫码
4. 心跳正常，连接保持
5. ❌ 5分钟后session过期，但WebSocket仍然占用资源
```

### 超时检测机制

```rust
// ws_status.rs
let mut timeout_check_interval = tokio::time::interval(Duration::from_secs(60));

loop {
    tokio::select! {
        // 每60秒检查一次
        _ = timeout_check_interval.tick() => {
            let expired = check_session_expired(&db, &session_id).await;
            if expired {
                // ✅ 主动通知前端
                session.text(r#"{"status":"expired","message":"QR code expired"}"#).await;
                // ✅ 关闭连接
                session.close(None).await;
                break;
            }
        }
    }
}
```

### 检查逻辑

```rust
async fn check_session_expired(db: &DatabaseConnection, session_id: &str) -> bool {
    match QrLoginSessions::find()
        .filter(Column::SessionId.eq(session_id))
        .one(db)
        .await
    {
        Ok(Some(session)) => {
            let now = Utc::now().naive_utc();
            session.expires_at < now  // ✅ 与数据库时间比较
        }
        _ => true  // session不存在或查询失败，视为过期
    }
}
```

### 前端处理

```javascript
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    if (data.status === 'expired') {
        // ✅ 服务器主动通知过期
        alert('⏰ 二维码已过期，请重新生成');
        ws.close();
    }
};
```

---

## 📊 内存泄漏防护

### 多重保护机制

| 保护层 | 检测间隔 | 说明 |
|--------|---------|------|
| **超时检测** | 60秒 | ✅ 检查session是否过期 |
| **心跳检测** | 30秒 | ✅ 检查连接是否活跃 |
| **Session TTL** | 5分钟 | ✅ 数据库自动过期 |

### 检查活跃连接数

```rust
// 可用于监控和调试
let count = ws_manager.get_connection_count().await;
assert_eq!(count, 0, "所有连接应该被清理");
```

---

## ⚠️ 注意事项

### 1. **情况A的重复移除**

虽然有第2次 `remove_connection` 调用，但：
- ✅ 功能正确：连接已被移除
- ✅ 性能影响：极小（HashMap查询 + None判断）
- ✅ 代码简洁：避免复杂的状态同步

### 2. **为什么不用标志位？**

**不推荐：**
```rust
// ❌ 增加复杂度
let removed = Arc::new(AtomicBool::new(false));
if !removed.load(Ordering::Relaxed) {
    ws_manager.remove_connection(...).await;
    removed.store(true, Ordering::Relaxed);
}
```

**当前方案：**
```rust
// ✅ 简单可靠
ws_manager.remove_connection(...).await;  // 幂等操作
```

---

## 📝 总结

| 清理场景 | 触发点 | 移除次数 | 检测间隔 | 结果 |
|---------|--------|---------|---------|------|
| **登录成功** | notify_status + 任务退出 | 2次 | 即时 | ✅ 第1次成功，第2次静默 |
| **Session超时** ⏰ | 超时检测 | 1次 | 60秒 | ✅ 主动通知并关闭 |
| **客户端断开** | 任务检测Close | 1次 | 即时 | ✅ 成功移除 |
| **连接异常** | 心跳失败 | 1次 | 30秒 | ✅ 成功移除 |

**核心保证：**
- ✅ 所有连接都会被清理
- ✅ 不会内存泄漏
- ✅ 主动超时检测（每60秒）
- ✅ 幂等操作，无副作用
- ✅ 并发安全（RwLock保护）
- ✅ 多重保护（超时+心跳+手动）

---

**版本：** 1.1.0 (添加超时检测机制)  
**更新时间：** 2024-11-19  
**状态：** ✅ 生产就绪
