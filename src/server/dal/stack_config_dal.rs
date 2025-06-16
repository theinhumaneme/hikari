use log::error;
use sqlx::{Error, PgPool, query, query_as, query_scalar};

use crate::server::{
    models::{deploy_config::DeployConfigDTO, stack_config::StackConfigDTO},
    traits::model::DataRepository,
};

pub struct StackConfigDAL {
    pub pool: PgPool,
}
impl DataRepository<StackConfigDTO> for StackConfigDAL {
    type Payload = StackConfigDTO;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn exists(&self, id: i64) -> Result<bool, Error> {
        let exists = query_scalar!(
            "SELECT EXISTS(SELECT id FROM compose_stack WHERE id = $1)",
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(exists.unwrap_or(false))
    }

    async fn find_all(&self) -> Result<Vec<StackConfigDTO>, Error> {
        let compose_stacks: Vec<StackConfigDTO> = query_as!(
            StackConfigDTO,
            r#"
            SELECT cs.id,
            cs.deployment_id,
            cs.stack_name,
            cs.filename,
            cs.home_directory,
            COALESCE(
                array_agg(c.id) FILTER (WHERE c.id IS NOT NULL),
                ARRAY[]::BIGINT[]
            ) AS containers
            FROM compose_stack AS cs
            LEFT JOIN container AS c
            ON c.stack_id = cs.id
            GROUP BY cs.id, cs.deployment_id, cs.stack_name, cs.filename, cs.home_directory
            ORDER BY cs.id;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(compose_stacks)
    }

    async fn find_by_id(&self, id: i64) -> Result<StackConfigDTO, Error> {
        let compose_stack: StackConfigDTO = query_as!(
            StackConfigDTO,
            r#"
            SELECT cs.id,
            cs.deployment_id,
            cs.stack_name,
            cs.filename,
            cs.home_directory,
            COALESCE(
                array_agg(c.id) FILTER (WHERE c.id IS NOT NULL),
                ARRAY[]::BIGINT[]
            ) AS containers
            FROM compose_stack AS cs
            LEFT JOIN container AS c
            ON c.stack_id = cs.id
            WHERE cs.id = $1
            GROUP BY cs.id, cs.deployment_id, cs.stack_name, cs.filename, cs.home_directory;
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(compose_stack)
    }

    async fn get_deployment_metadata(&self, id: i64) -> Result<DeployConfigDTO, Error> {
        let deployment: DeployConfigDTO = query_as!(
            DeployConfigDTO,
            r#"
            SELECT
            dc.id,
            dc.name,
            dc.client,
            dc.environment,
            dc.solution,
            COALESCE(
                array_agg(cs.id) FILTER (WHERE cs.id IS NOT NULL),
                ARRAY[]::BIGINT[]
            ) AS stack_ids
            FROM compose_stack AS cs
            JOIN deploy_config AS dc
            ON cs.deployment_id = dc.id
            WHERE cs.id = $1
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

    async fn create(&self, object: StackConfigDTO) -> Result<StackConfigDTO, Error> {
        let row = query!(
            r#"
            INSERT INTO
            compose_stack(deployment_id, stack_name, filename, home_directory
            ) VALUES ($1, $2, $3, $4)
            RETURNING id, deployment_id, stack_name, filename, home_directory;
            "#,
            object.deployment_id,
            object.stack_name,
            object.filename,
            object.home_directory
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(StackConfigDTO {
            id: Some(row.id),
            deployment_id: row.deployment_id,
            stack_name: row.stack_name,
            filename: row.filename,
            home_directory: row.home_directory,
            containers: Some(Vec::<i64>::new()),
        })
    }

    async fn update(&self, object: StackConfigDTO) -> Result<bool, Error> {
        let row = query!(
            r#"UPDATE compose_stack
            SET deployment_id=$2,
            stack_name=$3,
            filename=$4,
            home_directory=$5
            WHERE id=$1;"#,
            object.id,
            object.deployment_id,
            object.stack_name,
            object.filename,
            object.home_directory
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(row.rows_affected() > 0)
    }

    async fn delete(&self, id: i64) -> Result<bool, Error> {
        let row = query!(r#"DELETE FROM compose_stack WHERE id=$1;"#, id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                error!("Database query failed: {err}");
                err
            })?;
        Ok(row.rows_affected() > 0)
    }
}
