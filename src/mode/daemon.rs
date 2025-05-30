use std::{process::exit, thread, time::Duration};

use crate::{
    objects::structs::{MainConfig, UpdateOptions},
    utils::{
        config::load_hikari_config,
        crypto::decrypt_json,
        file_utils::{copy_file, download_file},
        manage::manage_node,
    },
};

pub fn daemon_mode(
    node_config: &MainConfig,
    update_config: &UpdateOptions,
    private_key_path: &str,
) {
    match download_file(
        &update_config.remote_url,
        &update_config.encrypted_file_path,
    ) {
        true => {
            match decrypt_json(
                &update_config.encrypted_file_path,
                &update_config.decrypted_file_path,
                &private_key_path,
            ) {
                Ok(()) => match load_hikari_config(&update_config.decrypted_file_path) {
                    Ok(config) => {
                        if config.version.trim() == node_config.version {
                            match load_hikari_config(&update_config.reference_file_path) {
                                Ok(reference) => {
                                    manage_node(
                                        &reference,
                                        &config,
                                        &node_config.client,
                                        &node_config.environment,
                                        &node_config.solution,
                                    );
                                    let _ = copy_file(
                                        &update_config.decrypted_file_path,
                                        &update_config.reference_file_path,
                                    );
                                }
                                Err(e) => {
                                    eprintln!("Error loading reference configuration: {}", e);
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
}
