//! This example is for the [Authorization code grant flow][0].
//! Please see the source code for this file.
//!
//! To run this example, you must register an Application on the Twitch
//! [Developer Console][1], acquire the Client ID and Client Secret,
//! and set the environment variables `TWITCH_CLIENT_ID` and `TWITCH_CLIENT_SECRET`.
//!
//! [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
//! [1]: https://dev.twitch.tv/console

use kekw_oauth2::endpoints::{AuthCodeQuery, AuthTokenReqBody, ClientId, ClientSecret};
use kekw_oauth2::server::*;
use kekw_oauth2::types::Scopes;
use once_cell::sync::Lazy;
use url::Url;

const AUTH_MAX_TRIES: usize = 5;

static AUTH_LISTEN_PORT: u16 = 8833;
static AUTH_LISTEN_IPS: &[&str] = &["127.0.0.1", "0.0.0.0"];

static REDIRECT_URI: &str = "http://localhost:8833";

static TWITCH_CLIENT_ID: Lazy<ClientId> = Lazy::new(|| {
    std::env::var("TWITCH_CLIENT_ID")
        .expect("environment variable `TWITCH_CLIENT_ID` is not set!")
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

fn main() {
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

    // Build the initial query according to the table under [Get the user to authorize your app].
    // Some of these are set by default (or are immutable).
    let query = AuthCodeQuery::builder()
        .client_id(TWITCH_CLIENT_ID.clone())
        .force_verify() // Disable this if you implement persistence.
        .redirect_uri(REDIRECT_URI.to_owned())
        .scope(TWITCH_AUTH_SCOPE.clone())
        // .state(state) // randomly generate a state string for CSRF protection.
        .build();
    let url = &Url::from(query).to_string();

    // Display the URL to that the user may choose to click it.
    eprintln!("Open this URL in your browser: {url}");

    smol::block_on(async {
        let addrs = make_socket_addrs(AUTH_LISTEN_IPS, AUTH_LISTEN_PORT);

        // Open the user's browser to the Twitch authentication dialog.
        open::that(url).expect("failed to open default program to handle URL");

        // Open a server and wait for the user to click the button.
        let res = await_auth_code(&addrs[..], AUTH_MAX_TRIES).await;

        dbg!(res);
    });
}
