use std::borrow::Cow;
use std::collections::HashMap;

use kekw_macros::{DerefNewType, NewTypeFrom};

// #[derive(Debug, Clone, Default, DerefNewType)]
// struct QueryPairs<'a> {
//     source: Cow<'a, str>,
//     #[deref]
//     pairs: &'a [(&'a str, Option<&'a str>)],
// }

#[derive(Debug, Clone, Default, DerefNewType, NewTypeFrom)]
pub struct QueryPairs<'a>(#[deref] Vec<(&'a str, Option<&'a str>)>);

impl<'a> From<&'a str> for QueryPairs<'a> {
    fn from(s: &'a str) -> Self {
        Self(
            s.strip_prefix('/')
                .and_then(|s| s.strip_prefix('?'))
                .unwrap_or(s)
                .split('&')
                .map(|s| {
                    s.split_once('=')
                        .map(|(lhs, rhs)| (lhs, Some(rhs)))
                        .unwrap_or((s, None))
                })
                .collect::<Vec<_>>(),
        )
    }
}

impl<'a> QueryPairs<'a> {
    pub fn to_vec(self) -> Vec<(String, Option<String>)> {
        self.iter()
            .map(|(lhs, rhs)| ((*lhs).to_owned(), (*rhs).map(ToOwned::to_owned)))
            .collect()
    }

    pub fn to_map(self) -> HashMap<String, Option<String>> {
        HashMap::from_iter(self.to_vec())
    }
}
