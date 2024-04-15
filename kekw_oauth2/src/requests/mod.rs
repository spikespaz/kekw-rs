//! This module covers endpoints for the [Authorization code grant flow][0].
//!
//! You must register an Application on the Twitch
//! [Developer Console][1], acquire the Client ID and Client Secret,
//!
//! [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
//! [1]: https://dev.twitch.tv/console

#[cfg(feature = "isahc")]
#[path = "./impl/impl_isahc.rs"]
mod r#impl;

#[cfg(feature = "http_types")]
#[path = "./impl/impl_http_types.rs"]
mod r#impl;

use kekw_macros::QueryParams;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

use crate::types::{AuthCode, ClientId, ClientSecret, CsrfState, Scopes};

static AUTHORIZE_CODE_REQUEST_URL: &str = "https://id.twitch.tv/oauth2/authorize";
static AUTHORIZE_TOKEN_REQUEST_URL: &str = "https://id.twitch.tv/oauth2/token";

fn unwrap_option<T>(opt: &Option<T>) -> &T {
    opt.as_ref().unwrap()
}

fn percent_encode(source: impl ToString) -> String {
    use percent_encoding::NON_ALPHANUMERIC;
    percent_encoding::percent_encode(source.to_string().as_ref(), NON_ALPHANUMERIC).collect()
}

/// [Authorization code grant flow][0] during the [second][2] step.
///
/// [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
/// [2]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#use-the-authorization-code-to-get-a-token
///
/// The documentation explicitly states that the query string is supposed to be
/// sent in the body of the `POST` request, but this always fails. Instead, send it in the URL.
#[derive(Debug, Serialize, Deserialize, TypedBuilder, QueryParams)]
pub struct AuthTokenRequestQuery {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub code: AuthCode,
    #[builder(default = "authorization_code", setter(skip))]
    grant_type: &'static str,
    pub redirect_uri: String,
}

impl From<AuthTokenRequestQuery> for Url {
    fn from(query: AuthTokenRequestQuery) -> Url {
        let mut url = Url::parse(AUTHORIZE_TOKEN_REQUEST_URL).unwrap();
        url.set_query(Some(&query.to_string()));
        url
    }
}

/// [Authorization code grant flow][0] during the [first][1] step.
///
/// [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
/// [1]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#get-the-user-to-authorize-your-app
///
/// This is built as query parameters for a [`Url`].
/// Turn this into a URL with [`Into::into`] for [`From::from`].
/// Tell the user to open this URL, then open an HTTP server that waits for
/// Twitch to send it a code, [`AuthCodeAllowed`], or a failure, [`AuthCodeDenied`].
///
/// [`AuthCodeAllowed`]: crate::response::AuthCodeAllowed
/// [`AuthCodeDenied`]: crate::response::AuthCodeDenied
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
    pub state: Option<CsrfState>,
}

impl From<&AuthCodeQuery> for Url {
    fn from(query: &AuthCodeQuery) -> Url {
        let mut url = Url::parse(AUTHORIZE_CODE_REQUEST_URL).unwrap();
        url.set_query(Some(&query.to_string()));
        url
    }
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
