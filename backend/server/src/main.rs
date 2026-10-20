//! 黔西南民族职业技术学院院级科研申报服务。

mod config;
mod error;
mod routes;
mod state;

use config::Config;
use error::AppError;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=info")),
        )
        .init();

    let config = Config::from_env()?;
    let state = AppState::connect(&config).await?;
    let app = routes::router(state, &config)?;
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;

    tracing::info!(bind = %config.bind, "科研申报服务已启动");
    axum::serve(listener, app).await?;
    Ok(())
}
