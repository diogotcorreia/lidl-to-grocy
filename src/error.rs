use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No default language is available for the selected country")]
    LidlNoDefaultLanguageForCountry,
    #[error("Product has been skipped")]
    SkippedProduct,
    #[error("Expected barcode associated with its product to have an amount, but it didn't")]
    BarcodeAmountNotFound,
    #[error("Expected barcode associated with its product to have a quantity unit, but it didn't")]
    BarcodeQuantityUnitNotFound,
    #[error("Barcode's quantity unit (#{0}) is not its product's stock nor purchase default")]
    BarcodeQuantityUnitUnsupported(u32),
    #[error("Product has tare weight handling enabled, but that is not supported by this program")]
    ProductHasTareWeightHandling,
}
