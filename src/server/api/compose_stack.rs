use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    mode::server::AppState,
    server::{
        dal::{deploy_config_dal::DeployConfigDAL, stack_config_dal::StackConfigDAL},
        models::stack_config::StackConfigDTO,
        traits::model::DataRepository,
        utils::utils::map_db_error,
    },
};

#[debug_handler]
pub async fn get_all_stacks(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<StackConfigDTO>>, (StatusCode, String)> {
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let value = stack_config_dal.find_all().await.map_err(map_db_error)?;
    Ok(Json(value))
}
#[derive(Deserialize)]
pub struct QueryParams {
    pub id: i64,
}

#[debug_handler]
pub async fn get_stack(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<StackConfigDTO>, (StatusCode, String)> {
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let value = stack_config_dal
        .find_by_id(id)
        .await
        .map_err(map_db_error)?;
    Ok(Json(value))
}

#[debug_handler]
pub async fn post_stack(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<StackConfigDTO>,
) -> Result<Json<StackConfigDTO>, (StatusCode, String)> {
    if payload.id.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Received an unexpected field - id".to_string(),
        ));
    }
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let deployment_exists = deploy_config_dal
        .exists(payload.deployment_id)
        .await
        .map_err(map_db_error)?;
    if !deployment_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("deployment_id - {} not found", payload.deployment_id),
        ));
    }
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let stack = stack_config_dal
        .create(StackConfigDTO {
            id: payload.id,
            deployment_id: payload.deployment_id,
            stack_name: payload.stack_name.clone(),
            filename: payload.stack_name.clone(),
            home_directory: payload.home_directory.clone(),
            containers: payload.containers.clone(),
        })
        .await
        .map_err(map_db_error)?;
    Ok(Json(stack))
}

#[debug_handler]
pub async fn update_stack(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<StackConfigDTO>,
) -> Result<Json<StackConfigDTO>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let deployment_exists = deploy_config_dal
        .exists(payload.deployment_id)
        .await
        .map_err(map_db_error)?;
    if !deployment_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("deployment_id - {} not found", payload.deployment_id),
        ));
    }
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let record_exists = stack_config_dal
        .exists(payload.id.unwrap())
        .await
        .map_err(map_db_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Stack of ID - {} not found", payload.id.unwrap()),
        ));
    }
    let updated: bool = stack_config_dal
        .update(StackConfigDTO {
            id: payload.id,
            deployment_id: payload.deployment_id,
            stack_name: payload.stack_name.clone(),
            filename: payload.stack_name.clone(),
            home_directory: payload.home_directory.clone(),
            containers: payload.containers.clone(),
        })
        .await
        .map_err(map_db_error)?;
    if updated {
        stack_config_dal
            .find_by_id(payload.id.unwrap())
            .await
            .map(Json)
            .map_err(map_db_error)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            format!("Unable to update Stack ID - {}", payload.id.unwrap()),
        ))
    }
}

#[debug_handler]
pub async fn delete_stack(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<StackConfigDTO>, (StatusCode, String)> {
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let record_exists = stack_config_dal.exists(id).await.map_err(map_db_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Stack of ID - {id} not found"),
        ));
    }

    let stack = stack_config_dal
        .find_by_id(id)
        .await
        .map_err(map_db_error)?;
    let deleted = stack_config_dal.delete(id).await.map_err(map_db_error)?;
    if deleted {
        Ok(Json(stack))
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "Unable to delete deployment".to_string(),
        ))
    }
}
