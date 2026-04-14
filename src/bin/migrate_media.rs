use std::env;
use std::error::Error;

use tokio::fs;
use uuid::Uuid;

use slayFridayBot::adapter::postgres::PgStore;
use slayFridayBot::repo::media_storage::dto::MediaEntry as JsonMediaEntry;

const DEFAULT_MEDIA_JSON_PATH: &str = "sticker_storage.json";
const DEFAULT_MEDIA_DB_HOST: &str = "127.0.0.1";
const DEFAULT_MEDIA_DB_PORT: &str = "5500";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let json_path = env::var("MEDIA_JSON_PATH")
        .unwrap_or_else(|_| DEFAULT_MEDIA_JSON_PATH.to_string());
    let db_url = get_db_url()?;

    let raw = fs::read_to_string(&json_path).await?;
    let entries: Vec<JsonMediaEntry> = serde_json::from_str(&raw)?;

    let pg = PgStore::new(&db_url).await?;
    let mut tx = pg.pool.begin().await?;

    for entry in &entries {
        sqlx::query(
            r#"
            insert into media (id, name, file_id, media_type, added_by)
            values ($1, $2, $3, 'sticker'::media_type, null)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&entry.name)
        .bind(&entry.file_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    println!(
        "Migrated {} media entries from {}",
        entries.len(),
        json_path
    );

    Ok(())
}

fn get_db_url() -> Result<String, Box<dyn Error>> {
    if let Ok(db_url) = env::var("MEDIA_DB_URL") {
        return Ok(db_url);
    }

    let user = env::var("MEDIA_DB_USER")?;
    let password = env::var("MEDIA_DB_PASSWORD")?;
    let db_name = env::var("MEDIA_DB_NAME")?;
    let host =
        env::var("MEDIA_DB_HOST").unwrap_or_else(|_| DEFAULT_MEDIA_DB_HOST.to_string());
    let port =
        env::var("MEDIA_DB_PORT").unwrap_or_else(|_| DEFAULT_MEDIA_DB_PORT.to_string());

    Ok(format!(
        "postgres://{}:{}@{}:{}/{}?sslmode=disable",
        user, password, host, port, db_name
    ))
}
