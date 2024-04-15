//! This example is for the [Authorization code grant flow][0].
//! Please see the source code for this file.
//!
//! To run this example, you must register an Application on the Twitch
//! [Developer Console][10], acquire the Client ID and Client Secret,
//! and set the environment variables `TWITCH_CLIENT_ID` and `TWITCH_CLIENT_SECRET`.
//!
//! [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
//! [1]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#get-the-user-to-authorize-your-app
//! [2]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#use-the-authorization-code-to-get-a-token
//! [10]: https://dev.twitch.tv/console

use eyre::{eyre, Context};
use futures_lite::AsyncReadExt;
use isahc::HttpClient;
use kekw_oauth2::requests::{AuthCodeQuery, AuthTokenRequestQuery};
use kekw_oauth2::response::AuthTokenAllowed;
use kekw_oauth2::server::*;
use kekw_oauth2::types::{ClientId, ClientSecret, CsrfState, Scopes};
use once_cell::sync::Lazy;
use url::Url;

/// The server provided by this crate is very simple and the only parameter is
/// the number of failed attempts before the server closes.
const AUTH_MAX_TRIES: usize = 5;

static AUTH_LISTEN_PORT: u16 = 8833;
static AUTH_LISTEN_IPS: &[&str] = &["127.0.0.1", "0.0.0.0"];

// This must match your settings on the Twitch Developer Console.
static REDIRECT_URI: &str = "http://localhost:8833";

static TWITCH_CLIENT_ID: Lazy<ClientId> = Lazy::new(|| {
    std::env::var("TWITCH_CLIENT_ID")
        .expect("environment variable `TWITCH_CLIENT_ID` is not set!")
        .into()
});
static TWITCH_CLIENT_SECRET: Lazy<ClientSecret> = Lazy::new(|| {
    std::env::var("TWITCH_CLIENT_SECRET")
        .expect("environment variable `TWITCH_CLIENT_SECRET` is not set!")
        .into()
});
/// Change the permissions required by the bot.
static TWITCH_AUTH_SCOPE: Lazy<Scopes> = Lazy::new(|| {
    use kekw_oauth2::types::Scope::*;
    Scopes::from_iter([
        ChatRead,
        ChannelReadPolls,
        ChannelReadRedemptions,
        // ChannelModerate,
        // ChannelManagePolls,
    ])
});

fn main() -> eyre::Result<()> {
    eprintln!(
        r#"Maximum authentication attempts: {AUTH_MAX_TRIES}
Listening on port: {AUTH_LISTEN_PORT}
Listening on IP addresses: {}
Redirect URI: {REDIRECT_URI}
Twitch client ID: {:#5?}
Authentication scopes requested: {}
"#,
        AUTH_LISTEN_IPS.join(", "),
        *TWITCH_CLIENT_ID,
        *TWITCH_AUTH_SCOPE
    );

    smol::block_on(async {
        let addrs = make_socket_addrs(AUTH_LISTEN_IPS, AUTH_LISTEN_PORT);

        let state = CsrfState::new_random();

        // Step 1.
        // Build the initial query according to the table under [Get the user to authorize your app].
        // Some of these are set by default (or are immutable).
        let query = AuthCodeQuery::builder()
            .client_id(TWITCH_CLIENT_ID.clone())
            .force_verify() // Disable this if you implement persistence.
            .redirect_uri(REDIRECT_URI.to_owned())
            .scope(TWITCH_AUTH_SCOPE.clone())
            .state(state.clone()) // randomly generate a state string for CSRF protection.
            .build();
        let url = Url::from(&query);

        // Display the URL to that the user may choose to click it.
        eprintln!("Open this URL in your browser: {url}\n");

        // Open the user's browser to the Twitch authentication dialog.
        open::that(url.as_str()).wrap_err("failed to open default program to handle URL")?;

        // Spawn a server and wait for the user to accept your authentication scope using `await_auth_code`.
        let allow = await_auth_code(&addrs[..], Some(&state), AUTH_MAX_TRIES)
            .await
            .expect("authorization code handshake failed");

        // Step 2.
        // Open an HTTP client of your choice (see the crate features in `Cargo.toml`).
        let client = HttpClient::new().wrap_err("failed to create an HTTP client")?;

        // Use the authorization code to get a token for your session.
        let req_body = AuthTokenRequestQuery::builder()
            .client_id(TWITCH_CLIENT_ID.clone())
            .client_secret(TWITCH_CLIENT_SECRET.clone())
            .code(allow.code)
            .redirect_uri(REDIRECT_URI.to_owned())
            .build();
        let req = isahc::Request::from(req_body);

        dbg!(&req);

        let res = match client.send_async(req).await {
            Ok(res) if res.status().is_success() => {
                deserialize_response::<AuthTokenAllowed>(res).await
            }
            Ok(res) => {
                let (_parts, mut body) = res.into_parts();
                let mut buf = Vec::new();
                body.read_to_end(&mut buf).await?;
                return Err(eyre!(serde_json::from_slice::<serde_json::Value>(&buf)?));
            }
            Err(_) => todo!(),
        };

        // Print out the response (including the token).
        Ok(println!("{:?}", res?.into_body()))
    })
}

/// Reassemble the [`isahc::Response<isahc::AsyncBody>`] as [`isahc::Response<T>`]
/// where `T` is a type to `Deserialize` into.
async fn deserialize_response<T>(
    res: isahc::Response<isahc::AsyncBody>,
) -> serde_json::Result<isahc::Response<T>>
where
    for<'de> T: serde::de::Deserialize<'de>,
{
    let (parts, mut body) = res.into_parts();
    let mut buf = Vec::new();
    body.read_to_end(&mut buf)
        .await
        .expect("failed to read a response to the end");
    let body = serde_json::from_slice(&buf)?;
    Ok(isahc::Response::from_parts(parts, body))
}
