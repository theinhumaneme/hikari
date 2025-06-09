use std::collections::HashMap;

use log::error;
use reqwest::StatusCode;
use sqlx::Error;

use crate::{
    objects::structs::{ComposeSpec, Container, DeployConfig, HikariConfig, StackConfig, Validate},
    server::{
        dal::{container_dal::ContainerDAL, stack_config_dal::StackConfigDAL},
        models::deploy_config::DeployConfigDTO,
        traits::model::DataRepository,
    },
};

pub fn map_db_error(e: Error) -> (StatusCode, String) {
    error!("{}", e);
    match e {
        Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                let c = db_err.constraint().unwrap_or("unknown");
                return (
                    StatusCode::CONFLICT,
                    format!("Duplicate entry: `{}` constraint", c),
                );
            }
            if db_err.is_foreign_key_violation() {
                let c = db_err.constraint().unwrap_or("unknown");
                return (
                    StatusCode::CONFLICT,
                    format!("Foreign key violation: `{}` constraint", c),
                );
            }
            if db_err.is_check_violation() {
                let c = db_err.constraint().unwrap_or("unknown");
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Check violation: `{}` constraint", c),
                );
            }
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into());
        }
        Error::RowNotFound => (StatusCode::NOT_FOUND, "Record not found".into()),

        Error::Io(err) => (StatusCode::SERVICE_UNAVAILABLE, err.to_string()),

        Error::Protocol(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),

        Error::Tls(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),

        Error::PoolTimedOut => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Connection timed out".into(),
        ),

        Error::TypeNotFound { type_name } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Type not found: {}", type_name),
        ),
        Error::ColumnNotFound(col) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Column not found: {}", col),
        ),
        Error::ColumnIndexOutOfBounds { index, len } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Column index out of bounds: {}/{}", index, len),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL SERVER ERROR".into(),
        ),
    }
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
                let stack_config_dto = stack_config_dal
                    .find_by_id(stack_id)
                    .await
                    .map_err(map_db_error)?;
                let mut services: HashMap<String, Container> = HashMap::new();
                if let Some(container_ids) = stack_config_dto.containers.clone() {
                    for container_id in container_ids {
                        let container_dto = container_dal
                            .find_by_id(container_id)
                            .await
                            .map_err(map_db_error)?;
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
