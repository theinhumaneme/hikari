use std::{process::exit, thread, time::Duration};

use log::error;

use crate::{
    objects::structs::{NodeConfig, NodeUpdateOptions},
    utils::{
        config::load_hikari_config,
        crypto::decrypt_json,
        error::ConfigError,
        file_utils::{copy_file, download_file},
        manage::manage_node,
    },
};

pub fn daemon_mode(
    node_config: &NodeConfig,
    node_update_config: &NodeUpdateOptions,
    private_key_path: &str,
) -> Result<(), ConfigError> {
    let remote_url = if let Some(val) = &node_update_config.remote_url {
        val
    } else {
        return Err(ConfigError::MissingField("remote_url".into()));
    };

    let encrypted_file_path = if let Some(val) = &node_update_config.encrypted_file_path {
        val
    } else {
        return Err(ConfigError::MissingField("encrypted_file_path".into()));
    };

    let decrypted_file_path = if let Some(val) = &node_update_config.decrypted_file_path {
        val
    } else {
        return Err(ConfigError::MissingField("decrypted_file_path".into()));
    };

    let poll_interval = if let Some(val) = &node_update_config.poll_interval {
        val
    } else {
        return Err(ConfigError::MissingField("poll_interval".into()));
    };

    match download_file(remote_url, encrypted_file_path) {
        Ok(true) => {
            match decrypt_json(encrypted_file_path, decrypted_file_path, private_key_path) {
                Ok(()) => match load_hikari_config(decrypted_file_path) {
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
                                        decrypted_file_path,
                                        &node_update_config.reference_file_path,
                                    );
                                }
                                Err(e) => {
                                    error!("Error loading reference configuration: {e}");
                                }
                            }
                        } else {
                            error!(
                                " Incoming `Hikari Config` version does not match with `Node` version`"
                            );
                            error!(
                                " Incoming `Hikari Config` version does not match with `Node` version`"
                            );
                            exit(1);
                        }
                    }
                    Err(e) => {
                        error!("Error loading configuration: {e}");
                    }
                },
                _ => {
                    error!("Error Occured")
                }
            }
        }
        Ok(false) => {
            error!("Unable to Download the file");
        }
        _ => {}
    }
    if let Ok(poll_secs) = poll_interval.parse::<u64>() {
        thread::sleep(Duration::from_secs(poll_secs));
    } else {
        error!("Invalid poll_interval value");
    }

    Ok(())
}
