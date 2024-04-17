use std::fmt;

use serde::{Deserialize, Serialize};

use crate::types::{AccessToken, AuthCode, CsrfState, RefreshToken, Scopes};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCodeAllowed {
    pub code: AuthCode,
    pub scope: Scopes,
    pub state: Option<CsrfState>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCodeDenied {
    pub error: String,
    pub error_description: String,
    pub state: Option<CsrfState>,
}

impl fmt::Display for AuthCodeDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.error_description, self.error)
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
