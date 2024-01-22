use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No default language is available for the selected country")]
    LidlNoDefaultLanguageForCountry,
    #[error("Product has been skipped")]
    SkippedProduct,
    #[error("Expected barcode associated with its product to have an amount, but it didn't")]
    BarcodeAmountNotFound,
}
