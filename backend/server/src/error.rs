use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::{DbErr, SqlErr};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("缺少环境变量 {0}")]
    MissingEnv(&'static str),
    #[error("跨域来源无效: {0}")]
    InvalidCorsOrigin(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("提交的数据未通过校验")]
    Validation(Vec<FieldError>),
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Database(error) if is_unique_violation(error) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Io(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::MissingEnv(_) | Self::InvalidCorsOrigin(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn client_message(&self) -> String {
        match self {
            Self::Database(error) if is_unique_violation(error) => "数据已存在".to_owned(),
            Self::Database(_)
            | Self::Io(_)
            | Self::Internal(_)
            | Self::MissingEnv(_)
            | Self::InvalidCorsOrigin(_) => "服务暂时不可用".to_owned(),
            Self::Validation(_) => "提交的数据未通过校验".to_owned(),
            other => other.to_string(),
        }
    }
}

fn is_unique_violation(error: &DbErr) -> bool {
    matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_)))
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!(error = %self, "请求失败");
        } else if let Self::Validation(fields) = &self {
            let detail = fields
                .iter()
                .map(|item| format!("{}：{}", item.field, item.message))
                .collect::<Vec<_>>()
                .join("；");
            tracing::warn!(detail = %detail, "请求被拒绝");
        } else {
            tracing::warn!(error = %self, "请求被拒绝");
        }
        let fields = match &self {
            Self::Validation(fields) => Some(fields.clone()),
            _ => None,
        };
        let body = Json(json!({
            "error": self.client_message(),
            "fields": fields,
        }));
        (status, body).into_response()
    }
}
