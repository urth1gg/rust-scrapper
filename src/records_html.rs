use sqlx::{Row, FromRow, Error, MySql, query, query_as};
use sqlx::mysql::MySqlPool;
use anyhow::Result;
use super::links_to_record_details::LinksToRecordDetails;

#[derive(Clone, Debug, FromRow)]
pub struct RecordsHtml {
    pub id: i32,
    pub link_to_record_details_id: i32,
    pub html: String,
    pub processed: i32,
}

impl RecordsHtml {
    pub async fn create_record(pool: &MySqlPool, record: &RecordsHtml) -> Result<(), Error> {
        println!("Creating record");
        query("INSERT INTO records_html (link_to_record_details_id, html) VALUES (?, ?)")
            .bind(&record.link_to_record_details_id)
            .bind(&record.html)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_all_records(pool: &MySqlPool) -> Result<Vec<RecordsHtml>, Error> {
        let records_html: Vec<RecordsHtml> = query_as("SELECT * FROM records_html")
            .fetch_all(pool)
            .await?;

        Ok(records_html)
    }

    pub async fn get_all_unprocessed_records(pool: &MySqlPool) -> Result<Vec<RecordsHtml>, Error> {
        let records_html: Vec<RecordsHtml> = query_as("SELECT * FROM records_html WHERE processed = 0")
            .fetch_all(pool)
            .await?;

        Ok(records_html)
    }

    pub async fn delete_record(pool: &MySqlPool, record: &RecordsHtml) -> Result<(), Error> {
        println!("Deleting record: {:?}", record);
        query("DELETE FROM records_html WHERE link_to_record_details_id = ? AND html = ?")
            .bind(&record.link_to_record_details_id)
            .bind(&record.html)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn record_exists(pool: &MySqlPool, links_to_record_details_id: i32) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM records_html WHERE link_to_record_details_id = ? )")
            .bind(&links_to_record_details_id)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }
}