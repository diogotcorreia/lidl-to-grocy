use std::fmt::Display;

use anyhow::Result;
use chrono::NaiveDateTime;

pub trait StoreApi {
    fn get_available_receipts(&self) -> Result<Vec<ReceiptSummary>>;
    fn get_specific_receipt(&self, receipt: &ReceiptSummary) -> Result<ReceiptDetailed>;
}

#[derive(Debug, PartialEq)]
pub struct Currency {
    /// ISO 4217 code of currency
    pub id: String,
    /// User facing symbol of currency
    pub symbol: String,
}

/// Used when listing available receipts; has minimal information
#[derive(Debug, PartialEq)]
pub struct ReceiptSummary {
    pub id: String,
    pub date: NaiveDateTime,
    pub currency: Currency,
    pub total_amount: f64,
    pub articles_count: Option<u32>,
}

impl Display for ReceiptSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date.format("%a %b %e %Y %T"))?;
        if let Some(articles_count) = self.articles_count {
            write!(f, " - {} product(s)", articles_count)?;
        }
        write!(f, " - {} {}", self.total_amount, self.currency.symbol)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Store {
    pub id: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Discount {
    pub amount: f64,
}

#[derive(Debug, PartialEq)]
pub struct ReceiptItem {
    /// Price per 1 of quantity
    pub unit_price: f64,
    /// Quantity is a float for weight-based products
    pub quantity: f64,
    pub is_weight: bool,
    pub name: String,
    pub barcode: String,
    pub discounts: Vec<Discount>,
}

#[derive(Debug, PartialEq)]
pub struct ReceiptDetailed {
    pub id: String,
    pub items: Vec<ReceiptItem>,
    pub date: NaiveDateTime,
    pub currency: Currency,
    pub store: Store,
}
