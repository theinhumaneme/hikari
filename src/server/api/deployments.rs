use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    mode::server::AppState,
    server::{
        common::map_repo_error, dal::deploy_config_dal::DeployConfigDAL,
        models::deploy_config::DeployConfigDTO, traits::model::DataRepository,
        ws::websocket::broadcast,
    },
};

#[debug_handler]
pub async fn get_all_deployments(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<DeployConfigDTO>>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let value = deploy_config_dal.find_all().await.map_err(map_repo_error)?;
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
        .map_err(map_repo_error)?;
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
        .map_err(map_repo_error)?;
    let deployment_temp = deployment.clone();
    tokio::spawn(async move {
        broadcast(
            state,
            deployment_temp.client,
            deployment_temp.solution,
            deployment_temp.environment,
        )
        .await
    });
    Ok(Json(deployment))
}

#[debug_handler]
pub async fn update_deployment(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<DeployConfigDTO>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    if payload.id.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Expected field - id".to_string()));
    }
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let record_exists = deploy_config_dal
        .exists(payload.id.unwrap())
        .await
        .map_err(map_repo_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Deployment of ID - {} not found", payload.id.unwrap()),
        ));
    }
    if payload.0
        == deploy_config_dal
            .find_by_id(payload.id.unwrap())
            .await
            .map_err(map_repo_error)?
    {
        return Err((
            StatusCode::NOT_MODIFIED,
            format!("Deployment of ID - {} is not modified", payload.id.unwrap()),
        ));
    }
    let deployment = deploy_config_dal
        .get_deployment_metadata(payload.id.unwrap())
        .await
        .map_err(map_repo_error)?;
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
        .map_err(map_repo_error)?;
    if updated {
        let temp_payload = payload.clone();
        tokio::spawn(async move {
            let _ = broadcast(
                state.clone(),
                temp_payload.client.clone(),
                temp_payload.solution.clone(),
                temp_payload.environment.clone(),
            )
            .await;
            let _ = broadcast(
                state,
                deployment.client,
                deployment.solution,
                deployment.environment,
            )
            .await;
        });
        deploy_config_dal
            .find_by_id(payload.id.unwrap())
            .await
            .map(Json)
            .map_err(map_repo_error)
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
    let record_exists = deploy_config_dal.exists(id).await.map_err(map_repo_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Deployment of ID - {id} not found"),
        ));
    }
    let deployment = deploy_config_dal
        .find_by_id(id)
        .await
        .map_err(map_repo_error)?;
    let deleted = deploy_config_dal.delete(id).await.map_err(map_repo_error)?;
    if deleted {
        let deployment_temp = deployment.clone();
        tokio::spawn(async move {
            broadcast(
                state,
                deployment_temp.client,
                deployment_temp.solution,
                deployment_temp.environment,
            )
            .await
        });
        Ok(Json(deployment))
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "Unable to delete deployment".to_string(),
        ))
    }
}
