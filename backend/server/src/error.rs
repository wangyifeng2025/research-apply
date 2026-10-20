use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("缺少环境变量 {0}")]
    MissingEnv(&'static str),
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("跨域来源无效: {0}")]
    InvalidCorsOrigin(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "请求失败");
        let status = match self {
            Self::Database(_) | Self::Io(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::MissingEnv(_) | Self::InvalidCorsOrigin(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({ "error": "服务暂时不可用" }));
        (status, body).into_response()
    }
}
