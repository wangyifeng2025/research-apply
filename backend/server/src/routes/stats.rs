use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::stats::{self, StatsOverview, StatsQuery};
use crate::state::AppState;

pub async fn overview(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(query): Query<StatsQuery>,
) -> Result<Json<StatsOverview>, AppError> {
    Ok(Json(stats::overview(&state, &user, query).await?))
}

pub async fn export(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(query): Query<StatsQuery>,
) -> Result<Response, AppError> {
    let bytes = stats::export_xlsx(&state, &user, query).await?;
    let disposition = HeaderValue::from_static("attachment; filename=\"application-stats.xlsx\"");
    let content_type = HeaderValue::from_static(
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );
    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    )
        .into_response())
}
