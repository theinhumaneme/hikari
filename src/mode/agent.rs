use futures::StreamExt;
use log::info;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    objects::structs::{HikariConfig, NodeConfig},
    utils::{file_utils::load_config_from_url, secrets::load_secrets},
};

pub async fn agent_mode(node_config: &NodeConfig) {
    let secrets = load_secrets("agent");
    let host = secrets[0].clone();

    let hikari_config: HikariConfig = load_config_from_url(
        format!(
            "http://{}:3000/api/v1/hikari/metadata?client={}&environment={}&solution={}",
            host, node_config.client, node_config.environment, node_config.solution
        )
        .as_str(),
    )
    .await
    .unwrap();
    dbg!(hikari_config);

    let (ws_stream, _) = connect_async(
        format!(
            "ws://{}:3000/ws?client={}&environment={}&solution={}",
            host, node_config.client, node_config.environment, node_config.solution
        )
        .as_str(),
    )
    .await
    .expect("failed to connect to websocket");

    println!("Connected to {}", secrets[0]);

    let (mut _ws_tx, mut ws_rx) = ws_stream.split();

    while let Some(Ok(message)) = ws_rx.next().await {
        match message {
            Message::Text(txt_bytes) => {
                let txt = txt_bytes.to_string();
                info!("{txt}");
                // parse `txt` below…
            }
            Message::Binary(bin) => { /* ignore */ }
            Message::Ping(_) | Message::Pong(_) => { /* ignore heartbeats */ }
            Message::Close(frame) => {
                println!("Server Closed Connection");
                break;
            }
            _ => {}
        }
    }

    // …now you can split/ws_stream, send/receive messages, etc…
}
