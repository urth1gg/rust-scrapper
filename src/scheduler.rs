use std::any::Any;

use fantoccini::{Client, Locator, ClientBuilder};
use anyhow::Error;
use serde_json::json;

#[derive(Clone)]
pub struct Scheduler{
    pub clients: Vec<Client>,
    pub used: Vec<bool>
}

impl Scheduler {
    pub fn new(clients: Vec<Client>) -> Self {
        let mut used = Vec::new();
        for _ in 0..clients.len() {
            used.push(false);
        }

        Self {
            clients: clients,
            used: used
        }
    }

    pub async fn get_client(&mut self) -> Result<&Client, Error> {
        for i in 0..self.clients.len() {
            if !self.used[i] {
                self.used[i] = true;
                println!("Cleints used: {:?}", self.used);
                return Ok(&self.clients[i]);
            }
        }

        Err(anyhow::anyhow!("No available clients"))
    }

    pub async fn release_client(&mut self, client: &Client) -> Result<(), Error> {
        let mut index = None;

        let client_session_id = client.session_id().await?.unwrap();

        for i in 0..self.clients.len() {
            let current_client_session_id = self.clients[i].session_id().await?.unwrap();

            if current_client_session_id == client_session_id {
                index = Some(i);
                break;
            }
        }
        
        match index {
            Some(index) => {
                self.used[index] = false;
                //client.clone().close().await?;
                Ok(())
            },
            None => Err(anyhow::anyhow!("Client not found"))
        }
    }

    pub async fn replace_client(&mut self, client: &Client, headless: bool) -> Result<(), Error> {
        let ports = [4444, 4445, 4446, 4447, 4448, 4449, 4450, 4451, 4452, 4453, 4454, 4455];

        let mut index = None;

        let client_session_id = client.session_id().await?.unwrap();

        for i in 0..self.clients.len() {
            let current_client_session_id = self.clients[i].session_id().await?.unwrap();

            if current_client_session_id == client_session_id {
                index = Some(i);
                break;
            }
        }
        
        let mut caps = serde_json::map::Map::new();

        let opts = match headless {
            true => json!({
                "args": ["--no-sandbox", "--headless", "--disable-gpu", "--disable-dev-shm-usage"]
            }),
            false => json!({
                "args": ["--no-sandbox", "--disable-gpu", "--disable-dev-shm-usage", "--display=192.168.1.2:0"],
            }),
        };
    
        caps.insert("goog:chromeOptions".to_string(), opts);

        let new_client = ClientBuilder::rustls()
            .capabilities(caps.clone())
            .connect(&format!("http://localhost:{}", ports[index.unwrap()]))
            .await?;

        let timeouts = fantoccini::wd::TimeoutConfiguration::new(
            Some(std::time::Duration::from_secs(30)),
            Some(std::time::Duration::from_secs(30)),
            Some(std::time::Duration::from_secs(30)),
        );

        new_client.set_window_rect(0, 0, 1920, 1080).await?;
        new_client.update_timeouts(timeouts.clone()).await?;
        new_client.persist().await?;

        let _ = &client.clone().close().await.unwrap();


        match index {
            Some(index) => {
                self.clients[index] = new_client;
                self.used[index] = false;
                Ok(())
            },
            None => Err(anyhow::anyhow!("Client not found"))
        }

        
    }

    pub async fn generate_new_client(&mut self, headless: bool, index: Option<usize>) -> Result<Client, Error> {
        let ports = [4444, 4445, 4446, 4447, 4448, 4449, 4450, 4451, 4452, 4453, 4454, 4455];

        let mut caps = serde_json::map::Map::new();

        let opts = match headless {
            true => json!({
                "args": ["--no-sandbox", "--headless", "--disable-gpu", "--disable-dev-shm-usage"]
            }),
            false => json!({
                "args": ["--no-sandbox", "--disable-gpu", "--disable-dev-shm-usage", "--display=192.168.1.2:0"],
            }),
        };
    
        caps.insert("goog:chromeOptions".to_string(), opts);

        let new_client = ClientBuilder::rustls()
            .capabilities(caps.clone())
            .connect(&format!("http://localhost:{}", ports[index.unwrap()]))
            .await?;

        let timeouts = fantoccini::wd::TimeoutConfiguration::new(
            Some(std::time::Duration::from_secs(33)),
            Some(std::time::Duration::from_secs(33)),
            Some(std::time::Duration::from_secs(33)),
        );

        new_client.set_window_rect(0, 0, 1920, 1080).await?;
        new_client.update_timeouts(timeouts.clone()).await?;
        new_client.persist().await?;

        Ok(new_client)
    }

}