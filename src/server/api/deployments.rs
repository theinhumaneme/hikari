use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use log::{error, trace};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::{query, query_as};

use crate::{objects::dto::DeployConfigDTO, server::app_state::AppState};

#[debug_handler]
pub async fn get_all_deployments(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<DeployConfigDTO>>, (StatusCode, String)> {
    let deployment: Vec<DeployConfigDTO> = query_as!(
        DeployConfigDTO,
        r#"
        SELECT dc.id,
        dc.client,
        dc.environment,
        dc.solution,
        COALESCE(
            array_agg(cs.id) FILTER (WHERE cs.id IS NOT NULL),
            ARRAY[]::BIGINT[]
        ) AS stack_ids
        FROM deploy_config AS dc
        LEFT JOIN compose_stack AS cs
        ON cs.deployment_id = dc.id
        GROUP BY dc.id, dc.client, dc.environment, dc.solution;
        "#,
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
#[derive(Deserialize)]
pub struct QueryParams {
    pub id: i64,
}

#[debug_handler]
pub async fn get_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    let deployment: DeployConfigDTO = query_as!(
        DeployConfigDTO,
        r#"
        SELECT dc.id,
        dc.client,
        dc.environment,
        dc.solution,
        COALESCE(
            array_agg(cs.id) FILTER (WHERE cs.id IS NOT NULL),
            ARRAY[]::BIGINT[]
        ) AS stack_ids
        FROM deploy_config AS dc
        LEFT JOIN compose_stack AS cs
        ON cs.deployment_id = dc.id
        WHERE dc.id = $1
        GROUP BY dc.id, dc.client, dc.environment, dc.solution;
        "#,
        id
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
            "Failed to fetch deployment".to_string(),
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
    let row = query!(
        "INSERT INTO deploy_config(client, environment, solution
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
            "Failed to insert deployment".to_string(),
        )
    })?;

    Ok(Json(DeployConfigDTO {
        id: Some(row.id),
        client: row.client,
        environment: row.environment,
        solution: row.solution,
        stack_ids: Some(Vec::<i64>::new()),
    }))
}

#[debug_handler]
pub async fn update_deployment(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<DeployConfigDTO>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    let row= sqlx::query!(
        "UPDATE deploy_config SET client=$2, environment=$3, solution=$4 WHERE id=$1 RETURNING id, client, environment, solution;",
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
    let stack_row = query!(
        r#"
        SELECT
        COALESCE(
        array_agg(cs.id) FILTER (WHERE cs.id IS NOT NULL),
        ARRAY[]::BIGINT[]
        ) AS stack_ids
        FROM compose_stack AS cs
        WHERE cs.deployment_id = $1
    "#,
        row.id
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
            "Failed to fetch stacks".to_string(),
        )
    })?;

    Ok(Json(DeployConfigDTO {
        id: Some(row.id),
        client: row.client,
        environment: row.environment,
        solution: row.solution,
        stack_ids: stack_row.stack_ids,
    }))
}

#[debug_handler]
pub async fn delete_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    let exists_row = sqlx::query!(
        r#"
        SELECT EXISTS(
        SELECT 1
        FROM deploy_config
        WHERE id = $1
        ) AS "exists!"
        "#,
        id,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        error!("DB error: {}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "DB failure".into())
    })?;
    if !exists_row.exists {
        // row does not exist → return 404
        return Err((StatusCode::NOT_FOUND, "Deployment not found".into()));
    }

    let row = query!(
        r#"DELETE FROM deploy_config WHERE id=$1 RETURNING id, client, environment, solution;"#,
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
    Ok(Json(DeployConfigDTO {
        id: Some(row.id),
        client: row.client,
        environment: row.environment,
        solution: row.solution,
        stack_ids: Some(Vec::<i64>::new()),
    }))
}
