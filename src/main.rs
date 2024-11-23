mod objects;
mod utils;
use std::{fs, path::Path, process::exit, thread, time::Duration};

use clap::Parser;
use dotenvy::dotenv;
use objects::structs::{HikariConfig, MainConfig, NodeConfig};
use utils::{
    cli::{HikariCli, HikariCommands},
    crypto::{decrypt_json, encrypt_json},
    docker_utils::{dry_run_generate_compose, generate_compose, start_compose, stop_compose},
    error::ConfigError,
    file_utils::{copy_file, download_file},
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
    let reference_config_file = std::env::var("REFERENCE_CONFIG_FILE").expect(
        "REFERENCE_FILE must
    be set.",
    );

    let mut node_config: MainConfig = Default::default();
    if Path::exists(Path::new("config.toml")) {
        node_config = match toml::from_str(fs::read_to_string("config.toml").unwrap().as_str()) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Could not load the `config.toml` file ");
                exit(1);
            }
        };
    } else {
        eprintln!("`config.toml` file does not exist")
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
            match download_file(file_remote_url.clone(), updated_config_file_enc.clone()) {
                true => {
                    match decrypt_json(
                        updated_config_file_enc.clone(),
                        updated_config_file_dec.clone(),
                        private_key_path.clone(),
                    ) {
                        Ok(()) => match load_config(updated_config_file_dec.clone()) {
                            Ok(config) => {
                                if config.version.trim() == node_config.version {
                                    for deploy_config in &config.deploy_configs {
                                        if (&node_config.client.trim()
                                            == &deploy_config.1.client.trim()
                                            && &node_config.solution.trim()
                                                == &deploy_config.1.solution.trim()
                                            && &node_config.environment.trim()
                                                == &deploy_config.1.environment.trim())
                                        {
                                            for stack in deploy_config.1.deploy_stacks.clone() {
                                                let compose_path: String = generate_compose(
                                                    stack.home_directory,
                                                    stack.stack_name.clone(),
                                                    stack.filename,
                                                    stack.compose_spec,
                                                );
                                                match stop_compose(compose_path.clone()) {
                                                    true => {
                                                        println!(
                                                            "Stack {} was successfully stopped",
                                                            stack.stack_name
                                                        );
                                                        match start_compose(compose_path.clone()) {
                                                            true => {
                                                                println!(
                                                                    "Stack {} was successfully started ",
                                                                    stack.stack_name
                                                                )
                                                            }
                                                            false => {
                                                                println!(
                                                                    "Stack {} could not be started",
                                                                    stack.stack_name
                                                                )
                                                            }
                                                        }
                                                    }
                                                    false => {
                                                        println!(
                                                            "Stack {} could not be stopped",
                                                            stack.stack_name
                                                        )
                                                    }
                                                };
                                            }
                                        } else {
                                            continue;
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
            thread::sleep(Duration::from_secs(poll_interval.parse().unwrap()));
        },
    }
}
