use std::fmt;
use std::str::FromStr;

use http_types::{Method, Request, Url};
use kekw_types::url::QueryPairs;
use percent_encoding::{percent_encode, CONTROLS};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::types::Scopes;

static AUTHORIZE_CODE_REQUEST_URL: &str = "https://id.twitch.tv/oauth2/authorize";

/// [Authorization code grant flow][1]
///
/// [1]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
#[derive(Serialize, TypedBuilder)]
#[builder(build_method(into = QueryPairs<'static>))]
pub struct AuthCodeQuery {
    pub client_id: String,
    #[builder(setter(strip_bool))]
    pub force_verify: bool,
    pub redirect_uri: Url,
    pub scope: Scopes,
    #[builder(setter(strip_option))]
    pub state: Option<String>,
}

impl From<AuthCodeQuery> for Url {
    fn from(other: AuthCodeQuery) -> Url {
        let mut url = Url::from_str(AUTHORIZE_CODE_REQUEST_URL).unwrap();
        url.set_query(Some(
            &serde_qs::to_string(&other).expect("failed to serialize query"),
        ));
        url
    }
}

impl From<AuthCodeQuery> for QueryPairs<'static> {
    fn from(other: AuthCodeQuery) -> Self {
        Self::from_iter([
            ("client_id", Some(other.client_id)),
            ("force_verify", Some(other.force_verify.to_string())),
            ("redirect_uri", Some(other.redirect_uri.to_string())),
            ("response_type", Some("code".into())),
            (
                "scope",
                Some(percent_encode(other.scope.to_string().as_bytes(), CONTROLS).to_string()),
            ),
            ("state", other.state),
        ])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthCodePassed {
    pub code: String,
    pub scope: usize,
    pub state: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthCodeDenied {
    pub error: String,
    pub error_description: Scopes,
    pub state: String,
}

impl fmt::Display for AuthCodeDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.error_description, self.error)
    }
}

impl std::error::Error for AuthCodeDenied {}

// #[cfg(test)]
// mod test {
//     fn
// }
