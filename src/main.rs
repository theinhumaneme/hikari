mod objects;
mod utils;
use std::fs;

use dotenvy::dotenv;
use objects::structs::NodeConfig;
use utils::{
    crypto::{decrypt_json, encrypt_json},
    docker_utils::generate_compose,
    error::ConfigError,
};

use crate::objects::structs::Validate;

fn load_config(file_path: &str) -> Result<NodeConfig, ConfigError> {
    let contents = fs::read_to_string(file_path)?;
    let config: NodeConfig = serde_json::from_str(&contents)?;
    config.validate()?;
    Ok(config)
}
fn main() {
    dotenv().ok();
    let encrypted_file_path: String =
        std::env::var("ENCRYPTED_FILE_PATH").expect("ENCRYPTED_FILE_PATH must be set.");
    let decrypted_file_path: String =
        std::env::var("DECRYPTED_FILE_PATH").expect("DECRYPTED_FILE_PATH must be set.");

    let _ = encrypt_json(
        "test.json".to_string(),
        encrypted_file_path.clone(),
        std::env::var("PUBLIC_KEY_FILENAME").expect("PUBLIC_KEY_FILENAME must be set."),
    );
    let _ = decrypt_json(
        encrypted_file_path.clone(),
        decrypted_file_path.clone(),
        std::env::var("PRIVATE_KEY_FILENAME").expect("PRIVATE_KEY_FILENAME must be set."),
    );
    match load_config("test.json") {
        Ok(config) => {
            println!("Configuration loaded successfully: {:#?}", config);
            for stack in config.deploy_stacks {
                generate_compose(
                    stack.home_directory,
                    stack.stack_name,
                    stack.filename,
                    stack.compose_spec,
                );
            }
        }
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
        }
    }
}
