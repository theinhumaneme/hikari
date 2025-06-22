use std::{fs, path::Path};

use log::info;
use serde_json::json;

use super::error::ConfigError;
use crate::objects::structs::{HikariConfig, NodeConfig, NodeUpdateOptions, Validate};

pub fn load_config() -> Result<(NodeConfig, NodeUpdateOptions), ConfigError> {
    let node_config: NodeConfig = {
        let contents = fs::read_to_string("node.toml").map_err(ConfigError::FileError)?;
        toml::from_str(&contents).map_err(ConfigError::TomlParseError)?
    };

    let node_update_config: NodeUpdateOptions = {
        let contents = fs::read_to_string("config.toml").map_err(ConfigError::FileError)?;
        toml::from_str(&contents).map_err(ConfigError::TomlParseError)?
    };
    if !Path::new(&node_update_config.reference_file_path).exists() {
        info!(
            "Looks like hikari is being installed here, generating placeholder {}",
            &node_update_config.reference_file_path
        );
        let config = json!({
            "version": "1",
            "deploy_configs": {}
        });
        let json_data = serde_json::to_string_pretty(&config).expect("Failed to serialize JSON");
        fs::write(&node_update_config.reference_file_path, json_data)
            .expect("Unable to write file");
    }
    Ok((node_config, node_update_config))
}

pub fn load_hikari_config(file_path: &str) -> Result<HikariConfig, ConfigError> {
    let contents = fs::read_to_string(file_path)?;
    let config: HikariConfig = serde_json::from_str(&contents)?;
    config.validate()?;
    Ok(config)
}
