use std::{process::exit, thread, time::Duration};

use log::error;

use crate::{
    objects::structs::{NodeConfig, NodeUpdateOptions},
    utils::{
        config::load_hikari_config,
        crypto::decrypt_json,
        file_utils::{copy_file, download_file},
        manage::manage_node,
    },
};

pub fn daemon_mode(
    node_config: &NodeConfig,
    node_update_config: &NodeUpdateOptions,
    private_key_path: &str,
) {
    if node_update_config.encrypted_file_path.is_none() {
        error!("`encrypted_file_path` variable is missing in `config.toml`");
    }
    if node_update_config.decrypted_file_path.is_none() {
        error!("`decrypted_file_path` variable is missing in `config.toml`");
    }
    if node_update_config.poll_interval.is_none() {
        error!("`poll_interval` variable is missing in `config.toml`");
    }
    match download_file(
        &node_update_config.remote_url.clone().unwrap(),
        &node_update_config.encrypted_file_path.clone().unwrap(),
    ) {
        true => {
            match decrypt_json(
                &node_update_config.encrypted_file_path.clone().unwrap(),
                &node_update_config.decrypted_file_path.clone().unwrap(),
                private_key_path,
            ) {
                Ok(()) => {
                    match load_hikari_config(
                        &node_update_config.decrypted_file_path.clone().unwrap(),
                    ) {
                        Ok(config) => {
                            if config.version.trim() == node_config.version {
                                match load_hikari_config(&node_update_config.reference_file_path) {
                                    Ok(reference) => {
                                        manage_node(
                                            &reference,
                                            &config,
                                            &node_config.client,
                                            &node_config.environment,
                                            &node_config.solution,
                                        );
                                        copy_file(
                                            &node_update_config
                                                .decrypted_file_path
                                                .clone()
                                                .unwrap(),
                                            &node_update_config.reference_file_path,
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("Error loading reference configuration: {e}");
                                    }
                                }
                            } else {
                                eprintln!(
                                    " Incoming `Hikari Config` version does not match with `Node` version`"
                                );
                                exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error loading configuration: {e}");
                        }
                    }
                }
                _ => {
                    println!("Error Occured")
                }
            }
        }
        false => {
            println!("Unable to Download the file");
        }
    }
    let poll_interval = &node_update_config.poll_interval.as_ref().unwrap();
    thread::sleep(Duration::from_secs(poll_interval.parse().unwrap()));
}
