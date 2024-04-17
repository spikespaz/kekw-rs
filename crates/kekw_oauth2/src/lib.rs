/// Start here.
///
/// Construct these types using the provided builder, and then either call
/// a function or use `Into` to create a `Request` for the crate of your choice.
pub mod requests;

/// The API can deserialize into these types.
pub mod response;

/// Spawn a server to listen for the authorization code response from the Twitch API.
/// This has no dependencies other than `async_net` and `futures_lite`, and is agnostic
/// to your runtime preferences.
pub mod server;

/// Atomics that appear in [`requests`] or [`response`].
pub mod types;
