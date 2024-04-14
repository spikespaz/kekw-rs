//! This example is for the [Authorization code grant flow][0].
//! Please see the source code for this file.
//!
//! To run this example, you must register an Application on the Twitch
//! [Developer Console][1], acquire the Client ID and Client Secret,
//! and set the environment variables `TWITCH_CLIENT_ID` and `TWITCH_CLIENT_SECRET`.
//!
//! [0]: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
//! [1]: https://dev.twitch.tv/console

use futures_lite::AsyncReadExt;
use isahc::HttpClient;
use kekw_oauth2::endpoints::{
    AuthCodeQuery, AuthState, AuthTokenAllowed, AuthTokenRequestQuery, ClientId, ClientSecret,
};
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

    smol::block_on(async {
        let addrs = make_socket_addrs(AUTH_LISTEN_IPS, AUTH_LISTEN_PORT);

        let state = AuthState::new("c3ab8aa609ea11e793ae92361f002671".to_owned());

        // Build the initial query according to the table under [Get the user to authorize your app].
        // Some of these are set by default (or are immutable).
        let query = AuthCodeQuery::builder()
            .client_id(TWITCH_CLIENT_ID.clone())
            .force_verify() // Disable this if you implement persistence.
            .redirect_uri(REDIRECT_URI.to_owned())
            .scope(TWITCH_AUTH_SCOPE.clone())
            // .state(state.clone()) // randomly generate a state string for CSRF protection.
            .build();
        let url = &Url::from(query).to_string();

        // Display the URL to that the user may choose to click it.
        eprintln!("Open this URL in your browser: {url}\n");

        // Open the user's browser to the Twitch authentication dialog.
        open::that(url).expect("failed to open default program to handle URL");

        // Open a server and wait for the user to click the button.
        let allow = await_auth_code(&addrs[..], AUTH_MAX_TRIES)
            .await
            .expect("authorization code handshake failed");

        match allow.state {
            Some(token) if token != state => {
                panic!("csrf attack oh no");
            }
            // None => {
            //     panic!("csrf attack oh no");
            // }
            _ => (),
        };

        let client = HttpClient::new().expect("failed to create http client");

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
                body.read_to_end(&mut buf)
                    .await
                    .expect("failed to read a response to the end");
                panic!("{:?}", serde_json::from_slice::<serde_json::Value>(&buf));
            }
            Err(_) => todo!(),
        };

        dbg!(res);
    });
}

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
