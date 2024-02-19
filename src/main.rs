mod data;
mod scheduler;
mod scrapper;
mod tasks;
mod pages_with_all_records;
mod records_html;
mod links_to_record_details;
mod extractor;
mod records_data;
mod websites_html;
mod invalid_websites;

use anyhow::Error;
use fantoccini::{Client, ClientBuilder};
use serde_json::json;
use sqlx::MySql;
use sqlx::Pool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use pages_with_all_records::PagesWithAllRecords;
use std::env;
use sqlx::MySqlPool;
use data::HouzzEntry;
use rand::Rng;
use tokio::sync::broadcast;
use records_html::RecordsHtml;
use records_data::RecordsData;
use links_to_record_details::LinksToRecordDetails;
use websites_html::WebsitesHtml;
use extractor::Extractor;
use invalid_websites::InvalidWebsites;
use std::convert::TryInto;
use std::collections::HashSet;


pub struct UrlData {
    url: String,
    page: i32,
}

#[derive(Clone, Debug)]
pub struct UrlDataLinks {
    url: String,
    link_to_record_details_id: i32,
}

pub struct UrlDataRecord {
    url: String,
    record_id: i32,

}

pub struct WebsitesHtmlData{
    website: String,
    website_html_id: i32,
    record_id: i32,
}

unsafe impl Send for UrlData {}

async fn generate_clients(headless: bool, number_of_clients: i32) -> Result<Vec<Client>, Error> {
    let mut clients = Vec::new();
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

    for number in 0..number_of_clients {
        let port = 4444 + number;
        let client = ClientBuilder::rustls()
            .capabilities(caps.clone())
            .connect(&format!("http://localhost:{}", port.to_string()))
            .await?;

        let timeouts = fantoccini::wd::TimeoutConfiguration::new(
            Some(std::time::Duration::from_secs(30)),
            Some(std::time::Duration::from_secs(30)),
            Some(std::time::Duration::from_secs(30)),
        );

        client.set_window_rect(0, 0, 1920, 1080).await?;
        client.update_timeouts(timeouts.clone()).await?;
        client.persist().await?;

        println!("Client created {:?}", client.session_id().await);
        clients.push(client);
    }

    Ok(clients)
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let data = data::generate_houzz_input_records();
    let pool = MySqlPool::connect(&database_url).await?;

    println!("Data: {:?}", data);

    let houzz_data: [HouzzEntry; 8] = data::generate_houzz_input_records();
    let houzz_data = houzz_data.into_iter().filter(|record| record.category == "General Contractors in Ontario - Houzz").collect::<Vec<_>>();
    let houzz_data_record = houzz_data[0].clone();

    println!("Houzz data: {:?}", houzz_data_record);

    //run_get_all_pages_houzz(pool, houzz_data_record).await?;
    //get_link_details_from_pages(&pool).await?;
    //run_get_all_records_html_from_links(pool).await?;
    //populate_records_data_from_records_html(&pool).await?;
    //fix_records_websites(&pool).await?;
    // Need to change get records houzz function to run the function below
    //run_insert_website_html_from_records_data_websites(&pool).await?;
    //update_contact_us_link_from_website_html(&pool).await?;
    //run_update_contact_page_html_from_websites_html(pool).await?;
    //update_record_data_email(&pool).await?;
    Ok(())
}

