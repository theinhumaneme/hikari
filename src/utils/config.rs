use std::{fs, path::Path, process::exit};

use serde_json::json;

use super::error::ConfigError;
use crate::objects::structs::{HikariConfig, NodeConfig, NodeUpdateOptions, Validate};

pub fn load_config() -> (NodeConfig, NodeUpdateOptions) {
    let mut node_config: NodeConfig = Default::default();
    if Path::exists(Path::new("node.toml")) {
        node_config = match toml::from_str(fs::read_to_string("node.toml").unwrap().as_str()) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Could not load the `node.toml` file ");
                exit(1);
            }
        };
    } else {
        eprintln!("`node.toml` file does not exist")
    }
    let mut node_update_config: NodeUpdateOptions = Default::default();
    if Path::exists(Path::new("config.toml")) {
        node_update_config =
            match toml::from_str(fs::read_to_string("config.toml").unwrap().as_str()) {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("Could not load the `config.toml` file ");
                    exit(1);
                }
            };
    } else {
        eprintln!("`config.toml` file does not exist")
    }
    if !Path::new(&node_update_config.reference_file_path).exists() {
        println!(
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
    return (node_config, node_update_config);
}

pub fn load_hikari_config(file_path: &str) -> Result<HikariConfig, ConfigError> {
    let contents = fs::read_to_string(file_path)?;
    let config: HikariConfig = serde_json::from_str(&contents)?;
    config.validate()?;
    Ok(config)
}
