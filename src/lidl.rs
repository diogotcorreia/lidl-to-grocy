use anyhow::Result;
use inquire::{Confirm, Select, Text};
use lidl::{
    get_countries,
    structs::{Country, ReceiptDetailed},
    LidlApi, OAuthFlow,
};

use crate::{error::Error, LidlConfig, LidlLocale};

pub(super) fn fetch_receipt_from_lidl(config: &mut LidlConfig) -> Result<ReceiptDetailed<f64>> {
    let lidl_api = match &config.refresh_token {
        None => init_token_lidl(config)?,
        Some(refresh_token) => {
            let locale = match &config.locale {
                Some(locale) => locale,
                None => {
                    let country = prompt_lidl_country()?;
                    let language = country
                        .get_default_language()
                        .ok_or(Error::LidlNoDefaultLanguageForCountry)?;

                    config.locale = Some(LidlLocale {
                        country: country.id,
                        language: language.id,
                    });
                    config.locale.as_ref().unwrap()
                }
            };
            OAuthFlow::get_token_from_refresh_token(
                locale.country.clone(),
                locale.language.clone(),
                refresh_token.clone(),
            )?
        }
    };
    // Save refresh token to config, for future runs
    config.refresh_token = Some(lidl_api.get_refresh_token());

    let receipts = lidl_api.get_available_receipts()?.receipts;

    let receipt = Select::new("Select receipt to import:", receipts).prompt()?;
    lidl_api.get_specific_receipt(&receipt)
}

fn init_token_lidl(config: &mut LidlConfig) -> Result<LidlApi> {
    let selected_country = prompt_lidl_country()?;
    let selected_language = selected_country
        .get_default_language()
        .ok_or(Error::LidlNoDefaultLanguageForCountry)?;

    println!("Please read these simple instructions:");
    println!("1. You will be a given a link to click on");
    println!("2. Before logging in, open DevTools and go to the Network tab");
    println!("3. Log in using your Lidl account credentials for the country you selected");
    println!("4. After logging in, DevTools will show a blocked request (due to unknown protocol)");
    println!("5. Open that request, and copy the value of the Location response header");
    println!();

    loop {
        let smart = Confirm::new("Do you understand the instructions above?")
            .with_help_message("Type y[es] if you are ready to follow them, or [n]o otherwise")
            .prompt()?;

        if smart {
            break;
        } else {
            println!("Okay, re-read them and let's try again");
            println!();
        }
    }

    let oauth_flow = OAuthFlow::init(&selected_country, &selected_language)?;
    println!(
        "Open the following URL in your browser to login: {}",
        oauth_flow.get_url()
    );
    println!();

    let callback_url = Text::new("Please enter the callback URL you got:")
        .with_placeholder("com.lidlplus.app://callback?...")
        .prompt()?;

    config.locale = Some(LidlLocale {
        country: selected_country.id,
        language: selected_language.id,
    });

    oauth_flow.validate(&callback_url)
}

fn prompt_lidl_country() -> Result<Country> {
    let countries = get_countries()?;

    Ok(Select::new("Select country for Lidl:", countries).prompt()?)
}
