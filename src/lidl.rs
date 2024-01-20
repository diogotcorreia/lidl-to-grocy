use anyhow::Result;
use inquire::{Select, Text};
use lidl::{get_countries, LidlApi, OAuthFlow};

use crate::{error::Error, LidlConfig};

pub(super) fn fetch_receipt_from_lidl(config: &mut LidlConfig) -> Result<()> {
    let lidl_api = match &config.refresh_token {
        None => init_token_lidl()?,
        Some(refresh_token) => OAuthFlow::get_token_from_refresh_token(refresh_token.clone())?,
    };
    // Save refresh token to config, for future runs
    config.refresh_token = Some(lidl_api.get_refresh_token());

    // todo!()
    Ok(())
}

fn init_token_lidl() -> Result<LidlApi> {
    let countries = get_countries()?;

    let selected_country = Select::new("Select country for Lidl:", countries).prompt()?;
    let selected_language = selected_country
        .get_default_language()
        .ok_or(Error::LidlNoDefaultLanguageForCountry)?;

    let oauth_flow = OAuthFlow::init(&selected_country, &selected_language)?;
    println!(
        "Open the following URL in your browser to login: {}",
        oauth_flow.get_url()
    );
    println!();
    println!("Instructions:");
    println!("1. Before logging in, open devtools and go to the network tab.");
    println!("2. After logging in, there will be a blocked request (due to unknown protocol).");
    println!("3. Open that request, and copy the value of the Location response header.");

    let callback_url = Text::new("Please enter the callback URL you got:")
        .with_placeholder("com.lidlplus.app://callback?...")
        .prompt()?;

    oauth_flow.validate(&callback_url)
}
