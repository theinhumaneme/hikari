mod mode;
mod objects;
mod utils;

use clap::Parser;

use mode::daemon::daemon_mode;
use utils::{
    cli::{HikariCli, HikariCommands},
    config::{load_config, load_hikari_config},
    crypto::{decrypt_json, encrypt_json},
    docker_utils::dry_run_generate_compose,
    secrets::load_secrets,
};

fn main() {
    let cli = HikariCli::parse();

    match &cli.command {
        HikariCommands::Encrypt {
            input_file,
            output_file,
        } => {
            let (public_key_path, _private_key_path) = load_secrets();
            let _ = encrypt_json(input_file, output_file, &public_key_path);
        }
        HikariCommands::Decrypt {
            input_file,
            output_file,
        } => {
            let (_public_key_path, private_key_path) = load_secrets();
            let _ = decrypt_json(input_file, output_file, &private_key_path);
        }
        HikariCommands::DryRun { input_file } => match load_hikari_config(input_file) {
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
            let (main_config, update_options) = load_config();
            let (_public_key_path, private_key_path) = load_secrets();
            daemon_mode(&main_config, &update_options, &private_key_path);
        },
    }
}
