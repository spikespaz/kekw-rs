use std::borrow::Cow;
use std::fmt;
use std::fmt::Write as _;

use kekw_macros::DerefNewType;
use url::Url;

#[derive(Debug, Default, DerefNewType, PartialEq)]
pub struct QueryPairs<'s>(#[deref] Cow<'s, str>);

impl QueryPairs<'_> {
    pub fn as_pairs(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.strip_prefix('/')
            .and_then(|s| s.strip_prefix('?'))
            .unwrap_or(self)
            .split('&')
            .map(|s| {
                s.split_once('=')
                    .map(|(lhs, rhs)| (lhs, Some(rhs)))
                    .unwrap_or((s, None))
            })
    }

    pub fn into_url<U>(self, base: U) -> Result<Url, <Url as TryFrom<U>>::Error>
    where
        Url: TryFrom<U>,
    {
        let mut url = Url::try_from(base)?;
        url.set_query(Some(self.as_ref()));
        Ok(url)
    }
}

impl From<String> for QueryPairs<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'s> From<&'s str> for QueryPairs<'s> {
    fn from(value: &'s str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl fmt::Display for QueryPairs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl<'s, K, V> FromIterator<(K, Option<V>)> for QueryPairs<'s>
where
    K: AsRef<str>,
    V: fmt::Display,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, Option<V>)>,
    {
        let mut buf = String::new();
        iter.into_iter().enumerate().for_each(|(i, pair)| {
            let sep = if i == 0 { '?' } else { '&' };
            match pair {
                (lhs, Some(rhs)) => write!(buf, "{sep}{}={rhs}", lhs.as_ref()).unwrap(),
                (lhs, None) => write!(buf, "{sep}{}", lhs.as_ref()).unwrap(),
            }
        });
        Self(Cow::Owned(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::QueryPairs;

    #[test]
    fn test_eq() {
        let query = QueryPairs::from_iter([
            ("foo", Some("a".into())),
            ("bar", Some("b".into())),
            ("baz", Some(String::from("c"))),
            ("quox", None),
            ("foobar", Some(5.to_string())),
            ("foobaz", Some("".into())),
            ("foobar", Some(8.9.to_string())),
        ]);

        dbg!(&query);
        eprintln!("{}", &query);

        assert_eq!(
            query.to_string(),
            "?foo=a&bar=b&baz=c&quox&foobar=5&foobaz=&foobar=8.9"
        )
    }
}
