use async_net::{AsyncToSocketAddrs, TcpListener, TcpStream};
use futures_lite::io::{self, BufReader};
use futures_lite::{AsyncBufReadExt as _, AsyncWriteExt as _, StreamExt as _};

use crate::endpoints::{AuthCodeAllowed, AuthCodeDenied};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    AuthDenied(#[from] AuthCodeDenied),
    #[error("{0}")]
    ParseQuery(#[from] serde_qs::Error),
}

pub async fn await_auth_code(
    addrs: impl AsyncToSocketAddrs,
    max_tries: usize,
) -> Result<AuthCodeAllowed, Error> {
    let listener = TcpListener::bind(addrs).await?;
    let mut incoming = listener.incoming();

    let mut attempt = 0;
    loop {
        dbg!(attempt);
        let Some(stream) = incoming.next().await else {
            break Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "TCP stream unexpectedly yielded None",
            )
            .into());
        };
        let stream = stream?;
        match receive_query_params(stream).await {
            Ok(Some(query)) => break parse_query_params(query),
            Ok(None) | Err(_) if attempt <= max_tries => {
                attempt += 1;
                continue;
            }
            Ok(None) => todo!(),
            Err(e) => break Err(e.into()),
        }
    }
}

async fn receive_query_params(mut stream: TcpStream) -> io::Result<Option<String>> {
    let mut reader = BufReader::new(&mut stream);

    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;
    let request_line = request_line.trim();

    // GET /path/to/resource?query=param HTTP/1.1
    let parts = request_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid request line",
        ));
    }
    let _method = parts[0];
    let path_and_query = parts[1];
    let version = parts[2];
    assert_eq!(version, "HTTP/1.1");

    // TODO: Validate headers
    let mut headers = Vec::new();
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line).await?;
        let header_line = header_line.trim();
        if header_line.is_empty() {
            break; // End of headers
        }
        let parts: Vec<&str> = header_line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let name = parts[0].trim();
            let value = parts[1].trim();
            headers.push((name.to_string(), value.to_string()));
        }
    }

    let (_path, query) = path_and_query
        .split_once('?')
        .map(|(path, query)| (path, Some(query)))
        .unwrap_or_else(|| (path_and_query, None));

    stream
        .write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes())
        .await?;

    Ok(query.map(ToString::to_string))
}

/// Incomplete: requires deserializing to be replaced with custom `Deserializer``.
/// Perhaps also convert the `FromStr` impl on `QueryString` derives to a serde `Serializer`.
/// When I do this, the `Deserializer` must be custom, and respect attributes on fields of structures, enums, and tuples.
/// There needs to be a `deserialize_proxy = fn(T) -> String` attribute that allows
/// affecting the serializer's operation. Perhaps an automatically-derived `QueryStrings` trait,
/// that implements `Serialize` for `QueryStrings`?
fn parse_query_params(query: impl AsRef<str>) -> Result<AuthCodeAllowed, Error> {
    if let Ok(allowed) = serde_qs::from_str(query.as_ref()) {
        Ok(allowed)
    } else {
        Err(Error::AuthDenied(serde_qs::from_str::<AuthCodeDenied>(
            query.as_ref(),
        )?))
    }
}
