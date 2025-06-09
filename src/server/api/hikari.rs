use std::{collections::HashMap, sync::Arc};

use axum::{Extension, Json, debug_handler, extract::Query};
use log::info;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    mode::server::AppState,
    objects::structs::{ComposeSpec, Container, DeployConfig, HikariConfig, StackConfig, Validate},
    server::{
        dal::{
            container_dal::ContainerDAL,
            deploy_config_dal::{DeployConfigDAL, Utils},
            stack_config_dal::StackConfigDAL,
        },
        models::deploy_config::DeployConfigDTO,
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

pub async fn build_hikari_config(
    deployments: Vec<DeployConfigDTO>,
    stack_config_dal: StackConfigDAL,
    container_dal: ContainerDAL,
) -> Result<HikariConfig, (StatusCode, String)> {
    let mut deploy_configs: HashMap<String, DeployConfig> = HashMap::new();
    for deploy_config_dto in deployments {
        let mut deploy_stacks: Vec<StackConfig> = Vec::new();
        if let Some(stack_ids) = deploy_config_dto.stack_ids.clone() {
            for stack_id in stack_ids {
                let stack_config_dto =
                    stack_config_dal
                        .find_by_id(stack_id)
                        .await
                        .map_err(|_err| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "INTERNAL SERVER ERROR".to_string(),
                            )
                        })?;
                let mut services: HashMap<String, Container> = HashMap::new();
                if let Some(container_ids) = stack_config_dto.containers.clone() {
                    for container_id in container_ids {
                        let container_dto =
                            container_dal
                                .find_by_id(container_id)
                                .await
                                .map_err(|_err| {
                                    (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        "INTERNAL SERVER ERROR".to_string(),
                                    )
                                })?;
                        let container: Container = container_dto.clone().into();
                        services.insert(container_dto.service_name, container);
                    }
                }
                deploy_stacks.push(StackConfig {
                    stack_name: stack_config_dto.stack_name,
                    filename: stack_config_dto.filename,
                    home_directory: stack_config_dto.home_directory,
                    compose_spec: ComposeSpec { services },
                });
            }
        }
        deploy_configs.insert(
            deploy_config_dto.name.clone(),
            DeployConfig {
                client: deploy_config_dto.client.clone(),
                environment: deploy_config_dto.client.clone(),
                solution: deploy_config_dto.client.clone(),
                deploy_stacks,
            },
        );
    }
    let hikari = HikariConfig {
        version: "1".to_string(),
        deploy_configs,
    };
    hikari
        .validate()
        .map_err(|_err| (StatusCode::BAD_REQUEST, _err.to_string()))?;
    Ok(hikari)
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
