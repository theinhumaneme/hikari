use log::error;
use sqlx::{Error, PgPool, query, query_as, query_scalar};

use crate::server::{models::container::ContainerDTO, traits::model::DataRepository};

pub struct ContainerDAL {
    pub pool: PgPool,
}
impl DataRepository<ContainerDTO> for ContainerDAL {
    type Payload = ContainerDTO;

    fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn exists(&self, id: i64) -> Result<bool, Error> {
        let exists = query_scalar!("SELECT EXISTS(SELECT id FROM container WHERE id = $1)", id)
            .fetch_one(&self.pool)
            .await?;
        Ok(exists.unwrap_or(false))
    }

    async fn find_all(&self) -> Result<Vec<ContainerDTO>, Error> {
        let compose_stacks: Vec<ContainerDTO> = query_as!(
            ContainerDTO,
            r#"
            SELECT *
            FROM container AS c
            ORDER BY c.id;
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

    async fn find_by_id(&self, id: i64) -> Result<ContainerDTO, Error> {
        let compose_stack: ContainerDTO = query_as!(
            ContainerDTO,
            r#"
            SELECT *
            FROM container AS c
            WHERE c.id = $1
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

    async fn create(&self, object: ContainerDTO) -> Result<ContainerDTO, Error> {
        let row = query!(
            r#"
            INSERT INTO
            container(
            stack_id,
            service_name,
            container_name,
            image,
            restart,
            "user",
            stdin_open,
            tty,
            command,
            pull_policy,
            ports,
            volumes,
            environment,
            mem_reservation,
            mem_limit,
            oom_kill_disable,
            privileged
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING
            id,
            stack_id,
            service_name,
            container_name,
            image,
            restart,
            "user",
            stdin_open,
            tty,
            command,
            pull_policy,
            ports,
            volumes,
            environment,
            mem_reservation,
            mem_limit,
            oom_kill_disable,
            privileged
            "#,
            object.stack_id,
            object.service_name,
            object.container_name,
            object.image,
            object.restart,
            object.user,
            object.stdin_open,
            object.tty,
            object.command,
            object.pull_policy,
            object.ports.as_deref(),
            object.volumes.as_deref(),
            object.environment.as_deref(),
            object.mem_reservation,
            object.mem_limit,
            object.oom_kill_disable,
            object.privileged,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!("Database query failed: {err}");
            err
        })?;
        Ok(ContainerDTO {
            id: Some(row.id),
            stack_id: row.stack_id,
            service_name: row.service_name,
            container_name: row.container_name,
            image: row.image,
            restart: row.restart,
            user: row.user,
            stdin_open: row.stdin_open,
            tty: row.tty,
            command: row.command,
            pull_policy: row.pull_policy,
            ports: row.ports,
            volumes: row.volumes,
            environment: row.environment,
            mem_reservation: row.mem_reservation,
            mem_limit: row.mem_limit,
            oom_kill_disable: row.oom_kill_disable,
            privileged: row.privileged,
        })
    }

    async fn update(&self, object: ContainerDTO) -> Result<bool, Error> {
        let row = query!(
            r#"
        UPDATE container SET
        stack_id = $2,
        service_name = $3,
        container_name = $4,
        image = $5,
        restart = $6,
        "user" = $7,
        stdin_open = $8,
        tty = $9,
        command = $10,
        pull_policy = $11,
        ports = $12,
        volumes = $13,
        environment = $14,
        mem_reservation = $15,
        mem_limit = $16,
        oom_kill_disable = $17,
        privileged = $18
        WHERE id = $1;
        "#,
            object.id,
            object.stack_id,
            object.service_name,
            object.container_name,
            object.image,
            object.restart,
            object.user,
            object.stdin_open,
            object.tty,
            object.command,
            object.pull_policy,
            object.ports.as_deref(),
            object.volumes.as_deref(),
            object.environment.as_deref(),
            object.mem_reservation,
            object.mem_limit,
            object.oom_kill_disable,
            object.privileged
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
        let row = query!(r#"DELETE FROM container WHERE id=$1;"#, id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                error!("Database query failed: {err}");
                err
            })?;
        Ok(row.rows_affected() > 0)
    }
}
