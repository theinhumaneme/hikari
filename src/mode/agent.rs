use futures::StreamExt;
use log::{error, info};
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    objects::structs::{HikariConfig, NodeConfig, NodeUpdateOptions},
    utils::{
        config::load_hikari_config,
        file_utils::{load_config_from_url, write_file},
        manage::manage_node,
        secrets::load_secrets,
    },
};

pub async fn configuration_init(
    node_config: &NodeConfig,
    node_update_config: &NodeUpdateOptions,
    host: String,
) {
    let mut incoming_config: HikariConfig = HikariConfig::default();
    match load_config_from_url(
        format!(
            "https://{}/api/v1/hikari/metadata?client={}&environment={}&solution={}",
            host, node_config.client, node_config.environment, node_config.solution
        )
        .as_str(),
    )
    .await
    {
        Ok(reference) => incoming_config = reference,
        Err(e) => {
            error!("Error loading initial configuration: {e}");
        }
    }

    match load_hikari_config(&node_update_config.reference_file_path) {
        Ok(reference) => {
            manage_node(
                &reference,
                &incoming_config,
                &node_config.client,
                &node_config.environment,
                &node_config.solution,
            );
            write_file(
                serde_json::to_string(&incoming_config)
                    .expect("Failed to serialize JSON")
                    .as_str(),
                &node_update_config.reference_file_path,
            );
        }
        Err(e) => {
            error!("Error loading reference configuration: {e}");
        }
    }
}

pub async fn agent_mode(node_config: &NodeConfig, node_update_config: &NodeUpdateOptions) {
    let secrets = load_secrets("agent");
    let host = secrets[0].clone();

    configuration_init(node_config, node_update_config, host.clone()).await;
    const MAX_BACKOFF: u64 = 64;
    let mut backoff: u64 = 1;

    loop {
        match connect_async(
            format!(
                "ws://{}/ws?client={}&environment={}&solution={}",
                host.clone(),
                node_config.client,
                node_config.environment,
                node_config.solution
            )
            .as_str(),
        )
        .await
        {
            Ok((ws_stream, _)) => {
                info!("Connected to {}", host.clone());
                backoff = 1;
                let (mut _ws_tx, mut ws_rx) = ws_stream.split();

                while let Some(msg_res) = ws_rx.next().await {
                    match msg_res {
                        Ok(message) => match message {
                            Message::Text(txt_bytes) => {
                                let text = txt_bytes.as_str();
                                if text == "DEPLOYMENT UPDATED" {
                                    info!("{text}");
                                    configuration_init(
                                        node_config,
                                        node_update_config,
                                        host.clone(),
                                    )
                                    .await;
                                }
                            }
                            Message::Binary(_bin) => { /* ignore */ }
                            Message::Ping(_) | Message::Pong(_) => { /* ignore heartbeats */ }
                            Message::Close(_) => {
                                error!("Server Closed Connection");
                                break;
                            }
                            _ => {}
                        },
                        Err(e) => {
                            error!("WebSocket receive error: {e}");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("WebSocket connection failed: {e}");
            }
        }

        info!("Reconnecting in {backoff} seconds");
        sleep(Duration::from_secs(backoff)).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}
