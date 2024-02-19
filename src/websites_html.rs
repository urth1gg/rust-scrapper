use sqlx::{Row, FromRow, Error, MySql, query, query_as};
use sqlx::mysql::MySqlPool;
use anyhow::Result;

#[derive(Clone, Debug, FromRow)]
pub struct WebsitesHtml {
    pub id: i32,
    pub records_data_id: i32,
    pub website: String,
    pub main_page_html: String,
    pub contact_page_html: String,
}
#[derive(Clone, Debug, FromRow)]
pub struct PartialWebsitesHtml {
    pub id: i32,
    pub records_data_id: i32,
    pub website: String,
    pub contact_page_html: Option<String>,
}

impl WebsitesHtml {
    pub async fn create_record(pool: &MySqlPool, website: &WebsitesHtml) -> Result<(), Error> {
        println!("Creating website: {:?}", website);
        query("INSERT INTO websites_html (records_data_id, website, main_page_html, contact_page_html) VALUES (?, ?, ?, ?)")
            .bind(&website.records_data_id)
            .bind(&website.website)
            .bind(&website.main_page_html)
            .bind(&website.contact_page_html)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn record_exists(pool: &MySqlPool, records_data_id: i32) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM websites_html WHERE records_data_id = ? )")
            .bind(records_data_id)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }

    pub async fn website_exists(pool: &MySqlPool, website: &str) -> Result<bool, Error> {
        let exists: (i32,) = query_as("SELECT EXISTS( SELECT 1 FROM websites_html WHERE website = ? )")
            .bind(website)
            .fetch_one(pool)
            .await?;

        Ok(exists.0 == 1)
    }

    pub async fn get_all_websites(pool: &MySqlPool) -> Result<Vec<WebsitesHtml>, Error> {
        let websites_html: Vec<WebsitesHtml> = query_as("SELECT * FROM websites_html")
            .fetch_all(pool)
            .await?;

        Ok(websites_html)
    }

    pub async fn get_all_websites_with_no_contact_page_html(pool: &MySqlPool) -> Result<Vec<PartialWebsitesHtml>, Error> {
        let websites_html: Vec<PartialWebsitesHtml> = query_as("SELECT id, records_data_id, website, contact_page_html FROM websites_html WHERE contact_page_html = '' OR contact_page_html IS NULL")
            .fetch_all(pool)
            .await?;

        Ok(websites_html)
    }

    pub async fn delete_website(pool: &MySqlPool, website: &WebsitesHtml) -> Result<(), Error> {
        println!("Deleting website: {:?}", website);
        query("DELETE FROM websites_html WHERE id = ?")
            .bind(&website.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_main_page_html(pool: &MySqlPool, website: &WebsitesHtml) -> Result<(), Error> {
        println!("Updating main page html: {:?}", website);
        query("UPDATE websites_html SET main_page_html = ? WHERE id = ?")
            .bind(&website.main_page_html)
            .bind(&website.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_contact_page_html(pool: &MySqlPool, website: &WebsitesHtml) -> Result<(), Error> {
        query("UPDATE websites_html SET contact_page_html = ? WHERE website = ?")
            .bind(&website.contact_page_html)
            .bind(&website.website)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_website_by_records_data_id(pool: &MySqlPool, records_data_id: i32) -> Result<WebsitesHtml, Error> {
        let website: WebsitesHtml = query_as("SELECT * FROM websites_html WHERE records_data_id = ?")
            .bind(records_data_id)
            .fetch_one(pool)
            .await?;

        Ok(website)
    }
}