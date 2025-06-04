use log::error;
use sqlx::{Error, PgPool, query_as};

use crate::server::{models::deploy_config::DeployConfigDTO, traits::model::DataRepository};

pub struct DeployConfigDAL {
    pub pool: PgPool,
}

impl DataRepository<DeployConfigDTO> for DeployConfigDAL {
    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn find_by_id(&self, id: i64) -> Result<DeployConfigDTO, Error> {
        let deployment: DeployConfigDTO = query_as!(
            DeployConfigDTO,
            r#"
            SELECT dc.id,
            dc.client,
            dc.environment,
            dc.solution,
            COALESCE(
                array_agg(cs.id) FILTER (WHERE cs.id IS NOT NULL),
                ARRAY[]::BIGINT[]
            ) AS stack_ids
            FROM deploy_config AS dc
            LEFT JOIN compose_stack AS cs
            ON cs.deployment_id = dc.id
            WHERE dc.id = $1
            GROUP BY dc.id, dc.client, dc.environment, dc.solution;
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            // Log the error server-side for debugging
            // :contentReference[oaicite:13]{index=13}
            error!("Database query failed: {err}");
            // Return a generic error message to the client with 500 status
            // :contentReference[oaicite:14]{index=14}
            err
        })?;
        Ok(deployment)
    }
}
