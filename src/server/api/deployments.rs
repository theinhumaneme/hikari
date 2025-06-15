use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    mode::server::AppState,
    server::{
        common::map_db_error, dal::deploy_config_dal::DeployConfigDAL,
        models::deploy_config::DeployConfigDTO, traits::model::DataRepository,
    },
};

#[debug_handler]
pub async fn get_all_deployments(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<DeployConfigDTO>>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let value = deploy_config_dal.find_all().await.map_err(map_db_error)?;
    Ok(Json(value))
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
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let value = deploy_config_dal
        .find_by_id(id)
        .await
        .map_err(map_db_error)?;
    Ok(Json(value))
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
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let deployment = deploy_config_dal
        .create(DeployConfigDTO {
            id: payload.id,
            name: payload.name.clone(),
            client: payload.client.clone(),
            environment: payload.environment.clone(),
            solution: payload.solution.clone(),
            stack_ids: payload.stack_ids.clone(),
        })
        .await
        .map_err(map_db_error)?;
    Ok(Json(deployment))
}

#[debug_handler]
pub async fn update_deployment(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<DeployConfigDTO>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let record_exists = deploy_config_dal
        .exists(payload.id.unwrap())
        .await
        .map_err(map_db_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Deployment of ID - {} not found", payload.id.unwrap()),
        ));
    }
    let updated: bool = deploy_config_dal
        .update(DeployConfigDTO {
            id: payload.id,
            name: payload.name.clone(),
            client: payload.client.clone(),
            environment: payload.environment.clone(),
            solution: payload.solution.clone(),
            stack_ids: payload.stack_ids.clone(),
        })
        .await
        .map_err(map_db_error)?;
    if updated {
        deploy_config_dal
            .find_by_id(payload.id.unwrap())
            .await
            .map(Json)
            .map_err(map_db_error)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            format!("Unable to update Deployment ID - {}", payload.id.unwrap()),
        ))
    }
}

#[debug_handler]
pub async fn delete_deployment(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let record_exists = deploy_config_dal.exists(id).await.map_err(map_db_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Deployment of ID - {id} not found"),
        ));
    }
    let deployment = deploy_config_dal
        .find_by_id(id)
        .await
        .map_err(map_db_error)?;
    let deleted = deploy_config_dal.delete(id).await.map_err(map_db_error)?;
    if deleted {
        Ok(Json(deployment))
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "Unable to delete deployment".to_string(),
        ))
    }
}
