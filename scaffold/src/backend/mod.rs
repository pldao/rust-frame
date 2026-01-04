use sea_orm::DbConn;

pub mod models;
pub mod app_router;
pub mod ws_manager;
pub mod errors;
pub mod config;
mod middleware;
mod utils;
mod api;

#[derive(Clone)]
pub struct AppState {
    pub pg_client: DbConn
}