use std::borrow::Cow;
use std::collections::HashMap;

use anyhow::Result;
use ir::{ReceiptSummary, StoreApi};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RefreshToken, Scope, TokenResponse, TokenType, TokenUrl,
};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, ACCEPT_LANGUAGE, AUTHORIZATION};
use reqwest::Url;
use structs::{Country, Language, ReceiptDetailed, ReceiptsPage};

use crate::error::Error;

pub mod error;
pub mod structs;

const APPGATEWAY_ENDPOINT: &str = "https://appgateway.lidlplus.com";
const ACCOUNTS_ENDPOINT: &str = "https://accounts.lidl.com";
const TICKETS_ENDPOINT: &str = "https://tickets.lidlplus.com";

const OAUTH_CLIENT_ID: &str = "LidlPlusNativeClient";
const OAUTH_REDIRECT_URL: &str = "com.lidlplus.app://callback";
const OAUTH_SCOPES: &[&str] =
    ["openid", "profile", "offline_access", "lpprofile", "lpapis"].as_slice();
const OAUTH_AUTHORIZE_PATH: &str = "connect/authorize";
const OAUTH_TOKEN_PATH: &str = "connect/token";
const OAUTH_AUTHORIZATION_HEADER: &str = "Basic TGlkbFBsdXNOYXRpdmVDbGllbnQ6c2VjcmV0"; // LidlPlusNativeClient:secret in base64

pub struct LidlApi {
    refresh_token: String,
    country_code: String,
    client: Client,
}

impl LidlApi {
    fn from_token_response<TT>(
        country_code: String,
        language_code: String,
        token_response: &impl TokenResponse<TT>,
    ) -> Result<Self>
    where
        TT: TokenType,
    {
        let access_token = token_response.access_token().secret();
        let mut headers = HeaderMap::new();
        headers.append(AUTHORIZATION, format!("Bearer {}", access_token).parse()?);
        headers.append(ACCEPT_LANGUAGE, language_code.parse()?);
        Ok(Self {
            refresh_token: token_response
                .refresh_token()
                .ok_or(Error::OAuthMissingRefreshToken)?
                .secret()
                .clone(),
            country_code,
            client: reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    pub fn get_refresh_token(&self) -> String {
        self.refresh_token.clone()
    }

    pub fn get_country_code(&self) -> String {
        self.country_code.clone()
    }
}

impl StoreApi for LidlApi {
    fn get_available_receipts(&self) -> Result<Vec<ReceiptSummary>> {
        let receipts_page: ReceiptsPage = self
            .client
            .get(format!(
                "{}/api/v2/{}/tickets",
                TICKETS_ENDPOINT, self.country_code,
            ))
            .query(&[
                ("pageNumber", "1"),
                ("onlyFavorite", "false"),
                ("itemId", ""),
            ])
            .send()?
            .json()?;

        Ok(receipts_page.receipts.into_iter().map(Into::into).collect())
    }

    fn get_specific_receipt(&self, receipt: &ReceiptSummary) -> Result<ir::ReceiptDetailed> {
        let receipt: ReceiptDetailed<f64> = self
            .client
            .get(format!(
                "{}/api/v2/{}/tickets/{}",
                TICKETS_ENDPOINT, self.country_code, receipt.id
            ))
            .send()?
            .json::<ReceiptDetailed<String>>()?
            .try_into()?;

        Ok(receipt.into())
    }
}

pub fn get_countries() -> Result<Vec<Country>> {
    Ok(reqwest::blocking::get(format!(
        "{}/{}",
        APPGATEWAY_ENDPOINT, "configurationapp/v3/countries"
    ))?
    .json()?)
}

pub struct OAuthFlow {
    client: BasicClient,
    auth_url: Url,
    csrf_token: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
    country_code: String,
    language_code: String,
}

impl OAuthFlow {
    fn init_client() -> Result<BasicClient> {
        Ok(BasicClient::new(
            ClientId::new(OAUTH_CLIENT_ID.to_string()),
            None,
            AuthUrl::new(format!("{}/{}", ACCOUNTS_ENDPOINT, OAUTH_AUTHORIZE_PATH))?,
            Some(TokenUrl::new(format!(
                "{}/{}",
                ACCOUNTS_ENDPOINT, OAUTH_TOKEN_PATH
            ))?),
        )
        .set_redirect_uri(RedirectUrl::new(OAUTH_REDIRECT_URL.to_string())?))
    }
    pub fn init(country: &Country, language: &Language) -> Result<OAuthFlow> {
        let client = Self::init_client()?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(
                OAUTH_SCOPES
                    .iter()
                    .map(|scope| Scope::new(scope.to_string())),
            )
            .add_extra_param("Country", country.id.clone())
            .add_extra_param("language", format!("{}-{}", language.id, country.id))
            .add_extra_param("force", "false")
            .add_extra_param("track", "false")
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok(OAuthFlow {
            client,
            auth_url,
            csrf_token,
            pkce_verifier,
            country_code: country.id.clone(),
            language_code: language.id.clone(),
        })
    }

    pub fn get_url(&self) -> String {
        self.auth_url.to_string()
    }

    pub fn validate(self, callback_url: &str) -> Result<LidlApi> {
        let parsed_url = Url::parse(callback_url)?;

        let query: HashMap<Cow<str>, Cow<str>> = parsed_url.query_pairs().collect();

        let code = query
            .get("code")
            .ok_or(Error::OAuthCallbackMissingParameter("code"))?;
        let state = query
            .get("state")
            .ok_or(Error::OAuthCallbackMissingParameter("state"))?;

        if self.csrf_token.secret() != state {
            return Err(Error::OAuthCsrfMismatch.into());
        }

        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(self.pkce_verifier)
            .request(Self::http_client)?;

        LidlApi::from_token_response(self.country_code, self.language_code, &token_response)
    }

    pub fn get_token_from_refresh_token(
        country_code: String,
        language_code: String,
        refresh_token: String,
    ) -> Result<LidlApi> {
        let client = Self::init_client()?;

        let token_response = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request(Self::http_client)?;

        LidlApi::from_token_response(country_code, language_code, &token_response)
    }

    fn http_client(
        mut request: oauth2::HttpRequest,
    ) -> Result<oauth2::HttpResponse, oauth2::reqwest::Error<reqwest::Error>> {
        request
            .headers
            .insert(AUTHORIZATION, OAUTH_AUTHORIZATION_HEADER.parse().unwrap());
        oauth2::reqwest::http_client(request)
    }
}
