use log::error;
use sqlx::{Error, PgPool, query, query_as, query_scalar};

use crate::server::{models::deploy_config::DeployConfigDTO, traits::model::DataRepository};

pub struct DeployConfigDAL {
    pub pool: PgPool,
}
impl DataRepository<DeployConfigDTO> for DeployConfigDAL {
    type Payload = DeployConfigDTO;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn exists(&self, id: i64) -> Result<bool, Error> {
        let exists = query_scalar!(
            "SELECT EXISTS(SELECT id FROM deploy_config WHERE id = $1)",
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(exists.unwrap_or(false))
    }

    async fn find_all(&self) -> Result<Vec<DeployConfigDTO>, Error> {
        let deployments: Vec<DeployConfigDTO> = query_as!(
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
            GROUP BY dc.id, dc.client, dc.environment, dc.solution;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(deployments)
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
            error!("Database query failed: {err}");
            err
        })?;
        Ok(deployment)
    }

    async fn create(&self, object: DeployConfigDTO) -> Result<DeployConfigDTO, Error> {
        let row = query!(
            "INSERT INTO deploy_config(client, environment, solution
            ) VALUES ($1, $2, $3) RETURNING id, client, environment, solution;",
            object.client,
            object.environment,
            object.solution
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(DeployConfigDTO {
            id: Some(row.id),
            client: row.client,
            environment: row.environment,
            solution: row.solution,
            stack_ids: Some(Vec::<i64>::new()),
        })
    }

    async fn update(&self, object: DeployConfigDTO) -> Result<bool, Error> {
        let row= query(
            r#"UPDATE deploy_config SET client=$2, environment=$3, solution=$4 WHERE id=$1 RETURNING id, client, environment, solution;"#).bind(
            object.id).
            bind(object.client).
        bind(object.environment).
            bind(object.solution).execute(&self.pool)
        .await.map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(row.rows_affected() > 0)
    }

    async fn delete(&self, id: i64) -> Result<bool, Error> {
        let row = query(
            r#"DELETE FROM deploy_config WHERE id=$1 RETURNING id, client, environment, solution;"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(row.rows_affected() > 0)
    }
}
