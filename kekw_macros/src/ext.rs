macro_rules! proc_macro_impl {
    ($($tt:tt)*) => {
        match (|| -> syn::Result<TokenStream2> {
            $($tt)*
        })() {
            Ok(tokens) => tokens.into(),
            Err(e) => e.to_compile_error().into(),
        }
    };
}
pub(crate) use proc_macro_impl;

macro_rules! err {
    ($tokens:expr, $message:expr) => {
        Err(syn::Error::new($tokens.span(), $message))
    };
}
pub(crate) use err;
use syn::{Attribute, Ident};

pub(crate) trait AttributesExt {
    fn find_by_ident<I>(&self, ident: &I) -> Option<usize>
    where
        I: ?Sized,
        Ident: PartialEq<I>;

    fn get_by_ident<I>(&self, ident: &I) -> Option<&Attribute>
    where
        I: ?Sized,
        Ident: PartialEq<I>;

    fn filter_by_ident<I>(&self, ident: &I) -> Vec<&Attribute>
    where
        I: ?Sized,
        Ident: PartialEq<I>;

    fn pop_by_ident<I>(&mut self, ident: &I) -> Option<Attribute>
    where
        I: ?Sized,
        Ident: PartialEq<I>;
}

impl AttributesExt for Vec<Attribute> {
    fn find_by_ident<I>(&self, ident: &I) -> Option<usize>
    where
        I: ?Sized,
        Ident: PartialEq<I>,
    {
        self.iter().position(|attr| attr.path().is_ident(ident))
    }

    fn get_by_ident<I>(&self, ident: &I) -> Option<&Attribute>
    where
        I: ?Sized,
        Ident: PartialEq<I>,
    {
        self.iter().find(|attr| attr.path().is_ident(ident))
    }

    fn filter_by_ident<I>(&self, ident: &I) -> Vec<&Attribute>
    where
        I: ?Sized,
        Ident: PartialEq<I>,
    {
        self.iter()
            .filter(|attr| attr.path().is_ident(ident))
            .collect()
    }

    fn pop_by_ident<I>(&mut self, ident: &I) -> Option<Attribute>
    where
        I: ?Sized,
        Ident: PartialEq<I>,
    {
        self.find_by_ident(ident).and_then(|i| Some(self.remove(i)))
    }
}
