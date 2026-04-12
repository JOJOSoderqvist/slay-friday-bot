use crate::errors::InfraError::{self, PGConnectError};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

const PG_POOL_MAX_CONNECTIONS: u32 = 5;

pub struct PgStore {
    pool: Pool<Postgres>,
}

impl PgStore {
    pub async fn new(conn_str: &str) -> Result<Self, InfraError> {
        let pool = PgPoolOptions::new()
            .max_connections(PG_POOL_MAX_CONNECTIONS)
            .connect(conn_str)
            .await
            .map_err(PGConnectError)?;

        Ok(PgStore { pool })
    }
}
