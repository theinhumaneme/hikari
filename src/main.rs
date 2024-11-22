mod objects;
mod utils;
use std::fs;

use objects::structs::NodeConfig;
use utils::error::ConfigError;

use crate::objects::structs::Validate;

fn load_config(file_path: &str) -> Result<NodeConfig, ConfigError> {
    let contents = fs::read_to_string(file_path)?;
    let config: NodeConfig = serde_json::from_str(&contents)?;
    config.validate()?;
    Ok(config)
}
fn main() {
    match load_config("test.json") {
        Ok(config) => {
            println!("Configuration loaded successfully: {:#?}", config);
        }
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
        }
    }
}
