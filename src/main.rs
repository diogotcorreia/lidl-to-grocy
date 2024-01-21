use std::collections::HashMap;

use anyhow::Result;
use grocy::purchase_lidl_products;
use serde::{Deserialize, Serialize};

use crate::lidl::fetch_receipt_from_lidl;

mod error;
mod grocy;
mod lidl;

const CONFIG_NAME: &str = "lidl-to-grocy";

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    lidl: LidlConfig,
    grocy: GrocyConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct LidlConfig {
    refresh_token: Option<String>,
    locale: Option<LidlLocale>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct LidlLocale {
    country: String,
    language: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct GrocyConfig {
    base_url: Option<String>,
    api_key: Option<String>,
    stores: HashMap<String, u32>,
}

fn main() -> Result<()> {
    let mut cfg: Config = confy::load(CONFIG_NAME, Some(CONFIG_NAME))?;

    let receipt = fetch_receipt_from_lidl(&mut cfg.lidl)?;
    dbg!(&receipt);

    purchase_lidl_products(&mut cfg.grocy, receipt)?;

    confy::store(CONFIG_NAME, Some(CONFIG_NAME), cfg)?;

    Ok(())
}
