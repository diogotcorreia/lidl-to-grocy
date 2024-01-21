use anyhow::Result;
use reqwest::{blocking::Client, header::HeaderMap};
use structs::ShoppingLocation;

mod structs;

const GROCY_TOKEN_HEADER: &str = "GROCY-API-KEY";

pub struct GrocyApi {
    client: Client,
    base_url: String,
}

impl GrocyApi {
    pub fn new(base_url: &str, api_key: &str) -> Result<GrocyApi> {
        let mut headers = HeaderMap::new();
        headers.append(GROCY_TOKEN_HEADER, api_key.parse()?);
        Ok(Self {
            base_url: base_url.to_string(),
            client: reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    pub fn get_shopping_locations(&self) -> Result<Vec<ShoppingLocation>> {
        Ok(self
            .client
            .get(format!("{}/api/objects/shopping_locations", self.base_url))
            .query(&[("query[]", "active=1")])
            .send()?
            .json()?)
    }
}