pub async fn get_all_pages_houzz(semaphore: Arc<Semaphore>, scheduler_clone: Arc<Mutex<scheduler::Scheduler>>, houzz_data_record: HouzzEntry, pool: MySqlPool, urls: Vec<UrlData>){
    let (cancel_tx, _) = broadcast::channel::<(usize, ())>(1);
    let cancel_tx = Arc::new(cancel_tx);

    let tasks: Vec<_> = urls
    .into_iter()
    .map(|url_data| {
        let semaphore = Arc::clone(&semaphore);
        let scheduler_clone = Arc::clone(&scheduler_clone);
        let houzz_data_record_clone = houzz_data_record.clone();
        let pool = pool.clone();
        let mut cancel_rx = cancel_tx.subscribe(); // Create a new receiver
        let cancel_tx_clone = Arc::clone(&cancel_tx); // Clone the Arc<Sender>
        tokio::spawn(async move {

            // Acquire a permit from the semaphore.
            let _permit = semaphore.acquire().await;

            // Check if cancellation has been requested
            if let Ok((cancel_page, _)) = cancel_rx.try_recv() {
                if url_data.page >= cancel_page.try_into().unwrap() {
                    println!("Cancellation requested for pages >= {}, stopping task", cancel_page);
                    return;
                }
            }

            // Try to get a client.
            let client = {
                let mut locked_scheduler = scheduler_clone.lock().await;
                match locked_scheduler.get_client().await {
                    Ok(client) => client.clone(),
                    Err(_) => {
                        println!("No available clients");
                        return;
                    }
                }
            };

            let scrapper = scrapper::Scrapper::new(&client);

            // Try to get the body.
            let body = match scrapper.get_element_html(&url_data.url, ".pro-results").await {
                Ok(body) => body,
                Err(e) => {
                    eprintln!("Error getting body: {:?}", e);
                    let mut locked_scheduler = scheduler_clone.lock().await;
                    if let Err(e) = locked_scheduler.release_client(&client).await {
                        println!("Failed to release client: {}", e);
                    }else{
                        println!("Released client");
                    }

                    drop(_permit);
                    return;
                }
            };

            let page_with_all_records = PagesWithAllRecords {
                id: 0,
                page: Some(url_data.page.to_string()),
                district: Some(houzz_data_record_clone.category.clone().to_string()),
                query: Some(url_data.url.to_string()),
                html: Some(body.clone().to_string()),
                processed: Some(0),
            };

            if body.contains("hz-browse-suggestions__tip") {
                println!("No more records found for this query");
                let _ = cancel_tx_clone.send((url_data.page.try_into().unwrap(), ()));
                return;
            }
    
            match PagesWithAllRecords::create_record(&page_with_all_records, &pool).await {
                Ok(_) => {
                    println!("Inserted and sleeping for");
                },
                Err(e) => {
                    // Log the error and continue with the next iteration
                    eprintln!("Error inserting record: {:?}", e);
                }
            }
    
            let sleep_time = rand::thread_rng().gen_range(1..3);
            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;


            {
                let mut locked_scheduler = scheduler_clone.lock().await;
                if let Err(e) = locked_scheduler.release_client(&client).await {
                    println!("Failed to release client: {}", e);
                }
            }


        })
    })
    .collect();

    // Wait for all tasks to complete.
    for task in tasks {
        task.await.unwrap();
    }


}

