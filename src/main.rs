mod mode;
mod objects;
mod server;
mod utils;

use clap::Parser;
use log::{error, info};
use mode::{daemon::daemon_mode, server::server_mode};
use utils::{
    cli::{HikariCli, HikariCommands},
    config::{load_config, load_hikari_config},
    crypto::{decrypt_json, encrypt_json},
    docker_utils::dry_run_generate_compose,
    error::ConfigError,
    secrets::load_secrets,
};

use crate::mode::agent::agent_mode;

#[tokio::main]
async fn main() -> Result<(), ConfigError> {
    let _ = log4rs::init_file("log4rs.yaml", Default::default());
    info!("Hikari Booting Up!");
    let (main_config, update_options) = load_config()?;
    let cli = HikariCli::parse();

    match &cli.command {
        HikariCommands::Encrypt {
            input_file,
            output_file,
        } => {
            let keys = load_secrets("daemon")?;
            let _ = encrypt_json(input_file, output_file, &keys[0]);
        }
        HikariCommands::Decrypt {
            input_file,
            output_file,
        } => {
            let keys = load_secrets("daemon")?;
            let _ = decrypt_json(input_file, output_file, &keys[1]);
        }
        HikariCommands::DryRun { input_file } => match load_hikari_config(input_file) {
            Ok(config) => {
                for deploy_config in config.deploy_configs {
                    for stack in deploy_config.1.deploy_stacks {
                        if let Err(e) = dry_run_generate_compose(stack.filename, stack.compose_spec)
                        {
                            error!("Failed to generate compose for {}: {e}", stack.stack_name);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error loading configuration: {e}");
            }
        },
        HikariCommands::Daemon => loop {
            let keys = load_secrets("daemon")?;
            if let Err(err) = daemon_mode(&main_config, &update_options, &keys[1]).await {
                error!("{err}");
                break;
            }
        },
        HikariCommands::Server => {
            server_mode().await?;
        }
        HikariCommands::Agent => agent_mode(&main_config, &update_options).await?,
    }

    Ok(())
}
