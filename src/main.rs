mod objects;
mod utils;
use std::{collections::HashSet, fs, path::Path, process::exit, thread, time::Duration};

use clap::Parser;
use dotenvy::dotenv;
use objects::structs::{HikariConfig, MainConfig, UpdateOptions};
use serde_json::json;
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

fn manage_configs(
    current_config: &HikariConfig,
    incoming_config: &HikariConfig,
    client: String,
    environment: String,
    solution: String,
) {
    for (key, current_deploy_config) in &current_config.deploy_configs {
        // find the incoming config w.r.t to the current config
        if let Some(incoming_deploy_config) = &incoming_config.deploy_configs.get(key) {
            if current_deploy_config.client == client
                && current_deploy_config.environment == environment
                && current_deploy_config.solution == solution
            {
                if incoming_deploy_config.client == client
                    && incoming_deploy_config.environment == environment
                    && incoming_deploy_config.solution == solution
                {
                    for (current_stack, incoming_stack) in current_deploy_config
                        .deploy_stacks
                        .iter()
                        .zip(incoming_deploy_config.deploy_stacks.iter())
                    {
                        // detect if the stacks are different
                        // stop the compose
                        // generate the new compose file from spec
                        // start the new compose file
                        if current_stack != incoming_stack {
                            println!("Changes detected in {}", current_stack.stack_name.clone());
                            match stop_compose(format!(
                                "{}/{}",
                                current_stack.home_directory, current_stack.filename
                            )) {
                                true => {
                                    println!(
                                        "Stack {} was successfully stopped",
                                        current_stack.stack_name
                                    );
                                    let compose_path = generate_compose(
                                        incoming_stack.home_directory.clone(),
                                        incoming_stack.stack_name.clone(),
                                        incoming_stack.filename.clone(),
                                        incoming_stack.compose_spec.clone(),
                                    );
                                    match start_compose(compose_path.clone()) {
                                        true => {
                                            println!(
                                                "Stack {} was successfully started ",
                                                incoming_stack.stack_name
                                            )
                                        }
                                        false => {
                                            println!(
                                                "Stack {} could not be started",
                                                incoming_stack.stack_name
                                            )
                                        }
                                    }
                                }
                                false => {
                                    println!(
                                        "Stack {} could not be stopped",
                                        incoming_stack.stack_name
                                    )
                                }
                            };
                        } else {
                            println!("No changes detected in {}", current_stack.stack_name);
                        }
                    }
                    // check if any stack has been deleted
                    let current_stacks: HashSet<_> = current_deploy_config
                        .deploy_stacks
                        .iter()
                        .map(|stack| &stack.stack_name)
                        .collect();
                    let incoming_stacks: HashSet<_> = incoming_deploy_config
                        .deploy_stacks
                        .iter()
                        .map(|stack| &stack.stack_name)
                        .collect();
                    let removed_stacks = current_stacks.difference(&incoming_stacks);
                    for stack_name in removed_stacks {
                        if let Some(stack) = current_deploy_config
                            .deploy_stacks
                            .iter()
                            .find(|s| s.stack_name == **stack_name)
                        {
                            match stop_compose(format!(
                                "{}/{}",
                                stack.home_directory, stack.filename
                            )) {
                                true => println!(
                                    "Successfully stopped removed stack {}",
                                    stack.stack_name
                                ),
                                false => {
                                    println!("Could not stop removed stack {}", stack.stack_name)
                                }
                            }
                        }
                    }
                    let added_stacks = incoming_stacks.difference(&current_stacks);
                    for stack_name in added_stacks {
                        if let Some(stack) = current_deploy_config
                            .deploy_stacks
                            .iter()
                            .find(|s| s.stack_name == **stack_name)
                        {
                            let compose_path = generate_compose(
                                stack.home_directory.clone(),
                                stack.stack_name.clone(),
                                stack.stack_name.clone(),
                                stack.compose_spec.clone(),
                            );
                            match start_compose(compose_path) {
                                true => println!(
                                    "Successfully started added stack {}",
                                    stack.stack_name
                                ),
                                false => {
                                    println!("Could not start added stack {}", stack.stack_name)
                                }
                            }
                        }
                    }
                }
            }
        } else if current_deploy_config.client == client
            && current_deploy_config.environment == environment
            && current_deploy_config.solution == solution
        {
            println!("Deploy Config {} has been removed", key);
            for stack in &current_deploy_config.deploy_stacks {
                match stop_compose(format!("{}/{}", stack.home_directory, stack.filename)) {
                    true => println!("Successfully stopped removed stack {}", stack.stack_name),
                    false => {
                        println!("Could not stop removed stack {}", stack.stack_name)
                    }
                }
            }
        }
    }
    for (key, incoming_deploy_config) in &incoming_config.deploy_configs {
        if !current_config.deploy_configs.contains_key(key) {
            if incoming_deploy_config.environment == environment
                && incoming_deploy_config.solution == solution
                && incoming_deploy_config.client == client
            {
                println!("New NodeConfig '{}' found in incoming config, creating all associated stacks...", key);
                for stack in &incoming_deploy_config.deploy_stacks {
                    let compose_path = generate_compose(
                        stack.home_directory.clone(),
                        stack.stack_name.clone(),
                        stack.filename.clone(),
                        stack.compose_spec.clone(),
                    );
                    match start_compose(compose_path) {
                        true => println!("Successfully started added stack {}", stack.stack_name),
                        false => {
                            println!("Could not start added stack {}", stack.stack_name)
                        }
                    }
                }
            }
        }
    }
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
                                            manage_configs(
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
