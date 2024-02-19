use sqlx::{Row, FromRow, Error, MySql, query, query_as};
use sqlx::mysql::MySqlPool;
use anyhow::Result;

#[derive(Clone, Debug, FromRow)]
pub struct LinksToRecordDetails {
    pub id: i32,
    pub pages_with_all_records_id: i32,
    pub company: String,
    pub link: String,
    pub visited: i32
}

impl LinksToRecordDetails {
    pub async fn create_record(pool: &MySqlPool, link: &LinksToRecordDetails) -> Result<(), Error> {
        println!("Creating link: {:?}", link);
        query("INSERT INTO links_to_record_details (pages_with_all_records_id, company, link) VALUES (?, ?, ?)")
            .bind(&link.pages_with_all_records_id)
            .bind(&link.company)
            .bind(&link.link)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete_record(pool: &MySqlPool, link: &LinksToRecordDetails) -> Result<(), Error> {
        println!("Deleting link: {:?}", link);
        query("DELETE FROM links_to_record_details WHERE pages_with_all_records_id = ? AND company = ? AND link = ?")
            .bind(&link.pages_with_all_records_id)
            .bind(&link.company)
            .bind(&link.link)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn mark_record_as_visited(pool: &MySqlPool, link: &LinksToRecordDetails) -> Result<(), Error> {
        println!("Marking link as visited: {:?}", link);
        query("UPDATE links_to_record_details SET visited = 1 WHERE link = ?")
            .bind(&link.link)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_all_records(pool: &MySqlPool) -> Result<Vec<LinksToRecordDetails>, Error> {
        let links_to_record_details: Vec<LinksToRecordDetails> = query_as("SELECT * FROM links_to_record_details")
            .fetch_all(pool)
            .await?;

        Ok(links_to_record_details)
    }

    pub async fn get_all_records_houzz(pool: &MySqlPool) -> Result<Vec<LinksToRecordDetails>, Error> {
        //let query = "SELECT DISTINCT links_to_record_details.link, links_to_record_details.pages_with_all_records_id, links_to_record_details.company, links_to_record_details.visited, links_to_record_details.id FROM links_to_record_details INNER JOIN pages_with_all_records ON pages_with_all_records.id = links_to_record_details.pages_with_all_records_id WHERE district LIKE '%Home Builders in Ontario - Houzz%'";
        //let query2 = "SELECT DISTINCT links_to_record_details.link, links_to_record_details.company FROM links_to_record_details INNER JOIN pages_with_all_records ON links_to_record_details.pages_with_all_records_id = pages_with_all_records.id WHERE district LIKE '%Home Builders in Ontario - Houzz%'";
        
        let links_to_record_details: Vec<LinksToRecordDetails> = query_as("SELECT DISTINCT links_to_record_details.link, links_to_record_details.pages_with_all_records_id, links_to_record_details.company, links_to_record_details.visited, links_to_record_details.id FROM links_to_record_details INNER JOIN pages_with_all_records ON pages_with_all_records.id = links_to_record_details.pages_with_all_records_id WHERE district LIKE '%Home Builders in Ontario - Houzz%'")
            .fetch_all(pool)
            .await?;

        Ok(links_to_record_details)
    }

    pub async fn record_exists(pool: &MySqlPool, links_to_record_details_data: &LinksToRecordDetails) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM links_to_record_details WHERE pages_with_all_records_id = ? AND company = ? AND link = ? )")
            .bind(&links_to_record_details_data.pages_with_all_records_id)
            .bind(&links_to_record_details_data.company)
            .bind(&links_to_record_details_data.link)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }

    pub async fn get_all_unvisited_records(pool: &MySqlPool) -> Result<Vec<LinksToRecordDetails>, Error> {
        let links_to_record_details: Vec<LinksToRecordDetails> = query_as("SELECT * FROM links_to_record_details WHERE visited = 0")
            .fetch_all(pool)
            .await?;

        Ok(links_to_record_details)
    }
    
}