use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Expr, ExprLit, Ident, Lit, LitStr, Variant};

use crate::ext::*;

pub(crate) struct VariantStrings(HashMap<Ident, LitStr>);

pub(crate) struct VariantExprs(HashMap<Ident, Expr>);

impl VariantStrings {
    pub fn from_variants<I>(
        ident: &I,
        variants: &mut Punctuated<Variant, Comma>,
    ) -> syn::Result<Self>
    where
        I: ?Sized,
        Ident: PartialEq<I>,
    {
        let mut map = HashMap::new();
        for variant in variants.iter_mut() {
            if let Some(attr) = variant.attrs.pop_by_ident(ident) {
                if let Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Str(variant_str),
                }) = attr.parse_args::<Expr>()?
                {
                    map.insert(variant.ident.clone(), variant_str);
                }
            }
        }
        Ok(Self(map))
    }

    pub fn as_iters(&self) -> (Vec<&Ident>, Vec<&LitStr>) {
        (self.keys().collect(), self.values().collect())
    }
}

impl Deref for VariantStrings {
    type Target = HashMap<Ident, LitStr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariantStrings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl VariantExprs {
    pub fn from_variants<I>(
        ident: &I,
        variants: &mut Punctuated<Variant, Comma>,
    ) -> syn::Result<Self>
    where
        I: ?Sized,
        Ident: PartialEq<I>,
    {
        let mut map = HashMap::new();
        for variant in variants.iter_mut() {
            if let Some(attr) = variant.attrs.pop_by_ident(ident) {
                map.insert(variant.ident.clone(), attr.parse_args::<Expr>()?);
            }
        }
        Ok(Self(map))
    }

    pub fn as_iters(&self) -> (Vec<&Ident>, Vec<&Expr>) {
        (self.keys().collect(), self.values().collect())
    }
}

impl Deref for VariantExprs {
    type Target = HashMap<Ident, Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariantExprs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
