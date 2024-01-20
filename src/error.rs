use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No default language is available for the selected country")]
    LidlNoDefaultLanguageForCountry,
}
