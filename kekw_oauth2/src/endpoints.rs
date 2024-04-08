use std::fmt;

use aliri_braid::braid;
use kekw_macros::QueryParams;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

use crate::types::Scopes;

static AUTHORIZE_CODE_REQUEST_URL: &str = "https://id.twitch.tv/oauth2/authorize";

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
pub struct AuthFlowState;

/// [Authorization code grant flow][1]
///
/// [1]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
#[derive(Serialize, Deserialize, TypedBuilder, QueryParams)]
pub struct AuthCodeQuery {
    pub client_id: ClientId,
    #[builder(setter(strip_bool))]
    #[query_param(skip_if = |x: &bool| !x)]
    pub force_verify: bool,
    pub redirect_uri: String,
    #[query_param(skip_if = Vec::is_empty, proxy = percent_encode)]
    pub scope: Scopes,
    #[builder(default, setter(strip_option))]
    #[query_param(skip_if = Option::is_none, proxy = unwrap_option)]
    pub state: Option<AuthFlowState>,
}

impl From<AuthCodeQuery> for Url {
    fn from(query: AuthCodeQuery) -> Url {
        let mut url = Url::parse(AUTHORIZE_CODE_REQUEST_URL).unwrap();
        url.set_query(Some(&query.to_string()));
        url
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthCodePassed {
    pub code: String,
    pub scope: usize,
    pub state: AuthFlowState,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthCodeDenied {
    pub error: String,
    pub error_description: Scopes,
    pub state: AuthFlowState,
}

impl fmt::Display for AuthCodeDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.error_description, self.error)
    }
}

impl std::error::Error for AuthCodeDenied {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Scope, Scopes};

    #[test]
    fn test_display_query() {
        let compare = "?client_id=oogabooga&redirect_uri=https://localhost:8083&scope=analytics%3Aread%3Aextensions%20channel%3Abot";
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
