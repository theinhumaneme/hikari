use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use log::error;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{objects::dto::DeployConfigDTO, server::app_state::AppState};

#[debug_handler]
pub async fn get_all_deployments(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<DeployConfigDTO>>, (StatusCode, String)> {
    let deployment: Vec<DeployConfigDTO> =
        sqlx::query_as!(DeployConfigDTO, "SELECT * FROM deployments;")
            .fetch_all(&state.pool)
            .await
            .map_err(|err| {
                // Log the error server-side for debugging
                // :contentReference[oaicite:13]{index=13}
                error!("Database query failed: {}", err);
                // Return a generic error message to the client with 500 status
                // :contentReference[oaicite:14]{index=14}
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to fetch deployments".to_string(),
                )
            })?;
    Ok(Json(deployment))
}
#[derive(Deserialize)]
pub struct QueryParams {
    pub id: i64,
}

#[debug_handler]
pub async fn get_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<Vec<DeployConfigDTO>>, (StatusCode, String)> {
    let deployment: Vec<DeployConfigDTO> = sqlx::query_as!(
        DeployConfigDTO,
        "SELECT id, client, environment, solution FROM deployments WHERE id=$1;",
        id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|err| {
        // Log the error server-side for debugging
        // :contentReference[oaicite:13]{index=13}
        error!("Database query failed: {}", err);
        // Return a generic error message to the client with 500 status
        // :contentReference[oaicite:14]{index=14}
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch deployments".to_string(),
        )
    })?;
    Ok(Json(deployment))
}

#[debug_handler]
pub async fn post_deployment(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<DeployConfigDTO>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    if payload.id.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Received an unexpected field - id".to_string(),
        ));
    }
    let deployment: DeployConfigDTO = sqlx::query_as!(
        DeployConfigDTO,
        "INSERT INTO deployments(client, environment, solution
        ) VALUES ($1, $2, $3) RETURNING id, client, environment, solution;",
        payload.client,
        payload.environment,
        payload.solution
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        // Log the error server-side for debugging
        // :contentReference[oaicite:13]{index=13}
        error!("Database query failed: {}", err);
        // Return a generic error message to the client with 500 status
        // :contentReference[oaicite:14]{index=14}
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch deployments".to_string(),
        )
    })?;
    Ok(Json(deployment))
}

#[debug_handler]
pub async fn update_deployment(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<DeployConfigDTO>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    let deployment: DeployConfigDTO = sqlx::query_as!(
        DeployConfigDTO,
        "UPDATE deployments SET client=$2, environment=$3, solution=$4 WHERE id=$1 RETURNING id, client, environment, solution;",
        payload.id,
        payload.client,
        payload.environment,
        payload.solution
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        // Log the error server-side for debugging :contentReference[oaicite:13]{index=13}
        error!("Database query failed: {}", err);
        // Return a generic error message to the client with 500 status :contentReference[oaicite:14]{index=14}
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch deployments".to_string(),
        )
    })?;
    Ok(Json(deployment))
}

#[debug_handler]
pub async fn delete_deployment(
    Extension(state): Extension<Arc<AppState>>,

    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    let deployment: DeployConfigDTO = sqlx::query_as!(
        DeployConfigDTO,
        "DELETE FROM deployments WHERE id=$1 RETURNING id, client, environment, solution;",
        id,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        // Log the error server-side for debugging
        // :contentReference[oaicite:13]{index=13}
        error!("Database query failed: {}", err);
        // Return a generic error message to the client with 500 status
        // :contentReference[oaicite:14]{index=14}
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch deployments".to_string(),
        )
    })?;
    Ok(Json(deployment))
}
