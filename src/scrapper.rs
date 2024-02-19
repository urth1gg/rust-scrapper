use fantoccini::{Client, Locator};
use anyhow::Result;


pub struct Scrapper<'a> {
    pub client: &'a Client,
}

impl<'a> Scrapper<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self {
            client,
        }
    }

    pub async fn close(&self) -> Result<()> {
        self.client.clone().close().await?;
        Ok(())
    }

    pub async fn get_body(&self, url: &str) -> Result<String> {
        self.client.goto(url).await?;
        let body = self.client.find(Locator::Css("body")).await?.html(false).await?;
        
        if body != "" {
            println!("Got body");
        } else {
            println!("No body");
        }
        
        Ok(body)
    }

    pub async fn get_element_html(&self, url: &str, selector: &str) -> Result<String> {
        self.client.goto(url).await?;
        let element = self.client.find(Locator::Css(selector)).await?.html(false).await?;
        
        if element != "" {
            println!("Got element");
        } else {
            println!("No element");
        }
        
        Ok(element)
    }

}   