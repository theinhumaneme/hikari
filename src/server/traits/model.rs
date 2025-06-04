use sqlx::{Error, PgPool};

pub trait DataRepository<T> {
    fn new(pool: &PgPool) -> Self;
    async fn find_by_id(&self, id: i64) -> Result<T, Error>;
}
