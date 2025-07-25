use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{server::models::container::ContainerDTO, utils::error::ConfigError};

pub trait Validate {
    fn validate(&self) -> Result<(), ConfigError>;
}

macro_rules! validate_field {
    ($field:expr, $field_name:expr) => {
        if $field.is_empty() {
            return Err(ConfigError::MissingField($field_name.to_string()));
        }
    };
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeConfig {
    pub version: String,
    pub solution: String,
    pub client: String,
    pub environment: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeUpdateOptions {
    pub remote_url: Option<String>,
    pub poll_interval: Option<String>,
    pub encrypted_file_path: Option<String>,
    pub decrypted_file_path: Option<String>,
    pub reference_file_path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HikariConfig {
    pub version: String,
    pub deploy_configs: HashMap<String, DeployConfig>,
}
impl Validate for HikariConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_field!(self.version, "version");

        for (index, config) in self.deploy_configs.iter().enumerate() {
            config
                .1
                .validate()
                .map_err(|e| ConfigError::MissingField(format!("deploy_configs[{index}]: {e}")))?;
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployConfig {
    pub client: String,
    pub environment: String,
    pub solution: String,
    pub deploy_stacks: Vec<StackConfig>,
}
impl Validate for DeployConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_field!(self.client, "client");
        validate_field!(self.environment, "environment");
        validate_field!(self.solution, "solution");

        if self.deploy_stacks.is_empty() {
            return Err(ConfigError::MissingField("deploy_stacks".to_string()));
        }

        for (index, stack) in self.deploy_stacks.iter().enumerate() {
            stack
                .validate()
                .map_err(|e| ConfigError::MissingField(format!("deploy_stacks[{index}]: {e}")))?;
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackConfig {
    pub stack_name: String,
    pub filename: String,
    pub home_directory: String,
    pub compose_spec: ComposeSpec,
}

impl Validate for StackConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_field!(self.stack_name, "name");
        validate_field!(self.filename, "filename");
        validate_field!(self.home_directory, "home_directory");
        self.compose_spec.validate()?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComposeSpec {
    pub services: HashMap<String, Container>,
}

impl Validate for ComposeSpec {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.services.is_empty() {
            return Err(ConfigError::MissingField(
                "compose_spec.services".to_string(),
            ));
        }
        for (name, service) in &self.services {
            service
                .validate()
                .map_err(|e| ConfigError::MissingField(format!("service[{name}]: {e}")))?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Container {
    pub container_name: String,
    pub image: String,
    pub restart: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_reservation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oom_kill_disable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privileged: Option<bool>,
}

impl Validate for Container {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_field!(self.container_name, "container_name");
        validate_field!(self.image, "image");
        validate_field!(self.restart, "restart");
        Ok(())
    }
}
impl From<ContainerDTO> for Container {
    fn from(dto: ContainerDTO) -> Self {
        Self {
            container_name: dto.container_name,
            image: dto.image,
            restart: dto.restart,
            user: dto.user,
            stdin_open: dto.stdin_open,
            tty: dto.tty,
            command: dto.command,
            pull_policy: dto.pull_policy,
            ports: dto.ports,
            volumes: dto.volumes,
            environment: dto.environment,
            mem_reservation: dto.mem_reservation,
            mem_limit: dto.mem_limit,
            oom_kill_disable: dto.oom_kill_disable,
            privileged: dto.privileged,
        }
    }
}
