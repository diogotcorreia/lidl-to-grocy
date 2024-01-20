use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("OAuth callback does not include required parameter {0}")]
    OAuthCallbackMissingParameter(&'static str),
    #[error("The CSRF token in the OAuth flow does not match the expected value")]
    OAuthCsrfMismatch,
    #[error("OAuth token response does not include refresh token")]
    OAuthMissingRefreshToken,
}
