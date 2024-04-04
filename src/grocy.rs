use colored::Colorize;
use std::{collections::HashMap, fmt::Display};

use anyhow::Result;
use chrono::{Duration, NaiveDate};
use grocy::{
    structs::{Location, ProductDetails},
    GrocyApi,
};
use inquire::{Confirm, CustomType, DateSelect, MultiSelect, Select, Text};
use lidl::structs::{ReceiptDetailed, ReceiptItem, Store};

use crate::{dynprompt, error::Error, GrocyConfig};

struct GrocyState {
    api: GrocyApi,
    locations: Vec<Location>,
}

pub(super) fn purchase_lidl_products(
    config: &mut GrocyConfig,
    receipt: ReceiptDetailed<f64>,
) -> Result<()> {
    let grocy_api = init_grocy_api(config)?;
    let store_id = get_store_id(config, &grocy_api, &receipt.store)?;

    let locations = grocy_api.get_locations()?;
    let grocy_state = GrocyState {
        api: grocy_api,
        locations,
    };

    let skipped_products: Vec<_> = receipt
        .items_line
        .into_iter()
        .filter(|item| {
            match purchase_lidl_product(&grocy_state, store_id, item, receipt.date.date()) {
                Ok(_) => false,
                Err(error) => {
                    println!("{}", error);
                    true
                }
            }
        })
        .collect();

    if !skipped_products.is_empty() {
        println!();
        println!("{}", "The following products were skipped:".on_red());
        for product in skipped_products {
            println!(
                "- {} {}",
                format!("{}x", product.quantity).yellow(),
                product.name.green()
            );
        }
    }

    Ok(())
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
    grocy_state: &GrocyState,
    store_id: u32,
    product: &ReceiptItem<f64>,
    purchase_date: NaiveDate,
) -> Result<()> {
    println!();
    println!();
    println!(
        "Handling product {} {}",
        format!("{}x", product.quantity).yellow(),
        product.name.green()
    );
    let product_details = grocy_state
        .api
        .get_product_by_barcode(&product.code_input)
        .map(|details| {
            println!(
                "Found product on Grocy: {}",
                details.product.name.bright_cyan()
            );
            details
        })
        .or_else(|_| handle_product_without_known_barcode(&grocy_state.api, product, store_id))?;

    let discount: f64 = product
        .discounts
        .iter()
        .map(|discount| discount.amount)
        .sum();
    let default_date = Some(product_details.product.default_best_before_days)
        .filter(|days| *days >= 0)
        .map(|days| chrono::Local::now().date_naive() + Duration::days(days.into()));

    let product_barcode = product_details
        .product_barcodes
        .iter()
        .find(|barcode| barcode.barcode == product.code_input);
    let note = product_barcode.and_then(|barcode| barcode.note.as_deref());

    let price = if product.is_weight {
        let quantity = CustomType::<f64>::new(&format!(
            "Enter quantity for this product ({} kg)",
            product.quantity
        ))
        .with_help_message(&format!(
            "Quantity unit: {}",
            product_details.default_quantity_unit_purchase.name_plural
        ))
        .prompt()?;

        let due_date = prompt_due_date(None, default_date)?;
        let location = prompt_location(grocy_state, product_details.product.location_id)?;

        let price = (product.original_amount - discount) / quantity;

        grocy_state.api.purchase_product(
            product_details.product.id,
            quantity,
            due_date,
            Some(purchase_date),
            Some(price),
            Some(location.id),
            Some(store_id),
            note,
        )?;

        price
    } else {
        let product_barcode_amount = product_barcode
            .and_then(|barcode| barcode.amount)
            .ok_or(Error::BarcodeAmountNotFound)?;

        let quantity = product.quantity.round() as u32;

        let mut last_date = None;
        let due_dates = (1..=quantity)
            .map(|at| {
                last_date = prompt_due_date(Some((at, quantity)), last_date.or(default_date))?;
                Ok(last_date)
            })
            .collect::<Result<Vec<_>>>()?;

        let location = prompt_location(grocy_state, product_details.product.location_id)?;

        let discount_per_item = discount / quantity as f64;

        let price = (product.current_unit_price - discount_per_item) / product_barcode_amount;

        for due_date in due_dates {
            grocy_state.api.purchase_product(
                product_details.product.id,
                product_barcode_amount,
                due_date,
                Some(purchase_date),
                Some(price),
                Some(location.id),
                Some(store_id),
                note,
            )?;
        }

        price
    };

    if let Some(barcode) = product_barcode {
        grocy_state
            .api
            .update_barcode_last_price(barcode.id, price)?;
    }

    Ok(())
}

enum UnknownProductAction {
    AssociateProduct,
    Skip,
}

