use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    mode::server::AppState,
    server::{
        common::map_repo_error,
        dal::{container_dal::ContainerDAL, stack_config_dal::StackConfigDAL},
        models::container::ContainerDTO,
        traits::model::DataRepository,
        ws::websocket::broadcast,
    },
};

#[debug_handler]
pub async fn get_all_containers(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<ContainerDTO>>, (StatusCode, String)> {
    let container_config_dal = ContainerDAL::new(&state.pool);
    let value = container_config_dal
        .find_all()
        .await
        .map_err(map_repo_error)?;
    Ok(Json(value))
}
#[derive(Deserialize)]
pub struct QueryParams {
    pub id: i64,
}

#[debug_handler]
pub async fn get_container(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<ContainerDTO>, (StatusCode, String)> {
    let container_config_dal = ContainerDAL::new(&state.pool);
    let value = container_config_dal
        .find_by_id(id)
        .await
        .map_err(map_repo_error)?;
    Ok(Json(value))
}

#[debug_handler]
pub async fn post_container(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<ContainerDTO>,
) -> Result<Json<ContainerDTO>, (StatusCode, String)> {
    if payload.id.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Received an unexpected field - id".to_string(),
        ));
    }
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let stack_exists = stack_config_dal
        .exists(payload.stack_id)
        .await
        .map_err(map_repo_error)?;
    if !stack_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("stack_id - {} not found", payload.stack_id),
        ));
    }
    let container_config_dal = ContainerDAL::new(&state.pool);
    let container = container_config_dal
        .create(ContainerDTO {
            id: payload.id,
            stack_id: payload.stack_id,
            service_name: payload.service_name.clone(),
            container_name: payload.container_name.clone(),
            image: payload.image.clone(),
            restart: payload.restart.clone(),
            user: payload.user.clone(),
            stdin_open: payload.stdin_open,
            tty: payload.tty,
            command: payload.command.clone(),
            pull_policy: payload.pull_policy.clone(),
            ports: payload.ports.clone(),
            volumes: payload.volumes.clone(),
            environment: payload.environment.clone(),
            mem_reservation: payload.mem_reservation.clone(),
            mem_limit: payload.mem_limit.clone(),
            oom_kill_disable: payload.oom_kill_disable,
            privileged: payload.privileged,
        })
        .await
        .map_err(map_repo_error)?;
    let deployment = container_config_dal
        .get_deployment_metadata(container.id.unwrap())
        .await
        .map_err(map_repo_error)?;
    tokio::spawn(async move {
        broadcast(
            state,
            deployment.client,
            deployment.solution,
            deployment.environment,
        )
        .await
    });
    Ok(Json(container))
}

#[debug_handler]
pub async fn update_container(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<ContainerDTO>,
) -> Result<Json<ContainerDTO>, (StatusCode, String)> {
    if payload.id.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Expected field - id".to_string()));
    }
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let stack_exists = stack_config_dal
        .exists(payload.stack_id)
        .await
        .map_err(map_repo_error)?;
    if !stack_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("stack_id - {} not found", payload.stack_id),
        ));
    }
    let container_config_dal = ContainerDAL::new(&state.pool);
    let record_exists = container_config_dal
        .exists(payload.id.unwrap())
        .await
        .map_err(map_repo_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Container of ID - {} not found", payload.id.unwrap()),
        ));
    }
    if payload.0
        == container_config_dal
            .find_by_id(payload.id.unwrap())
            .await
            .map_err(map_repo_error)?
    {
        return Err((
            StatusCode::NOT_MODIFIED,
            format!("Deployment of ID - {} is not modified", payload.id.unwrap()),
        ));
    }

    let updated: bool = container_config_dal
        .update(ContainerDTO {
            id: payload.id,
            stack_id: payload.stack_id,
            service_name: payload.service_name.clone(),
            container_name: payload.container_name.clone(),
            image: payload.image.clone(),
            restart: payload.restart.clone(),
            user: payload.user.clone(),
            stdin_open: payload.stdin_open,
            tty: payload.tty,
            command: payload.command.clone(),
            pull_policy: payload.pull_policy.clone(),
            ports: payload.ports.clone(),
            volumes: payload.volumes.clone(),
            environment: payload.environment.clone(),
            mem_reservation: payload.mem_reservation.clone(),
            mem_limit: payload.mem_limit.clone(),
            oom_kill_disable: payload.oom_kill_disable,
            privileged: payload.privileged,
        })
        .await
        .map_err(map_repo_error)?;
    if updated {
        let deployment = container_config_dal
            .get_deployment_metadata(payload.id.unwrap())
            .await
            .map_err(map_repo_error)?;
        tokio::spawn(async move {
            broadcast(
                state,
                deployment.client,
                deployment.solution,
                deployment.environment,
            )
            .await
        });
        container_config_dal
            .find_by_id(payload.id.unwrap())
            .await
            .map(Json)
            .map_err(map_repo_error)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            format!("Unable to update Conatainer ID - {}", payload.id.unwrap()),
        ))
    }
}

#[debug_handler]
pub async fn delete_container(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParams { id }): Query<QueryParams>,
) -> Result<Json<ContainerDTO>, (StatusCode, String)> {
    let container_config_dal = ContainerDAL::new(&state.pool);
    let record_exists = container_config_dal
        .exists(id)
        .await
        .map_err(map_repo_error)?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Container of ID - {id} not found"),
        ));
    }
    let container = container_config_dal
        .find_by_id(id)
        .await
        .map_err(map_repo_error)?;
    let deleted = container_config_dal
        .delete(id)
        .await
        .map_err(map_repo_error)?;
    if deleted {
        let deployment = container_config_dal
            .get_deployment_metadata(id)
            .await
            .map_err(map_repo_error)?;
        tokio::spawn(async move {
            broadcast(
                state,
                deployment.client,
                deployment.solution,
                deployment.environment,
            )
            .await
        });
        Ok(Json(container))
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "Unable to delete Container".to_string(),
        ))
    }
}
