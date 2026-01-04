pub mod constants;

// 重新导出常用常量，方便使用
pub use constants::{
    cors, email, http, jwt, qr_code, session, websocket,
};
