use anyhow::Result;
use grocy::GrocyApi;
use inquire::{Select, Text};
use lidl::structs::{ReceiptDetailed, ReceiptItem, Store};

use crate::GrocyConfig;

pub(super) fn purchase_lidl_products(
    config: &mut GrocyConfig,
    receipt: ReceiptDetailed<f64>,
) -> Result<()> {
    let grocy_api = init_grocy_api(config)?;
    let store_id = get_store_id(config, &grocy_api, &receipt.store)?;
    dbg!(store_id);

    let skipped_products = receipt
        .items_line
        .into_iter()
        .filter(|item| purchase_lidl_product(&grocy_api, store_id, item).is_err());

    todo!();
}

fn init_grocy_api(config: &mut GrocyConfig) -> Result<GrocyApi> {
    let base_url = match &config.base_url {
        Some(url) => url,
        None => {
            let url = Text::new("Please enter your Grocy instance's url:")
                .with_placeholder("https://grocy.example.com")
                .with_help_message("Do not include a forward slash at the end")
                .prompt()?;
            config.base_url = Some(url);
            config.base_url.as_ref().unwrap()
        }
    };
    let api_key = match &config.api_key {
        Some(key) => key,
        None => {
            let key = Text::new("Please enter the Grocy's API key:")
                .with_help_message(
                    "Can be generated through the web interface, in Settings -> Manage API Keys",
                )
                .prompt()?;
            config.api_key = Some(key);
            config.api_key.as_ref().unwrap()
        }
    };

    GrocyApi::new(base_url, api_key)
}

fn get_store_id(config: &mut GrocyConfig, grocy_api: &GrocyApi, store: &Store) -> Result<u32> {
    let available_locations = grocy_api.get_shopping_locations()?;
    match config.stores.get(&store.id) {
        Some(id) => Ok(*id),
        None => {
            let location = Select::new("Select store for this receipt:", available_locations)
                .with_help_message(&format!("Store name from receipt: {}", store.name))
                .prompt()?;
            config.stores.insert(store.id.clone(), location.id);
            Ok(location.id)
        }
    }
}

fn purchase_lidl_product(
    grocy_api: &GrocyApi,
    store_id: u32,
    product: &ReceiptItem<f64>,
) -> Result<()> {
    todo!()
}
