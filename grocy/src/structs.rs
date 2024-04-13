use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize};

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
    pub quantity_unit_stock: QuantityUnit,
    pub default_quantity_unit_purchase: QuantityUnit,
    pub qu_conversion_factor_purchase_to_stock: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    #[serde(deserialize_with = "deserialize_fallible")]
    pub location_id: Option<u32>,
    pub default_best_before_days: i32,
    #[serde(deserialize_with = "deserialize_fallible")]
    pub qu_id_purchase: Option<u32>,
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
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityUnit {
    pub id: u32,
    pub name: String,
    pub name_plural: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserField {
    pub id: u32,
    pub entity: String,
    pub name: String,
    pub caption: String,
    pub r#type: String,
    pub input_required: u32, // effectively a bool
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProductBarcodePayload<'a> {
    pub product_id: u32,
    pub barcode: &'a str,
    pub amount: Option<f64>,
    pub qu_id: Option<u32>,
    pub shopping_location_id: u32,
    pub note: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCreated {
    pub created_object_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBarcodeLastPricePayload {
    pub last_price: f64,
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
    pub note: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u32,
}

// e.g., for when deserializing -1 into an Option<u32>
// should return None instead of raising an error
fn deserialize_fallible<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Deserialize::deserialize(deserializer).ok())
}
