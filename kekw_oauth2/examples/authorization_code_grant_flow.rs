use std::net::SocketAddr;

use async_net::IpAddr;
use kekw_oauth2::endpoints::{AuthCodeQuery, ClientId};
use kekw_oauth2::server::*;
use kekw_oauth2::types::{Scope, Scopes};
use once_cell::sync::Lazy;
use url::Url;

const AUTH_MAX_TRIES: usize = 3;
static AUTH_LISTEN_PORT: u16 = 8833;
static AUTH_LISTEN_IPS: &[&str] = &["127.0.0.1", "0.0.0.0"];
static REDIRECT_URI: &str = "http://localhost:8833";
static TWITCH_CLIENT_ID: Lazy<ClientId> = Lazy::new(|| {
    std::env::var("TWITCH_CLIENT_ID")
        .expect("environment variable `TWITCH_CLIENT_ID` is not set!")
        .into()
});
static TWITCH_AUTH_SCOPE: Lazy<Scopes> =
    Lazy::new(|| Scopes::from_iter([Scope::ChatRead, Scope::ChannelReadRedemptions]));

fn main() {
    eprintln!(
        r#"Maximum authentication attempts: {AUTH_MAX_TRIES}
Listening on port: {AUTH_LISTEN_PORT}
Listening on IP addresses: {}
Redirect URI: {REDIRECT_URI}
Twitch client ID: {:#?}
Authentication scopes requested: {}
"#,
        AUTH_LISTEN_IPS.join(", "),
        *TWITCH_CLIENT_ID,
        *TWITCH_AUTH_SCOPE
    );

    let query = AuthCodeQuery::builder()
        .client_id(TWITCH_CLIENT_ID.clone())
        .force_verify()
        .redirect_uri(REDIRECT_URI.to_owned())
        .scope(TWITCH_AUTH_SCOPE.clone())
        .build();
    let url = &Url::from(query).to_string();

    eprintln!("Open this URL in your browser: {url}");

    smol::block_on(async {
        let addrs = AUTH_LISTEN_IPS
            .iter()
            .map(|ip| SocketAddr::new(ip.parse::<IpAddr>().unwrap(), AUTH_LISTEN_PORT))
            .collect::<Vec<_>>();

        open::that(url).expect("failed to open default program for https url");

        let res = await_auth_code(&addrs[..], AUTH_MAX_TRIES).await;

        dbg!(res);
    });
}
