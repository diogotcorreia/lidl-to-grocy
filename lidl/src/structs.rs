use std::{fmt::Display, num::ParseFloatError};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub id: String,
    default_name: String,
    active: bool,
    languages: Vec<Language>,
}

impl Country {
    pub fn get_default_language(&self) -> Option<Language> {
        self.languages
            .iter()
            .find(|language| language.active)
            .cloned()
    }
}

impl Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.id, self.default_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub id: String,
    default_name: String,
    active: bool,
    default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReceiptsPage {
    #[serde(rename = "tickets")]
    pub receipts: Vec<Receipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Receipt {
    pub id: String,
    pub date: DateTime<Utc>,
    pub currency: Currency,
    pub total_amount: f64,
    pub store_code: String,
    pub articles_count: u32,
}

impl From<Receipt> for ir::ReceiptSummary {
    fn from(value: Receipt) -> Self {
        Self {
            id: value.id,
            date: value.date.naive_utc(), // Lidl sends local date with utc offset (which is wrong)
            currency: value.currency.into(),
            total_amount: value.total_amount,
            articles_count: Some(value.articles_count),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Currency {
    pub code: String,
    pub symbol: String,
}

impl From<Currency> for ir::Currency {
    fn from(value: Currency) -> Self {
        Self {
            id: value.code,
            symbol: value.symbol,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// The Lidl API returns floats and integers as strings,
// so we generalize this struct to accept both and convert
// everything to floats after deserializing
pub(crate) struct ReceiptDetailed<T> {
    pub id: String,
    pub items_line: Vec<ReceiptItem<T>>,
    pub date: NaiveDateTime,
    pub total_amount_numeric: f64,
    pub currency: Currency,
    pub store: Store,
}

impl TryFrom<ReceiptDetailed<String>> for ReceiptDetailed<f64> {
    type Error = ParseFloatError;

    fn try_from(value: ReceiptDetailed<String>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            items_line: value
                .items_line
                .into_iter()
                .map(|item| item.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            date: value.date,
            total_amount_numeric: value.total_amount_numeric,
            currency: value.currency,
            store: value.store,
        })
    }
}

impl From<ReceiptDetailed<f64>> for ir::ReceiptDetailed {
    fn from(value: ReceiptDetailed<f64>) -> Self {
        Self {
            id: value.id,
            items: value.items_line.into_iter().map(Into::into).collect(),
            date: value.date,
            currency: value.currency.into(),
            store: value.store.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReceiptItem<T> {
    pub current_unit_price: T,
    pub quantity: T,
    pub is_weight: bool,
    pub original_amount: T,
    pub name: String,
    pub code_input: String,
    pub discounts: Vec<Discount<T>>,
}

impl TryFrom<ReceiptItem<String>> for ReceiptItem<f64> {
    type Error = ParseFloatError;

    fn try_from(value: ReceiptItem<String>) -> Result<Self, Self::Error> {
        Ok(Self {
            current_unit_price: value.current_unit_price.replace(',', ".").parse()?,
            quantity: value.quantity.replace(',', ".").parse()?,
            is_weight: value.is_weight,
            original_amount: value.original_amount.replace(',', ".").parse()?,
            name: value.name,
            code_input: value.code_input,
            discounts: value
                .discounts
                .into_iter()
                .map(|discount| discount.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<ReceiptItem<f64>> for ir::ReceiptItem {
    fn from(value: ReceiptItem<f64>) -> Self {
        Self {
            unit_price: value.current_unit_price,
            quantity: value.quantity,
            is_weight: value.is_weight,
            name: value.name,
            barcode: value.code_input,
            discounts: value.discounts.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Discount<T> {
    pub description: String,
    pub amount: T,
}

impl TryFrom<Discount<String>> for Discount<f64> {
    type Error = ParseFloatError;

    fn try_from(value: Discount<String>) -> Result<Self, Self::Error> {
        Ok(Self {
            description: value.description,
            amount: value.amount.replace(',', ".").parse()?,
        })
    }
}

impl From<Discount<f64>> for ir::Discount {
    fn from(value: Discount<f64>) -> Self {
        Self {
            amount: value.amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Store {
    pub id: String,
    pub name: String,
}

impl From<Store> for ir::Store {
    fn from(value: Store) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Response of the v3 ticket endpoint
pub(crate) struct UnifiedReceiptDetailed {
    pub id: String,
    pub date: NaiveDateTime,
    pub store: Store,
    pub html_printed_receipt: String,
}
