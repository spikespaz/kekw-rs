//! This module covers endpoints for the [Authorization code grant flow][0].
//!
//! You must register an Application on the Twitch
//! [Developer Console][1], acquire the Client ID and Client Secret,
//!
//! [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
//! [1]: https://dev.twitch.tv/console

use std::fmt;

use aliri_braid::braid;
use kekw_macros::QueryParams;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

use crate::types::Scopes;

static AUTHORIZE_CODE_REQUEST_URL: &str = "https://id.twitch.tv/oauth2/authorize";
static AUTHORIZE_TOKEN_REQUEST_URL: &str = "https://id.twitch.tv/oauth2/token";

fn unwrap_option<T>(opt: &Option<T>) -> &T {
    opt.as_ref().unwrap()
}

fn percent_encode(source: impl ToString) -> String {
    use percent_encoding::NON_ALPHANUMERIC;
    percent_encoding::percent_encode(source.to_string().as_ref(), NON_ALPHANUMERIC).collect()
}

#[braid(secret, serde)]
pub struct ClientId;

#[braid(secret, serde)]
pub struct ClientSecret;

#[braid(secret, serde)]
pub struct AuthState;

#[braid(secret, serde)]
pub struct AuthCode;

#[braid(secret, serde)]
pub struct AccessToken;

#[braid(secret, serde)]
pub struct RefreshToken;

/// [Authorization code grant flow][0] during the [first][1] step.
///
/// [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
/// [1]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#get-the-user-to-authorize-your-app
///
/// This is built as query parameters for a [`Url`].
/// Turn this into a URL with [`Into::into`] for [`From::from`].
/// Tell the user to open this URL, then open an HTTP server that waits for
/// Twitch to send it a code, [`AuthCodeAllowed`], or a failure, [`AuthCodeDenied`].
#[derive(Serialize, Deserialize, TypedBuilder, QueryParams)]
pub struct AuthCodeQuery {
    pub client_id: ClientId,
    #[builder(setter(strip_bool))]
    #[query_param(skip_if = |x: &bool| !x)]
    pub force_verify: bool,
    pub redirect_uri: String,
    #[builder(default = "code", setter(skip))]
    response_type: &'static str,
    #[query_param(skip_if = Vec::is_empty, proxy = percent_encode)]
    pub scope: Scopes,
    #[builder(default, setter(strip_option))]
    #[query_param(skip_if = Option::is_none, proxy = unwrap_option)]
    pub state: Option<AuthState>,
}

impl From<AuthCodeQuery> for Url {
    fn from(query: AuthCodeQuery) -> Url {
        let mut url = Url::parse(AUTHORIZE_CODE_REQUEST_URL).unwrap();
        url.set_query(Some(&query.to_string()));
        url
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCodeAllowed {
    pub code: AuthCode,
    pub scope: Scopes,
    pub state: Option<AuthState>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCodeDenied {
    pub error: String,
    pub error_description: String,
    pub state: Option<AuthState>,
}

impl fmt::Display for AuthCodeDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.error_description, self.error)
    }
}

impl std::error::Error for AuthCodeDenied {}

/// [Authorization code grant flow][0] during the [second][2] step.
///
/// [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
/// [2]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#use-the-authorization-code-to-get-a-token
///
/// The documentation explicitly states that the query string is supposed to be
/// sent in the body of the `POST` request, but this always fails. Instead, send it in the URL.
#[derive(Debug, Serialize, Deserialize, TypedBuilder, QueryParams)]
pub struct AuthTokenRequestBody {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub code: AuthCode,
    #[builder(default = "authorization_code", setter(skip))]
    grant_type: &'static str,
    pub redirect_uri: String,
}

impl From<AuthTokenRequestBody> for Url {
    fn from(query: AuthTokenRequestBody) -> Url {
        let mut url = Url::parse(AUTHORIZE_TOKEN_REQUEST_URL).unwrap();
        url.set_query(Some(&query.to_string()));
        url
    }
}

/// Encodes [`AuthTokenReqBody`] with `Display`/`ToString`.
/// This is a "form encoded" query string, except it is not actually form encoded.
/// We bypass `serde_qs` for the encoding, and for now, only utilize it for decoding.
#[cfg(feature = "http-types")]
impl From<AuthTokenRequestBody> for http_types::Request {
    fn from(query: AuthTokenRequestBody) -> http_types::Request {
        let url = Url::from(query);
        http_types::Request::post(url)
    }
}

// #[cfg(feature = "isahc")]
// impl From<AuthTokenQuery> for isahc::Request<()> {
//     fn from(query: AuthTokenQuery) -> isahc::Request<()> {
//         let url = Url::from(query);
//         isahc::Request::post(url.as_str())
//             .body(())
//             .expect("failed to set request body")
//     }
// }

#[cfg(feature = "isahc")]
impl From<AuthTokenRequestBody> for isahc::Request<String> {
    fn from(body: AuthTokenRequestBody) -> isahc::Request<String> {
        isahc::Request::post(AUTHORIZE_TOKEN_REQUEST_URL)
            .body(serde_json::to_string(&body).unwrap())
            .expect("failed to set request body")
    }
}

/// Body of the final response from the Twitch API.
/// This completes the [second][2] step of the [Authorization code grant flow][0].
///
/// [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
/// [2]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#use-the-authorization-code-to-get-a-token
///
/// ```json
/// {
///   "access_token": "rfx2uswqe8l4g1mkagrvg5tv0ks3",
///   "expires_in": 14124,
///   "refresh_token": "5b93chm6hdve3mycz05zfzatkfdenfspp1h1ar2xxdalen01",
///   "scope": [
///     "channel:moderate",
///     "chat:edit",
///     "chat:read"
///   ],
///   "token_type": "bearer"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokenAllowed {
    pub access_token: AccessToken,
    pub expires_in: usize,
    pub refresh_token: Option<RefreshToken>,
    pub scope: Scopes,
    pub token_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Scope, Scopes};

    #[test]
    fn test_display_query() {
        let compare = "client_id=oogabooga&redirect_uri=https://localhost:8083&response_type=code&scope=analytics%3Aread%3Aextensions%20channel%3Abot";
        let data = AuthCodeQuery::builder()
            .client_id("oogabooga".into())
            .redirect_uri("https://localhost:8083".into())
            .scope(Scopes::from_iter([
                Scope::AnalyticsReadExtensions,
                Scope::ChannelBot,
            ]))
            .build();
        assert_eq!(compare, data.to_string());
    }
}
