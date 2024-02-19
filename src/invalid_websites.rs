use sqlx::{Row, FromRow, Error, MySql, query, query_as};

use sqlx::mysql::MySqlPool;

#[derive(Clone, Debug, FromRow)]
pub struct InvalidWebsites {
    pub website: String,
}

impl InvalidWebsites {
    pub async fn create_record(pool: &MySqlPool, record: &InvalidWebsites) -> Result<(), Error> {
        println!("Creating invalid website: {:?}", record);
        query("INSERT INTO invalid_websites (website) VALUES (?)")
            .bind(&record.website)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn record_exists(pool: &MySqlPool, website: &str) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM invalid_websites WHERE website = ? )")
            .bind(website)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }

    pub async fn get_all_records(pool: &MySqlPool) -> Result<Vec<InvalidWebsites>, Error> {
        let invalid_websites: Vec<InvalidWebsites> = query_as("SELECT * FROM invalid_websites")
            .fetch_all(pool)
            .await?;

        Ok(invalid_websites)
    }

    pub async fn delete_record(pool: &MySqlPool, record: &InvalidWebsites) -> Result<(), Error> {
        println!("Deleting invalid website: {:?}", record);
        query("DELETE FROM invalid_websites WHERE website = ?")
            .bind(&record.website)
            .execute(pool)
            .await?;

        Ok(())
    }
}