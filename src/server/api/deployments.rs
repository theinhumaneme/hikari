use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use log::info;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    mode::server::AppState,
    server::{
        dal::deploy_config_dal::DeployConfigDAL, models::deploy_config::DeployConfigDTO,
        traits::model::DataRepository,
    },
};

#[debug_handler]
pub async fn get_all_deployments(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<DeployConfigDTO>>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let value = deploy_config_dal.find_all().await.map_err(|_err| {
        info!("Unable to fetch deployments");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL SERVER ERROR".to_string(),
        )
    })?;
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
    let record_exists = deploy_config_dal.exists(id).await.map_err(|_err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL SERVER ERROR".to_string(),
        )
    })?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Deployment of ID - {id} not found"),
        ));
    }
    let value = deploy_config_dal.find_by_id(id).await.map_err(|_err| {
        info!("Unable to find Deployment by ID {id}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL SERVER ERROR".to_string(),
        )
    })?;
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
        .create(&payload.client, &payload.environment, &payload.solution)
        .await
        .map_err(|_err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR".to_string(),
            )
        })?;
    Ok(Json(deployment))
}

#[debug_handler]
pub async fn update_deployment(
    Extension(state): Extension<Arc<AppState>>,
    payload: Json<DeployConfigDTO>,
) -> Result<Json<DeployConfigDTO>, (StatusCode, String)> {
    if payload.id.is_none() {
        return Err((StatusCode::BAD_REQUEST, "id field not found".to_string()));
    }
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let record_exists = deploy_config_dal
        .exists(payload.id.unwrap())
        .await
        .map_err(|_err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR".to_string(),
            )
        })?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Deployment of ID - {} not found", payload.id.unwrap()),
        ));
    }
    let updated: bool = deploy_config_dal
        .update(
            payload.id.unwrap(),
            &payload.client,
            &payload.environment,
            &payload.solution,
        )
        .await
        .map_err(|_err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR".to_string(),
            )
        })?;
    if updated {
        deploy_config_dal
            .find_by_id(payload.id.unwrap())
            .await
            .map(Json)
            .map_err(|_err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL SERVER ERROR".to_string(),
                )
            })
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
    let record_exists = deploy_config_dal.exists(id).await.map_err(|_err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL SERVER ERROR".to_string(),
        )
    })?;
    if !record_exists {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Deployment of ID - {id} not found"),
        ));
    }

    let deployment = deploy_config_dal.find_by_id(id).await.unwrap();
    let _deleted = deploy_config_dal
        .delete(id)
        .await
        .map(|value| {
            if value {
                Ok(Json(deployment))
            } else {
                Err((StatusCode::BAD_REQUEST, "lorem ipsum"))
            }
        })
        .map_err(|_err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR".to_string(),
            )
        });
    Err((
        StatusCode::BAD_REQUEST,
        format!("Unable to delete Deployment ID - {id}"),
    ))
}
