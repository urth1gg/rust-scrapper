use sqlx::{MySqlPool, Row};
use anyhow::Result;
use sqlx::query_as;
use std::env;
#[derive(Clone, Debug)]
pub struct PagesWithAllRecords {
    pub id: i32,
    pub page: Option<String>,
    pub district: Option<String>,
    pub query: Option<String>,
    pub html: Option<String>,
    pub processed: Option<i32>,
}

impl PagesWithAllRecords {
    pub async fn create_record(page: &PagesWithAllRecords, pool: &MySqlPool) -> Result<()> {
        println!("Creating page: {:?}", page);
        sqlx::query("INSERT INTO pages_with_all_records (page, district, query, html) VALUES (?, ?, ?, ?)")
            .bind(&page.page)
            .bind(&page.district)
            .bind(&page.query)
            .bind(&page.html)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn mark_record_as_processed(page: &PagesWithAllRecords, pool: &MySqlPool) -> Result<()> {
        println!("Marking page as processed: {:?}", page);
        sqlx::query("UPDATE pages_with_all_records SET processed = 1 WHERE id = ?")
            .bind(&page.id)
            .execute(pool)
            .await?;
    
        Ok(())
    }

    pub async fn get_all_records(pool: &MySqlPool) -> Result<Vec<PagesWithAllRecords>> {
        let mut pages_with_all_records = Vec::new();

            
        let pages: Vec<PagesWithAllRecords> = query_as!(
            PagesWithAllRecords,
            "SELECT * FROM pages_with_all_records"
        )
        .fetch_all(pool)
        .await?;


        pages_with_all_records.extend(pages);

        Ok(pages_with_all_records)
    }

    pub async fn get_all_unprocessed_records(pool: &MySqlPool) -> Result<Vec<PagesWithAllRecords>> {
        let records: Vec<PagesWithAllRecords> = query_as!(
            PagesWithAllRecords,
            "SELECT * FROM pages_with_all_records WHERE processed = 0"
        )
        .fetch_all(pool)
        .await?;
    
        Ok(records)
    }

    // Similar changes can be made to other methods...
}