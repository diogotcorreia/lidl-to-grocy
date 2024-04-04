use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingLocation {
    pub id: u32,
    pub name: String,
}

impl Display for ShoppingLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetails {
    pub product: Product,
    pub product_barcodes: Vec<ProductBarcode>,
    pub default_quantity_unit_purchase: QuantityUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub location_id: u32,
    pub default_best_before_days: i32,
    pub qu_id_purchase: u32,
}

impl Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductBarcode {
    pub id: u32,
    pub product_id: u32,
    pub barcode: String,
    pub qu_id: Option<u32>,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityUnit {
    pub id: u32,
    pub name: String,
    pub name_plural: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProductBarcodePayload<'a> {
    pub product_id: u32,
    pub barcode: &'a str,
    pub amount: Option<f64>,
    pub qu_id: Option<u32>,
    pub shopping_location_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCreated {
    pub created_object_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: u32,
    pub name: String,
    pub is_freezer: u32, // effectively a bool
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_freezer != 0 {
            write!(f, "{} ❄️", self.name)
        } else {
            self.name.fmt(f)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseProductPayload<'a> {
    pub amount: f64,
    pub best_before_date: &'a str,
    pub purchased_date: Option<&'a str>,
    pub transaction_type: &'a str,
    pub price: Option<f64>,
    pub location_id: Option<u32>,
    pub shopping_location_id: Option<u32>,
    pub stock_label_type: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u32,
}
