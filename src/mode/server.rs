use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    Extension, Router,
    routing::{any, delete, get, post, put},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{
    net::TcpListener,
    sync::{RwLock, broadcast::Sender},
};

use crate::{
    server::{
        api::{
            compose_stack::{delete_stack, get_all_stacks, get_stack, post_stack, update_stack},
            container::{
                delete_container, get_all_containers, get_container, post_container,
                update_container,
            },
            deployments::{
                delete_deployment, get_all_deployments, get_deployment, post_deployment,
                update_deployment,
            },
            hikari::{get_hikari_by_metadata, get_hikari_by_name},
        },
        ws::websocket::websocket_handler,
    },
    utils::secrets::load_secrets,
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub channel_map: Arc<RwLock<HashMap<String, Sender<String>>>>,
}

pub async fn server_mode() {
    let secrets = load_secrets("server");
    let pool = PgPoolOptions::new()
        .test_before_acquire(true)
        .max_connections(50)
        .min_connections(20)
        .idle_timeout(Duration::from_secs(1800))
        .max_lifetime(Duration::from_secs(1800))
        .connect(
            format!(
                "postgres://{}:{}@{}:{}/{}",
                secrets[0], secrets[1], secrets[2], secrets[3], secrets[4]
            )
            .as_str(),
        )
        .await
        .unwrap();
    let shared_state = Arc::new(AppState {
        pool,
        channel_map: Arc::new(RwLock::new(HashMap::new())),
    });
    let app = Router::new()
        .route("/api/v1/deployments", get(get_all_deployments))
        .route("/api/v1/deployment", get(get_deployment))
        .route("/api/v1/deployment", post(post_deployment))
        .route("/api/v1/deployment", put(update_deployment))
        .route("/api/v1/deployment", delete(delete_deployment))
        .route("/api/v1/stacks", get(get_all_stacks))
        .route("/api/v1/stack", get(get_stack))
        .route("/api/v1/stack", post(post_stack))
        .route("/api/v1/stack", put(update_stack))
        .route("/api/v1/stack", delete(delete_stack))
        .route("/api/v1/containers", get(get_all_containers))
        .route("/api/v1/container", get(get_container))
        .route("/api/v1/container", post(post_container))
        .route("/api/v1/container", put(update_container))
        .route("/api/v1/container", delete(delete_container))
        .route("/api/v1/hikari/metadata", get(get_hikari_by_metadata))
        .route("/api/v1/hikari/name", get(get_hikari_by_name))
        .route("/ws", any(websocket_handler))
        .layer(Extension(shared_state));

    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
