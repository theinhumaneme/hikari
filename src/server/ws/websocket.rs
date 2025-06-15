use std::sync::Arc;

use axum::{
    Extension, debug_handler,
    extract::{
        Query, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::SinkExt;
use futures::StreamExt;
use log::info;
use serde::Deserialize;
use tokio::sync::broadcast::{channel, error::SendError};

use crate::mode::server::AppState;

#[derive(Deserialize)]
pub struct QueryParamsWS {
    client: String,
    solution: String,
    environment: String,
}

#[debug_handler]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(QueryParamsWS {
        client,
        solution,
        environment,
    }): Query<QueryParamsWS>,
    Extension(state): Extension<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state, client, solution, environment))
}

pub async fn broadcast(
    state: Arc<AppState>,
    client: String,
    solution: String,
    environment: String,
) -> Result<(), SendError<String>> {
    let sender = {
        let mut map = state.channel_map.write().await;
        map.entry(format!("{environment}_{solution}_{client}").clone())
            .or_insert_with(|| channel(100).0)
            .clone()
    };
    sender.send("deployment ready".to_string())?;
    Ok(())
}

pub async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    client: String,
    solution: String,
    environment: String,
) {
    let (mut ws_tx, mut _ws_rx) = socket.split();

    let sender = {
        let mut map = state.channel_map.write().await;
        map.entry(format!("{environment}_{solution}_{client}").clone())
            .or_insert_with(|| channel(100).0)
            .clone()
    };

    // Spawn task to forward broadcast → WebSocket
    let mut rx = sender.subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                info!("something wrong");
                break;
            }
        }
    });
}