pub async fn get_all_records_html_from_links(semaphore: Arc<Semaphore>, scheduler_clone: Arc<Mutex<scheduler::Scheduler>>, pool: MySqlPool, urls: Vec<UrlDataLinks>) -> Result<(), Error>{

    let tasks: Vec<_> = urls
    .into_iter()
    .map(|url_data: UrlDataLinks| {
        let semaphore = Arc::clone(&semaphore);
        let scheduler_clone = Arc::clone(&scheduler_clone);
        let pool = pool.clone();
        tokio::spawn(async move {

            // Acquire a permit from the semaphore.
            let _permit = semaphore.acquire().await;


            let record_exists = match RecordsHtml::record_exists(&pool, url_data.link_to_record_details_id).await{
                Ok(exists) => exists,
                Err(e) => {
                    eprintln!("Error checking if record exists: {:?}", e);
                    return;
                }
            };

            if record_exists {
                println!("Record already exists, skipping");
                return;
            }

            let client;
            loop {
                let mut locked_scheduler = scheduler_clone.lock().await;
                match locked_scheduler.get_client().await {
                    Ok(available_client) => {
                        client = available_client.clone();
                        break;
                    },
                    Err(_) => {
                        println!("No available clients, retrying in 5 seconds...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }

            println!("Client id: {:?}", client.session_id().await);

            let scrapper = scrapper::Scrapper::new(&client);

            // Try to get the body.
            let body = match scrapper.get_element_html(&url_data.url, "#business").await {
                Ok(body) => body,
                Err(e) => {
                    eprintln!("Error getting body: {:?}", e);
                    let mut locked_scheduler = scheduler_clone.lock().await;

                    match locked_scheduler.replace_client(&client, false).await{
                        Ok(_) => {
                            println!("Replaced client");
                        },
                        Err(e) => {
                            println!("Failed to replace client: {}", e);
                        }
                    }

                    drop(_permit);
                    return;
                }
            };

            let record_html = RecordsHtml {
                id: 0,
                link_to_record_details_id: url_data.link_to_record_details_id,
                html: body.clone().to_string(),
                processed: 0,
            };
    
            match RecordsHtml::create_record(&pool, &record_html).await {
                Ok(_) => {
                    println!("Inserted and sleeping for");
                },
                Err(e) => {
                    // Log the error and continue with the next iteration
                    eprintln!("Error inserting record: {:?}", e);
                }
            }
    
            let sleep_time = rand::thread_rng().gen_range(1..3);
            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;


            let link_to_record_details = LinksToRecordDetails {
                id: url_data.link_to_record_details_id,
                pages_with_all_records_id: 0,
                company: "".to_string(),
                link: url_data.url.clone(),
                visited: 1,
            };

            LinksToRecordDetails::mark_record_as_visited(&pool, &link_to_record_details).await;

            {
                let mut locked_scheduler = scheduler_clone.lock().await;
                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                    println!("Failed to release client: {}", e);
                }
            }

        })
    })
    .collect();


    for task in tasks {
        task.await.unwrap();
    }

    Ok(())
}

pub async fn get_link_details_from_pages(pool: &Pool<MySql> ) -> Result<(), Error> {
    let pages_with_all_records = PagesWithAllRecords::get_all_unprocessed_records(&pool).await?;

    println!("Pages with all records: {:?}", pages_with_all_records.len());
    for page_with_all_records in pages_with_all_records {
        let extractor = Extractor::new(page_with_all_records.clone().html.unwrap());


        let company_info_list = extractor.get_company_info_houzz();

        for company_info in company_info_list {
            let link = LinksToRecordDetails {
                id: 0,
                pages_with_all_records_id: page_with_all_records.id,
                company: company_info.company,
                link: company_info.link,
                visited: 0,
            };

            println!("Link: {:?}", link);

            match LinksToRecordDetails::create_record(&pool, &link).await {
                Ok(_) => {
                    println!("Inserted and sleeping for");
                },
                Err(e) => {
                    // Log the error and continue with the next iteration
                    eprintln!("Error inserting record: {:?}", e);
                }
            }
        }

        PagesWithAllRecords::mark_record_as_processed(&page_with_all_records, &pool).await?;
    }

    Ok(())
}

pub async fn populate_records_data_from_records_html(pool: &MySqlPool) -> Result<(), Error>{
    let records_html = RecordsHtml::get_all_unprocessed_records(&pool).await?;

    for record_html in records_html {
        let extractor = Extractor::new(record_html.html);

        let record_data = extractor.get_company_details_houzz();


        let records_data = RecordsData {
            id: 0,
            records_html_id: record_html.id,
            email: "".to_string(),
            phone: record_data.phone,
            website: record_data.website,
            contact_us_link: Some("".to_string()),
        };


        if records_data.website != ""{
            let record_exists_by_website = match RecordsData::record_exists_by_website(&pool, &records_data.website).await{
                Ok(exists) => exists,
                Err(e) => {
                    eprintln!("Error checking if record exists by website: {:?}", e);
                    continue;
                }
            };
    
            if record_exists_by_website {
                println!("Record already exists, skipping");
                continue;
            }
        }

        if records_data.phone != ""{
            let record_exists_by_phone = match RecordsData::record_exists_by_phone(&pool, &records_data.phone).await{
                Ok(exists) => exists,
                Err(e) => {
                    eprintln!("Error checking if record exists by phone: {:?}", e);
                    continue;
                }
            };
    
            if record_exists_by_phone {
                println!("Record already exists, skipping");
                continue;
            }
        }


        match RecordsData::create_record(&pool, &records_data).await {
            Ok(_) => {
                println!("Inserted and sleeping for");
            },
            Err(e) => {
                // Log the error and continue with the next iteration
                eprintln!("Error inserting record: {:?}", e);
            }
        }

    }

    Ok(())

}

pub async fn fix_records_websites(pool: &MySqlPool) -> Result<(), Error>{
    let records_data = RecordsData::get_all_records(&pool).await?;

    for record_data in records_data {
        let mut records_data = record_data.clone();
        let mut extractor = Extractor::new(record_data.website.clone());

        if records_data.website == ""{
            continue;
        }

        if !records_data.website.contains("http"){
            records_data.website = format!("https://{}", records_data.website);
        }


        match RecordsData::update_website(&pool, &records_data).await {
            Ok(_) => {
                println!("Updated and sleeping for");
            },
            Err(e) => {
                // Log the error and continue with the next iteration
                eprintln!("Error updating record: {:?}", e);
            }
        }

    }

    Ok(())
}

pub async fn insert_website_html_from_records_data_websites(semaphore: Arc<Semaphore>, scheduler_clone: Arc<Mutex<scheduler::Scheduler>>, pool: &MySqlPool, urls: Vec<UrlDataRecord>) -> Result<(), Error>{

    let tasks: Vec<_> = urls
    .into_iter()
    .map(|url_data: UrlDataRecord| {
        let semaphore = Arc::clone(&semaphore);
        let scheduler_clone = Arc::clone(&scheduler_clone);
        let pool = pool.clone();
        tokio::spawn(async move {

            // Acquire a permit from the semaphore.
            let _permit = semaphore.acquire().await;

            match InvalidWebsites::record_exists(&pool, &url_data.url).await{
                Ok(exists) => {
                    if exists {
                        println!("Website is invalid, skipping");
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("Error checking if record exists: {:?}", e);
                    return;
                }
            }
            let record_exists = match WebsitesHtml::website_exists(&pool, &url_data.url).await{
                Ok(exists) => exists,
                Err(e) => {
                    eprintln!("Error checking if record exists: {:?}", e);
                    return;
                }
            };

            if record_exists {
                println!("Record already exists, skipping");
                return;
            }

            // Try to get a client.
            let client;
            loop {
                let mut locked_scheduler = scheduler_clone.lock().await;
                match locked_scheduler.get_client().await {
                    Ok(available_client) => {
                        client = available_client.clone();
                        break;
                    },
                    Err(_) => {
                        println!("No available clients, retrying in 5 seconds...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }

            let scrapper = scrapper::Scrapper::new(&client);
            let mut body: String = "".to_string(); 

            match scrapper.get_body(&url_data.url).await {
                Ok(b) => {
                    body = b
                },
                Err(e) => {
                    eprintln!("Error getting body: {:?}", e);
                    println!("Website:{}", url_data.url);
                    if format!("{:?}", e).contains("ERR_NAME_NOT_RESOLVED") || 
                        format!("{:?}",e).contains("no element found matching selector: no such element: Unable to locate element:") || 
                        format!("{:?}",e).contains("ERR_ADDRESS_UNREACHABLE"){                       
                        let invalid_website = InvalidWebsites {
                            website: url_data.url.clone(),
                        };

                        match InvalidWebsites::create_record(&pool, &invalid_website).await {
                            Ok(_) => {
                                println!("Found invalid website");
                                let mut locked_scheduler = scheduler_clone.lock().await;
                                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                                    println!("Failed to release client: {}", e);
                                }
                            },
                            Err(e) => {
                                // Log the error and continue with the next iteration
                                eprintln!("Error inserting record: {:?}", e);
                                let mut locked_scheduler = scheduler_clone.lock().await;
                                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                                    println!("Failed to release client: {}", e);
                                }
                                return;
                            }
                        }
                    }else if format!("{:?}", e).contains("ERR_SSL_VERSION_OR_CIPHER_MISMATCH") || format!("{:?}",e).contains("ERR_SSL_PROTOCOL_ERROR") {
                        let http_url = url_data.url.replace("https", "http");

                        match scrapper.get_body(&http_url).await {
                            Ok(b) => {
                                body = b
                            },
                            Err(e) => {
                                let mut locked_scheduler = scheduler_clone.lock().await;
                                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                                    println!("Failed to release client: {}", e);
                                }
                                return;
                            }
                        }


                    }else{
                        let mut locked_scheduler = scheduler_clone.lock().await;

                        match locked_scheduler.replace_client(&client, false).await{
                            Ok(_) => {
                                println!("Replaced client");
                            },
                            Err(e) => {
                                println!("Failed to replace client: {}", e);
                            }
                        }
    
                        drop(_permit);
                        return;
                    }

                }
            };

            if body == ""{
                println!("Body empty.");
                let mut locked_scheduler = scheduler_clone.lock().await;
                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                    println!("Failed to release client: {}", e);
                }
                return;
            }
            let record_html = WebsitesHtml {
                id: 0,
                website: url_data.url.clone(),
                main_page_html: body.clone().to_string(),
                contact_page_html: "".to_string(),
                records_data_id: url_data.record_id,
            };

            match WebsitesHtml::create_record(&pool, &record_html).await {
                Ok(_) => {
                    println!("Inserted and sleeping for");
                },
                Err(e) => {
                    // Log the error and continue with the next iteration
                    eprintln!("Error inserting record: {:?}", e);
                }
            }
    
            let sleep_time = rand::thread_rng().gen_range(1..3);
            println!("Sleeping for {} seconds", sleep_time);
            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;


            {
                let mut locked_scheduler = scheduler_clone.lock().await;
                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                    println!("Failed to release client: {}", e);
                }
            }

        })
    })
    .collect();


    for task in tasks {
        task.await.unwrap();
    }

    Ok(())
}

pub async fn update_contact_us_link_from_website_html(pool: &MySqlPool) -> Result<(), Error>{
    let websites_html = WebsitesHtml::get_all_websites(&pool).await?;

    for website_html in websites_html {
        let mut website_html = website_html.clone();

        if website_html.contact_page_html != ""{
            continue;
        }

        let mut extractor = Extractor::new(website_html.main_page_html.clone());

        let contact_us_link = match extractor.find_contact_us_link(){
            Some(link) => link,
            None => "".to_string(),
        };

        if contact_us_link == ""{
            continue;
        }

        let mut records_data = RecordsData::get_record_data_by_records_data_id(&pool, website_html.records_data_id).await?;
        records_data.contact_us_link = Some(contact_us_link.clone());


        let ( domain, authority ) = match parse_url(&website_html.website){
            Some((sld, tld)) => (sld, tld),
            None => ("", ""),
        };

        if domain == "" || authority == ""{
            continue;
        }

        let mut absolute_url = "".to_string();

        if contact_us_link.contains(domain){
            absolute_url = contact_us_link;
        }else{
            if contact_us_link.starts_with("/") {
                absolute_url = format!("{}://{}.{}{}", "https", domain, authority, contact_us_link);
            }else{
                absolute_url = format!("{}://{}.{}/{}", "https", domain, authority, contact_us_link.replace("/", ""));
            }
        }

        records_data.contact_us_link = Some(absolute_url.clone());
        
        match RecordsData::set_contact_us_link(&pool, &records_data).await {
            Ok(_) => {
                println!("Updated and sleeping for");
            },
            Err(e) => {
                // Log the error and continue with the next iteration
                eprintln!("Error updating record: {:?}", e);
            }
        }

    }

    Ok(())
}

fn parse_url(url: &str) -> Option<(&str, &str)> {
    let without_protocol = url.trim_start_matches("http://").trim_start_matches("https://");
    let domain_end = without_protocol.find('/').unwrap_or_else(|| without_protocol.len());
    let domain = &without_protocol[..domain_end];
    let parts: Vec<&str> = domain.split('.').collect();
    let len = parts.len();

    if len < 2 {
        None
    } else {
        let sld = parts[len - 2];
        let tld = parts[len - 1];

        Some((sld, tld))
    }
}

pub async fn update_contact_page_html_from_websites_html(semaphore: Arc<Semaphore>, scheduler_clone: Arc<Mutex<scheduler::Scheduler>>, pool: MySqlPool, urls: Vec<WebsitesHtmlData>) -> Result<(), Error>{

    let tasks: Vec<_> = urls
    .into_iter()
    .map(|url_data: WebsitesHtmlData| {
        let semaphore = Arc::clone(&semaphore);
        let scheduler_clone = Arc::clone(&scheduler_clone);
        let pool = pool.clone();
        tokio::spawn(async move {

            let _permit = semaphore.acquire().await;

            let record_data = RecordsData::get_record_data_by_records_data_id(&pool, url_data.record_id).await.unwrap();

            let contact_us_link = match record_data.contact_us_link{
                Some(link) => link,
                None => "".to_string(),
            };

            println!("ID: {:?}", url_data.website_html_id);
            println!("Website: {:?}", url_data.website);

            if contact_us_link == ""{
                println!("Contact us link is empty, skipping");
                return;
            }

            let client;
            loop {
                let mut locked_scheduler = scheduler_clone.lock().await;
                match locked_scheduler.get_client().await {
                    Ok(available_client) => {
                        client = available_client.clone();
                        break;
                    },
                    Err(_) => {
                        println!("No available clients, retrying in 5 seconds...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }

            println!("Client id: {:?}", client.session_id().await);

            let scrapper = scrapper::Scrapper::new(&client);

            let mut body: String = "".to_string();

            // Try to get the body.
            match scrapper.get_body(&contact_us_link).await {
                Ok(b) => {
                    body = b
                },
                Err(e) => {
                    eprintln!("Error getting body: {:?}", e);
                    println!("Website: {}", contact_us_link);

                    let error_string = format!("{:?}", e);

                    if error_string.contains("ERR_NAME_NOT_RESOLVED") || 
                        error_string.contains("no element found matching selector: no such element: Unable to locate element:") || 
                        error_string.contains("ERR_ADDRESS_UNREACHABLE"){                       
                        let invalid_website = InvalidWebsites {
                            website: contact_us_link.clone(),
                        };
            
                        match InvalidWebsites::create_record(&pool, &invalid_website).await {
                            Ok(_) => {
                                println!("Inserted invalid website");

                            },
                            Err(e) => {
                                // Log the error and continue with the next iteration
                                eprintln!("Error inserting invalid website: {:?}", e);
                            }
                        }

                        let mut locked_scheduler = scheduler_clone.lock().await;
                        if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                            println!("Failed to release client: {}", e);
                        }

                        return;
                    } else if error_string.contains("ERR_SSL_VERSION_OR_CIPHER_MISMATCH") || error_string.contains("ERR_SSL_PROTOCOL_ERROR") {
                        let http_contact_us_link = contact_us_link.replace("https", "http");
            
                        match scrapper.get_body(&http_contact_us_link).await {
                            Ok(b) => {
                                body = b
                            },
                            Err(e) => {
                                let mut locked_scheduler = scheduler_clone.lock().await;
                                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                                    println!("Failed to release client: {}", e);
                                }
                                return;
                            }
                        }
                    } else {
                        let mut locked_scheduler = scheduler_clone.lock().await;
            
                        match locked_scheduler.replace_client(&client, true).await{
                            Ok(_) => {
                                println!("Replaced client");
                            },
                            Err(e) => {
                                println!("Failed to replace client: {}", e);
                            }
                        }
            
                        drop(_permit);
                        return;
                    }
                }
            };

            if body == ""{
                println!("Body empty.");
                let mut locked_scheduler = scheduler_clone.lock().await;
                if let Err(e) = locked_scheduler.replace_client(&client, false).await {
                    println!("Failed to release client: {}", e);
                }
                return;
            }

            let website = WebsitesHtml {
                id: url_data.website_html_id,
                website: url_data.website,
                main_page_html: "".to_string(),
                contact_page_html: body.clone().to_string(),
                records_data_id: url_data.record_id,
            };

            match WebsitesHtml::update_contact_page_html(&pool, &website).await {
                Ok(_) => {
                    println!("Updated!");
                },
                Err(e) => {
                    // Log the error and continue with the next iteration
                    eprintln!("Error inserting record: {:?}", e);
                }
            }
    
            let sleep_time = rand::thread_rng().gen_range(1..3);
            println!("Sleeping for {} seconds", sleep_time);
            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;


            {
                let mut locked_scheduler = scheduler_clone.lock().await;
                if let Err(e) = locked_scheduler.replace_client(&client, true).await {
                    println!("Failed to release client: {}", e);
                }
            }


            println!("Releasing permit");
        })
    })
    .collect();

    for task in tasks {
        task.await.unwrap();
    }
    
    Ok(())
}

pub async fn update_record_data_email(pool: &MySqlPool) -> Result<(), Error> {
    let websites_html = WebsitesHtml::get_all_websites(pool).await?;

    for website_html in websites_html {
        let mut extractor = Extractor::new(website_html.main_page_html);
        let emails = extractor.find_emails_by_regex();

        extractor.set_html(website_html.contact_page_html);
        let emails_from_contact_page = extractor.find_emails_by_regex();

        // Combine emails from both pages and remove duplicates
        let all_emails_combined = format!("{},{}", emails, emails_from_contact_page);
        let mut all_emails: Vec<String> = all_emails_combined.split(',')
                                                             .map(String::from)
                                                             .collect();

        // Remove duplicates while maintaining order
        let mut seen = HashSet::new();
        all_emails.retain(|email| seen.insert(email.clone()));

        // Join the emails back into a single string
        let unique_emails = all_emails.join(", ");

        let record_data = RecordsData {
            id: website_html.records_data_id,
            records_html_id: 0,
            email: unique_emails,
            phone: "".to_string(),
            website: "".to_string(),
            contact_us_link: Some("".to_string()),
        };

        RecordsData::update_email(&pool, &record_data).await?;

        println!("Record updated");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}

pub async fn run_get_all_pages_houzz(pool: MySqlPool, houzz_data_record: HouzzEntry) -> Result<(), Error> {

    let clients = generate_clients(true, 5).await?;
    let scheduler = scheduler::Scheduler::new(clients.clone());
    let mut urls: Vec<UrlData> = Vec::new();

    println!("Getting data");


    for page in 334..500 {
        let url = houzz_data_record.link.clone();
        let page_url = format!("{}?fi={}", url, page * 15);
        println!("Page URL: {}", page_url);
        urls.push(UrlData{
            url: page_url,
            page: page,
        });
    }

    let scheduler_clone = Arc::new(Mutex::new(scheduler));
    let semaphore = Arc::new(Semaphore::new(5));

    get_all_pages_houzz(semaphore, scheduler_clone, houzz_data_record, pool.clone(), urls).await;

    Ok(())
}

pub async fn run_get_all_records_html_from_links(pool: MySqlPool) -> Result<(), Error> {

    let clients = generate_clients(false, 10).await?;
    let scheduler = scheduler::Scheduler::new(clients.clone());
    let mut urls: Vec<UrlDataLinks> = Vec::new();

    let links_to_record_details = LinksToRecordDetails::get_all_unvisited_records(&pool).await?;

    for link_to_record_details in links_to_record_details {
        let url = link_to_record_details.link.clone();
        urls.push(UrlDataLinks{
            url: url,
            link_to_record_details_id: link_to_record_details.id,
        });
    }

    println!("url length: {:?}", urls.len());
    let scheduler_clone = Arc::new(Mutex::new(scheduler));
    let semaphore = Arc::new(Semaphore::new(10));

    get_all_records_html_from_links(semaphore, scheduler_clone, pool.clone(), urls).await?;

    Ok(())
}

pub async fn run_insert_website_html_from_records_data_websites(pool: &MySqlPool) -> Result<(), Error> {

    let clients = generate_clients(false, 10).await?;
    let scheduler = scheduler::Scheduler::new(clients.clone());
    let mut urls: Vec<UrlDataRecord> = Vec::new();

    let records_data = RecordsData::get_all_records_houzz(&pool).await?;

    for record_data in records_data {
        let url = record_data.website.clone();
        urls.push(UrlDataRecord{
            url: url,
            record_id: record_data.id,
        });
    }

    let scheduler_clone = Arc::new(Mutex::new(scheduler));
    let semaphore = Arc::new(Semaphore::new(10));

    insert_website_html_from_records_data_websites(semaphore, scheduler_clone, pool, urls).await?;

    Ok(())
}

async fn run_update_contact_page_html_from_websites_html(pool: MySqlPool) -> Result<(), Error> {

    let clients = generate_clients(false, 10).await?;
    let scheduler = scheduler::Scheduler::new(clients.clone());

    let scheduler_clone = Arc::new(Mutex::new(scheduler));
    let semaphore = Arc::new(Semaphore::new(10));

    let websites_html = WebsitesHtml::get_all_websites_with_no_contact_page_html(&pool).await?;

    let mut urls: Vec<WebsitesHtmlData> = Vec::new();

    for website_html in websites_html {
        urls.push(WebsitesHtmlData{
            website: website_html.website,
            website_html_id: website_html.id,
            record_id: website_html.records_data_id,
        });
    }

    update_contact_page_html_from_websites_html(semaphore, scheduler_clone, pool, urls).await?;

    Ok(())
}

/*

SELECT *
INTO OUTFILE '/var/lib/mysql-files/view_records_with_districts_design_builder.csv'
FIELDS TERMINATED BY ',' 
ENCLOSED BY '"'
LINES TERMINATED BY '\n'
FROM view_records_with_districts
WHERE districts LIKE '%General Contractors%';

SELECT 
    links_to_record_details.company, 
    links_to_record_details.link, 
    records_data.email, 
    records_data.phone
INTO OUTFILE '/var/lib/mysql-files/records_without_website_gc.csv'
FIELDS TERMINATED BY ',' 
ENCLOSED BY '"'
LINES TERMINATED BY '\n'
FROM records_data
JOIN records_html ON records_data.records_html_id = records_html.id
JOIN links_to_record_details ON records_html.link_to_record_details_id = links_to_record_details.id
JOIN pages_with_all_records ON links_to_record_details.pages_with_all_records_id = pages_with_all_records.id
WHERE pages_with_all_records.district LIKE '%General Contractors%' AND records_data.website = '';
*/