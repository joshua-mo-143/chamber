

use sqlx::{postgres::PgPoolOptions, PgPool};
use sqlx::Executor;
use boulder_db::postgres::MIGRATIONS;

pub async fn get_test_db_connection() -> PgPool {
        let pool = PgPoolOptions::new()
                        .max_connections(50)
                        .min_connections(50)
                        .connect("postgres://postgres:postgres@localhost:8500/postgres")
                        .await
                        .unwrap();

        pool.execute(MIGRATIONS).await.unwrap();

        pool
}
