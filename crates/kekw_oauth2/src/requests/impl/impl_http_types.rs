use http_types::Request;

/// Encodes [`AuthTokenReqBody`] with `Display`/`ToString`.
/// This is a "form encoded" query string, except it is not actually form encoded.
/// We bypass `serde_qs` for the encoding, and for now, only utilize it for decoding.
impl From<AuthTokenRequestQuery> for Request {
    fn from(query: AuthTokenRequestQuery) -> Request {
        Request::post(Url::from(query))
    }
}
