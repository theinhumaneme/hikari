use std::sync::Arc;

use axum::{Extension, Json, debug_handler, extract::Query};
use log::info;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    mode::server::AppState,
    objects::structs::HikariConfig,
    server::{
        common::build_hikari_config,
        dal::{
            container_dal::ContainerDAL,
            deploy_config_dal::{DeployConfigDAL, Utils},
            stack_config_dal::StackConfigDAL,
        },
        traits::model::DataRepository,
    },
};

#[derive(Deserialize)]
pub struct QueryParamsMetadata {
    pub client: String,
    pub environment: String,
    pub solution: String,
}

#[derive(Deserialize)]
pub struct QueryParamsName {
    pub name: String,
}

#[debug_handler]
pub async fn get_hikari_by_metadata(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParamsMetadata {
        client,
        environment,
        solution,
    }): Query<QueryParamsMetadata>,
) -> Result<Json<HikariConfig>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let container_dal = ContainerDAL::new(&state.pool);
    let deployments = deploy_config_dal
        .find_by_metadata(&client, &environment, &solution)
        .await
        .map_err(|_err| {
            info!("Unable to fetch deployments");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR".to_string(),
            )
        })?;
    let hikari = build_hikari_config(deployments, stack_config_dal, container_dal).await?;
    Ok(Json(hikari))
}

#[debug_handler]
pub async fn get_hikari_by_name(
    Extension(state): Extension<Arc<AppState>>,
    Query(QueryParamsName { name }): Query<QueryParamsName>,
) -> Result<Json<HikariConfig>, (StatusCode, String)> {
    let deploy_config_dal = DeployConfigDAL::new(&state.pool);
    let stack_config_dal = StackConfigDAL::new(&state.pool);
    let container_dal = ContainerDAL::new(&state.pool);
    let deployment = deploy_config_dal
        .find_by_name(&name)
        .await
        .map_err(|_err| {
            info!("Unable to fetch deployments");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR".to_string(),
            )
        })?;
    let hikari = build_hikari_config(vec![deployment], stack_config_dal, container_dal).await?;
    Ok(Json(hikari))
}
