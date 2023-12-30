use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn get_test_db_connection() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .min_connections(5)
        .connect("postgres://postgres:postgres@127.0.0.1:8500/postgres")
        .await
        .unwrap()
}
