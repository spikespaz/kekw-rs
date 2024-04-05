use std::str::FromStr;

use http_types::{Method, Request, Url};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::types::scopes::Scopes;

static AUTHORIZE_REQUEST_URL: &str = "https://id.twitch.tv/oauth2/authorize";

/// Authorization code grant flow
///
/// <https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow>
#[derive(Serialize, TypedBuilder)]
#[builder(build_method(into = Url))]
pub struct AuthorizeTokenParams {
    pub client_id: String,
    #[builder(setter(strip_bool))]
    pub force_verify: bool,
    pub redirect_uri: Url,
    #[builder(default = "code", setter(skip))]
    response_type: &'static str,
    pub scope: Scopes,
    #[builder(setter(strip_option))]
    pub state: Option<String>,
}

impl From<AuthorizeTokenParams> for Url {
    fn from(other: AuthorizeTokenParams) -> Url {
        let mut url = Url::from_str(AUTHORIZE_REQUEST_URL).unwrap();
        url.set_query(Some(
            &serde_qs::to_string(&other).expect("failed to serialize query"),
        ));
        url
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthorizeTokenResponse {
    pub access_token: String,
    pub expires_in: usize,
    pub token_type: String,
}
