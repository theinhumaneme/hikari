mod objects;
mod utils;
use std::{fs, path::Path, process::exit, thread, time::Duration};

use clap::Parser;
use dotenvy::dotenv;
use objects::structs::{HikariConfig, MainConfig, UpdateOptions};
use serde_json::json;
use utils::{
    cli::{HikariCli, HikariCommands},
    crypto::{decrypt_json, encrypt_json},
    docker_utils::dry_run_generate_compose,
    error::ConfigError,
    file_utils::{copy_file, download_file},
    manage::manage_node,
};

use crate::objects::structs::Validate;

fn load_config(file_path: String) -> Result<HikariConfig, ConfigError> {
    let contents = fs::read_to_string(file_path)?;
    let config: HikariConfig = serde_json::from_str(&contents)?;
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

    let mut node_config: MainConfig = Default::default();
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
    let mut update_config: UpdateOptions = Default::default();
    if Path::exists(Path::new("config.toml")) {
        update_config = match toml::from_str(fs::read_to_string("config.toml").unwrap().as_str()) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Could not load the `config.toml` file ");
                exit(1);
            }
        };
    } else {
        eprintln!("`config.toml` file does not exist")
    }
    if !Path::new(&update_config.reference_file_path.clone()).exists() {
        println!(
            "Looks like hikari is being installed here, generating placeholder {}",
            &update_config.reference_file_path.clone()
        );
        let config = json!({
            "version": "1",
            "deploy_configs": {}
        });
        let json_data = serde_json::to_string_pretty(&config).expect("Failed to serialize JSON");
        fs::write(&update_config.reference_file_path.clone(), json_data)
            .expect("Unable to write file");
    }

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
                for deploy_config in config.deploy_configs {
                    for stack in deploy_config.1.deploy_stacks {
                        dry_run_generate_compose(stack.filename, stack.compose_spec);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error loading configuration: {}", e);
            }
        },
        HikariCommands::Daemon => loop {
            match download_file(
                update_config.remote_url.clone(),
                update_config.encrypted_file_path.clone(),
            ) {
                true => {
                    match decrypt_json(
                        update_config.encrypted_file_path.clone(),
                        update_config.decrypted_file_path.clone(),
                        private_key_path.clone(),
                    ) {
                        Ok(()) => match load_config(update_config.decrypted_file_path.clone()) {
                            Ok(config) => {
                                if config.version.trim() == node_config.version {
                                    match load_config(update_config.reference_file_path.clone()) {
                                        Ok(reference) => {
                                            manage_node(
                                                &reference,
                                                &config,
                                                node_config.client.clone(),
                                                node_config.environment.clone(),
                                                node_config.solution.clone(),
                                            );
                                            let _ = copy_file(
                                                update_config.decrypted_file_path.clone(),
                                                update_config.reference_file_path.clone(),
                                            );
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error loading reference configuration: {}",
                                                e
                                            );
                                        }
                                    }
                                } else {
                                    eprintln!(" Incoming `Hikari Config` version does not match with `Node` version`");
                                    exit(1);
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
            thread::sleep(Duration::from_secs(
                update_config.poll_interval.parse().unwrap(),
            ));
        },
    }
}
