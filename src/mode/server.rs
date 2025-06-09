use std::{sync::Arc, time::Duration};

use axum::{
    Extension, Router,
    routing::{delete, get, post, put},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

use crate::{
    server::api::{
        compose_stack::{delete_stack, get_all_stacks, get_stack, post_stack, update_stack},
        container::{
            delete_container, get_all_containers, get_container, post_container, update_container,
        },
        deployments::{
            delete_deployment, get_all_deployments, get_deployment, post_deployment,
            update_deployment,
        },
    },
    utils::secrets::load_secrets,
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
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
    let shared_state = Arc::new(AppState { pool });
    let app = Router::new()
        .route("/deployments", get(get_all_deployments))
        .route("/deployment", get(get_deployment))
        .route("/deployment", post(post_deployment))
        .route("/deployment", put(update_deployment))
        .route("/deployment", delete(delete_deployment))
        .route("/stacks", get(get_all_stacks))
        .route("/stack", get(get_stack))
        .route("/stack", post(post_stack))
        .route("/stack", put(update_stack))
        .route("/stack", delete(delete_stack))
        .route("/containers", get(get_all_containers))
        .route("/container", get(get_container))
        .route("/container", post(post_container))
        .route("/container", put(update_container))
        .route("/container", delete(delete_container))
        .layer(Extension(shared_state));

    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
