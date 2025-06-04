use sqlx::{Error, PgPool};

pub trait DataRepository<T> {
    fn new(pool: &PgPool) -> Self;
    async fn exists(&self, id: i64) -> Result<bool, Error>;
    async fn find_by_id(&self, id: i64) -> Result<T, Error>;
    async fn find_all(&self) -> Result<Vec<T>, Error>;
    async fn create(&self, client: &str, environment: &str, solution: &str) -> Result<T, Error>;
    async fn update(
        &self,
        id: i64,
        client: &str,
        environment: &str,
        solution: &str,
    ) -> Result<bool, Error>;

    async fn delete(&self, id: i64) -> Result<bool, Error>;
}
