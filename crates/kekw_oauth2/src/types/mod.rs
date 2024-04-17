pub mod scopes;

#[rustfmt::skip]
#[doc(inline)]
pub use scopes::*;

use aliri_braid::braid;

#[braid(secret, serde)]
pub struct ClientId;

#[braid(secret, serde)]
pub struct ClientSecret;

#[braid(secret, serde)]
pub struct CsrfState;

impl CsrfState {
    pub fn new_random() -> Self {
        use rand::distributions::Distribution;
        Self(
            rand::distributions::Alphanumeric
                .sample_iter(rand::thread_rng())
                .take(16)
                .map(|c| c as char)
                .collect(),
        )
    }
}

#[braid(secret, serde)]
pub struct AuthCode;

#[braid(secret, serde)]
pub struct AccessToken;

#[braid(secret, serde)]
pub struct RefreshToken;
