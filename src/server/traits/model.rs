use sqlx::PgPool;

use crate::{server::models::deploy_config::DeployConfigDTO, utils::error::RepoError};

pub trait DataRepository<T> {
    type Payload;

    fn new(pool: &PgPool) -> Self;
    async fn exists(&self, id: i64) -> Result<bool, RepoError>;
    async fn find_by_id(&self, id: i64) -> Result<T, RepoError>;
    async fn find_all(&self) -> Result<Vec<T>, RepoError>;
    async fn create(&self, payload: Self::Payload) -> Result<T, RepoError>;
    async fn update(&self, payload: Self::Payload) -> Result<bool, RepoError>;
    async fn delete(&self, id: i64) -> Result<bool, RepoError>;
    async fn get_deployment_metadata(&self, id: i64) -> Result<DeployConfigDTO, RepoError>;
}
