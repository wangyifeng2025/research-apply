//! 黔西南民族职业技术学院院级科研申报服务。

pub mod auth;
pub mod config;
pub mod error;
pub mod routes;
pub mod seed;
pub mod services;
pub mod state;
pub mod support;

use std::net::SocketAddr;

use error::AppError;
use state::AppState;

pub async fn run() -> Result<(), AppError> {
    //读取.env文件，并设置环境变量
    dotenvy::dotenv().ok();
    support::log::init();

    let config = config::Config::from_env()?;
    let state = AppState::connect(&config).await?;
    let app = routes::router(state, &config)?;
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;

    tracing::info!(bind = %config.bind, "科研申报服务已启动");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
