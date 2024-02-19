use sqlx::{Row, FromRow, Error, MySql, query, query_as};
use sqlx::mysql::MySqlPool;
use anyhow::Result;

#[derive(Clone, Debug, FromRow)]
pub struct RecordsData {
    pub id: i32,
    pub records_html_id: i32,
    pub email: String,
    pub phone: String,
    pub website: String,
    pub contact_us_link: Option<String>,
}

impl RecordsData {
    pub async fn create_record(pool: &MySqlPool, record: &RecordsData) -> Result<(), Error> {
        println!("Creating record: {:?}", record);
        query("INSERT INTO records_data (records_html_id, email, phone, website) VALUES (?, ?, ?, ?)")
            .bind(&record.records_html_id)
            .bind(&record.email)
            .bind(&record.phone)
            .bind(&record.website)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn record_exists(pool: &MySqlPool, records_html_id: i32) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM records_data WHERE records_html_id = ? )")
            .bind(records_html_id)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }

    pub async fn record_exists_by_website(pool: &MySqlPool, website: &str) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM records_data WHERE website = ? )")
            .bind(website)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }

    pub async fn record_exists_by_phone(pool: &MySqlPool, phone: &str) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM records_data WHERE phone = ? )")
            .bind(phone)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }

    pub async fn get_all_records(pool: &MySqlPool) -> Result<Vec<RecordsData>, Error> {
        let records_data: Vec<RecordsData> = query_as("SELECT * FROM records_data")
            .fetch_all(pool)
            .await?;

        Ok(records_data)
    }

    pub async fn get_all_records_with_no_contact_us_link(pool: &MySqlPool) -> Result<Vec<RecordsData>, Error> {
        let records_data: Vec<RecordsData> = query_as("SELECT * FROM records_data WHERE contact_us_link IS NULL")
            .fetch_all(pool)
            .await?;

        Ok(records_data)
    }

    pub async fn delete_record(pool: &MySqlPool, record: &RecordsData) -> Result<(), Error> {
        println!("Deleting record: {:?}", record);
        query("DELETE FROM records_data WHERE records_html_id = ? AND email = ? AND phone = ? AND website = ?")
            .bind(&record.records_html_id)
            .bind(&record.email)
            .bind(&record.phone)
            .bind(&record.website)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn set_contact_us_link(pool: &MySqlPool, record: &RecordsData) -> Result<(), Error> {
        println!("Setting contact us link: {:?}", record);
        query("UPDATE records_data SET contact_us_link = ? WHERE id = ?")
            .bind(&record.contact_us_link)
            .bind(&record.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_website(pool: &MySqlPool, record: &RecordsData) -> Result<(), Error> {
        println!("Updating website: {:?}", record);
        query("UPDATE records_data SET website = ? WHERE id = ?")
            .bind(&record.website)
            .bind(&record.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_email(pool: &MySqlPool, record: &RecordsData) -> Result<(), Error> {
        println!("Updating email: {:?}", record);
        query("UPDATE records_data SET email = ? WHERE id = ?")
            .bind(&record.email)
            .bind(&record.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_all_records_houzz(pool: &MySqlPool) -> Result<Vec<RecordsData>, Error> {
        let records_data: Vec<RecordsData> = query_as("SELECT records_data.email, records_data.id, records_data.website, records_data.contact_us_link, records_data.phone, records_data.records_html_id FROM records_data INNER JOIN records_html ON records_data.records_html_id = records_html.id INNER JOIN links_to_record_details ON records_html.link_to_record_details_id = links_to_record_details.id INNER JOIN pages_with_all_records ON pages_with_all_records.id = links_to_record_details.pages_with_all_records_id WHERE pages_with_all_records.district LIKE '%General Contractors in Ontario - Houzz%' AND records_data.website != ''")
            .fetch_all(pool)
            .await?;

        Ok(records_data)
    }

    pub async fn get_record_data_by_records_data_id(pool: &MySqlPool, records_data_id: i32) -> Result<RecordsData, Error> {
        let record: RecordsData = query_as("SELECT * FROM records_data WHERE id = ?")
            .bind(records_data_id)
            .fetch_one(pool)
            .await?;

        Ok(record)
    }
}