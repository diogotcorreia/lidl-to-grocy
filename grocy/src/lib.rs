use anyhow::Result;
use chrono::NaiveDate;
use reqwest::{blocking::Client, header::HeaderMap};
use structs::{
    AddProductBarcodePayload, Location, ObjectCreated, Product, ProductDetails,
    PurchaseProductPayload, QuantityUnit, ShoppingLocation, Transaction,
};

pub mod structs;

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

    pub fn get_product_by_barcode(&self, barcode: &str) -> Result<ProductDetails> {
        Ok(self
            .client
            .get(format!(
                "{}/api/stock/products/by-barcode/{}",
                self.base_url, barcode
            ))
            .send()?
            .json()?)
    }

    pub fn get_all_products(&self) -> Result<Vec<Product>> {
        Ok(self
            .client
            .get(format!("{}/api/objects/products", self.base_url))
            .query(&[("query[]", "active=1")])
            .send()?
            .json()?)
    }

    pub fn get_quantity_unit(&self, id: u32) -> Result<QuantityUnit> {
        Ok(self
            .client
            .get(format!(
                "{}/api/objects/quantity_units/{}",
                self.base_url, id
            ))
            .send()?
            .json()?)
    }

    pub fn create_product_barcode(
        &self,
        product_id: u32,
        barcode: &str,
        quantity: Option<f64>,
        quantity_unit_id: Option<u32>,
        shopping_location_id: u32,
        note: Option<&str>,
    ) -> Result<ObjectCreated> {
        Ok(self
            .client
            .post(format!("{}/api/objects/product_barcodes", self.base_url))
            .json(&AddProductBarcodePayload {
                product_id,
                barcode,
                amount: quantity,
                qu_id: quantity_unit_id,
                shopping_location_id,
                note,
            })
            .send()?
            .json()?)
    }

    pub fn get_locations(&self) -> Result<Vec<Location>> {
        Ok(self
            .client
            .get(format!("{}/api/objects/locations", self.base_url))
            .query(&[("query[]", "active=1")])
            .send()?
            .json()?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn purchase_product(
        &self,
        product: u32,
        amount: f64,
        best_before_date: Option<NaiveDate>,
        purchased_date: Option<NaiveDate>,
        price: Option<f64>,
        location: Option<u32>,
        shopping_location: Option<u32>,
        note: Option<&str>,
    ) -> Result<Vec<Transaction>> {
        Ok(self
            .client
            .post(format!(
                "{}/api/stock/products/{}/add",
                self.base_url, product
            ))
            .json(&PurchaseProductPayload {
                amount,
                best_before_date: best_before_date
                    .map_or("2999-12-31".to_string(), |date| date.to_string())
                    .as_ref(),
                purchased_date: purchased_date.map(|date| date.to_string()).as_deref(),
                transaction_type: "purchase",
                price,
                location_id: location,
                shopping_location_id: shopping_location,
                stock_label_type: 0,
                note,
            })
            .send()?
            .json()?)
    }
}
