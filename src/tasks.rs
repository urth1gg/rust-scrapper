use sqlx::{MySqlPool, Row};
use anyhow::Result;
use crate::scrapper::Scrapper;
use crate::data::{self, HouzzEntry};

pub struct Tasks{

}

impl Tasks{
    pub async fn get_pages_with_all_records<'a>(pool: &MySqlPool, scrapper: &'a Scrapper<'_>) -> Result<()> {
        let houzz_data: [HouzzEntry; 8] = data::generate_houzz_input_records();

        let houzz_data = houzz_data.into_iter().filter(|record| record.category == "Home Builders in Ontario - Houzz").collect::<Vec<_>>();

        println!("Houzz data: {:?}", houzz_data);

        Ok(())
    }
    
}