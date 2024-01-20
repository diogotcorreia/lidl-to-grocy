use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::lidl::fetch_receipt_from_lidl;

mod error;
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
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct GrocyConfig {
    api_key: Option<String>,
}

fn main() -> Result<()> {
    let mut cfg: Config = confy::load(CONFIG_NAME, Some(CONFIG_NAME))?;
    dbg!(&cfg);

    fetch_receipt_from_lidl(&mut cfg.lidl)?;

    confy::store(CONFIG_NAME, Some(CONFIG_NAME), cfg)?;

    Ok(())
}
