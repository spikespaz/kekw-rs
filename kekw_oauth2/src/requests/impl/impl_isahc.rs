use isahc::Request;
use url::Url;

use crate::requests::AuthTokenRequestQuery;

impl From<AuthTokenRequestQuery> for Request<()> {
    fn from(query: AuthTokenRequestQuery) -> Request<()> {
        let url = Url::from(query);
        Request::post(url.as_str())
            .body(())
            .expect("failed to set request body")
    }
}

// impl From<AuthTokenRequestBody> for AsyncBody {
//     fn from(value: AuthTokenRequestBody) -> Self {
//         AsyncBody::from(dbg!(
//             serde_json::to_string(&value).expect("failed to serialize AuthTokenRequestBody")
//         ))
//     }
// }

// impl From<AuthTokenRequestBody> for Request<AuthTokenRequestBody> {
//     fn from(body: AuthTokenRequestBody) -> Request<AuthTokenRequestBody> {
//         Request::post(AUTHORIZE_TOKEN_REQUEST_URL)
//             .body(body)
//             .expect("failed to set request body")
//     }
// }
