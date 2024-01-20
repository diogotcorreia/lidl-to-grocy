use std::borrow::Cow;
use std::collections::HashMap;

use anyhow::Result;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, ExtraTokenFields, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, StandardTokenResponse, TokenResponse,
    TokenType, TokenUrl,
};
use reqwest::header::AUTHORIZATION;
use reqwest::Url;
use structs::{Country, Language};

use crate::error::Error;

mod error;
mod structs;

const APPGATEWAY_ENDPOINT: &str = "https://appgateway.lidlplus.com";
const ACCOUNTS_ENDPOINT: &str = "https://accounts.lidl.com";

const OAUTH_CLIENT_ID: &str = "LidlPlusNativeClient";
const OAUTH_REDIRECT_URL: &str = "com.lidlplus.app://callback";
const OAUTH_SCOPES: &[&str] =
    ["openid", "profile", "offline_access", "lpprofile", "lpapis"].as_slice();
const OAUTH_AUTHORIZE_PATH: &str = "connect/authorize";
const OAUTH_TOKEN_PATH: &str = "connect/token";
const OAUTH_AUTHORIZATION_HEADER: &str = "Basic TGlkbFBsdXNOYXRpdmVDbGllbnQ6c2VjcmV0"; // LidlPlusNativeClient:secret in base64

pub struct LidlApi {
    #[allow(dead_code)]
    access_token: String,
    refresh_token: String,
}

impl LidlApi {
    pub fn get_refresh_token(&self) -> String {
        self.refresh_token.clone()
    }
}

impl<EF, TT> TryFrom<StandardTokenResponse<EF, TT>> for LidlApi
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    type Error = crate::error::Error;

    fn try_from(response: StandardTokenResponse<EF, TT>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            access_token: response.access_token().secret().clone(),
            refresh_token: response
                .refresh_token()
                .ok_or(Error::OAuthMissingRefreshToken)?
                .secret()
                .clone(),
        })
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

        Ok(token_response.try_into()?)
    }

    pub fn get_token_from_refresh_token(refresh_token: String) -> Result<LidlApi> {
        let client = Self::init_client()?;

        let token_response = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request(Self::http_client)?;

        Ok(token_response.try_into()?)
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