impl Display for UnknownProductAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssociateProduct => write!(f, "Associate barcode with existing product"),
            Self::Skip => write!(f, "Skip this product"),
        }
    }
}

fn handle_product_without_known_barcode(
    grocy_api: &GrocyApi,
    product: &ReceiptItem<f64>,
    store_id: u32,
) -> Result<ProductDetails> {
    let options = vec![
        UnknownProductAction::AssociateProduct,
        UnknownProductAction::Skip,
    ];

    let action = Select::new(
        &format!(
            "This barcode ({}) is not associated with a product. What do you want to do?",
            product.code_input
        ),
        options,
    )
    .prompt()?;

    match action {
        UnknownProductAction::AssociateProduct => {
            let products = grocy_api.get_all_products()?;

            let selected_product =
                Select::new("Select product to associate barcode with:", products)
                    .prompt_skippable()?
                    .ok_or(Error::SkippedProduct)?;

            let mut quantity = None;
            if !product.is_weight {
                let quantity_unit = grocy_api.get_quantity_unit(selected_product.qu_id_purchase)?;

                quantity = Some(
                    CustomType::<f64>::new("Enter quantity for each barcode:")
                        .with_help_message(&format!("Quantity unit: {}", quantity_unit.name_plural))
                        .with_error_message("Please type a valid number (use dot for decimals)")
                        .prompt()?,
                );
            }

            let note = Text::new("Enter note for this barcode:")
                .with_initial_value(&product.name)
                .prompt_skippable()?;

            let userfields = prompt_barcode_userfields(grocy_api)?;

            grocy_api.create_product_barcode(
                selected_product.id,
                &product.code_input,
                quantity,
                Some(selected_product.qu_id_purchase),
                store_id,
                note.as_deref(),
                userfields,
            )?;

            grocy_api.get_product_by_barcode(&product.code_input)
        }
        UnknownProductAction::Skip => Err(Error::SkippedProduct.into()),
    }
}

fn prompt_due_date(
    progress: Option<(u32, u32)>,
    default_date: Option<NaiveDate>,
) -> Result<Option<NaiveDate>> {
    let prompt = match progress {
        Some((at, total)) => format!("What is the due date of the product? ({}/{})", at, total),
        None => "What is the due date of the product?".to_string(),
    };
    let help_msg = match &default_date {
        Some(date) => format!("Default due date: {}. Press ESC to skip due date. Ctrl + up/down to move by year. Ctrl + left/right to move by month", date),
        None => "This product never expires by default. Press ESC to skip due date. Ctrl + up/down to move by year. Ctrl + left/right to move by month".to_string(),
    };
    Ok(DateSelect::new(&prompt)
        .with_starting_date(default_date.unwrap_or(chrono::Local::now().date_naive()))
        .with_min_date(chrono::Local::now().date_naive())
        .with_help_message(&help_msg)
        .prompt_skippable()?)
}

fn prompt_location(grocy_state: &GrocyState, default_location: u32) -> Result<Location> {
    let locations = grocy_state.locations.clone();
    let default_location_index = locations
        .iter()
        .position(|loc| loc.id == default_location)
        .unwrap_or(0);
    let location = Select::new("Where will the item be stored?", locations)
        .with_starting_cursor(default_location_index)
        .prompt()?;

    Ok(location)
}

fn prompt_barcode_userfields(grocy_api: &GrocyApi) -> Result<HashMap<String, String>> {
    let userfields = grocy_api.get_barcode_userfields()?;
    let mut values = HashMap::new();

    for field in userfields {
        let optional = field.input_required == 0;
        let opt_tag = if optional { "optional" } else { "required" };
        let msg = format!("{} for this barcode ({}):", field.caption, opt_tag);
        let options = field
            .config
            .as_deref()
            .map(|config| config.lines().map(|l| l.to_owned()).collect());

        let value: Option<String> = match field.r#type.as_str() {
            "checkbox" => dynprompt::prompt(
                Confirm::new(&msg).with_help_message("Type y[es] or n[o]"),
                optional,
            )?,
            "text-single-line" => dynprompt::prompt(Text::new(&msg), optional)?,
            "date" => dynprompt::prompt(DateSelect::new(&msg), optional)?,
            "preset-list" if options.is_some() => {
                dynprompt::prompt(Select::new(&msg, options.unwrap()), optional)?
            }
            "preset-checklist" if options.is_some() => {
                dynprompt::prompt(MultiSelect::new(&msg, options.unwrap()), optional)?
            }
            _ => None, // unsupported type
        };

        if let Some(value) = value {
            values.insert(field.name, value);
        }
    }

    Ok(values)
}
