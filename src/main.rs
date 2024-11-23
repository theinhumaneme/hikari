mod objects;
mod utils;
use std::{fs, thread, time::Duration};

use clap::Parser;
use dotenvy::dotenv;
use objects::structs::NodeConfig;
use utils::{
    cli::{HikariCli, HikariCommands},
    crypto::{decrypt_json, encrypt_json},
    docker_utils::{dry_run_generate_compose, generate_compose},
    error::ConfigError,
    file_utils::download_file,
};

use crate::objects::structs::Validate;

fn load_config(file_path: String) -> Result<NodeConfig, ConfigError> {
    let contents = fs::read_to_string(file_path)?;
    let config: NodeConfig = serde_json::from_str(&contents)?;
    config.validate()?;
    Ok(config)
}
fn main() {
    dotenv().ok();
    let public_key_path: String = std::env::var("PUBLIC_KEY_FILENAME").expect(
        "PUBLIC_KEY_FILENAME must
    be set.",
    );
    let private_key_path: String = std::env::var("PRIVATE_KEY_FILENAME").expect(
        "PRIVATE_KEY_FILENAME must
    be set.",
    );
    let updated_config_file = std::env::var("UPDATED_CONFIG_FILE").expect(
        "UPDATED_CONFIG_FILE must
    be set.",
    );
    let updated_config_file_enc = std::env::var("ENCRYPTED_FILE_PATH").expect(
        "ENCRYPTED_FILE_PATH must
    be set.",
    );
    let updated_config_file_dec = std::env::var("DECRYPTED_FILE_PATH").expect(
        "DECRYPTED_FILE_PATH must
    be set.",
    );
    let file_remote_url = std::env::var("FILE_REMOTE_URL").expect(
        "FILE_REMOTE_URL must
    be set.",
    );
    let poll_interval = std::env::var("POLL_INTERVAL").expect(
        "POLL_INTERVAL must
    be set.",
    );
    let cli = HikariCli::parse();

    match &cli.command {
        HikariCommands::Encrypt {
            input_file,
            output_file,
        } => {
            let _ = encrypt_json(input_file.clone(), output_file.clone(), public_key_path);
        }
        HikariCommands::Decrypt {
            input_file,
            output_file,
        } => {
            let _ = decrypt_json(input_file.clone(), output_file.clone(), private_key_path);
        }
        HikariCommands::DryRun { input_file } => match load_config(input_file.clone()) {
            Ok(config) => {
                for stack in config.deploy_stacks {
                    dry_run_generate_compose(stack.filename, stack.compose_spec);
                }
            }
            Err(e) => {
                eprintln!("Error loading configuration: {}", e);
            }
        },
        HikariCommands::Daemon => loop {
            match download_file(file_remote_url.clone(), updated_config_file_enc.clone()) {
                true => {
                    match decrypt_json(
                        updated_config_file_enc.clone(),
                        updated_config_file.clone(),
                        private_key_path.clone(),
                    ) {
                        Ok(()) => match load_config(updated_config_file_dec.clone()) {
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
                        },
                        _ => {
                            println!("Error Occured")
                        }
                    }
                }
                false => {
                    println!("Unable to Download the file");
                }
            }
            thread::sleep(Duration::from_secs(poll_interval.parse().unwrap()));
        },
    }
}
