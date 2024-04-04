mod ext;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Expr, ExprLit, Ident, ItemEnum, Lit, LitStr, Variant};

use self::ext::*;

static STATIC_STRING_ATTRIBUTE: &str = "static_str";

#[proc_macro_derive(VariantStrings, attributes(static_str))]
pub fn derive_variant_strings(item: TokenStream1) -> TokenStream1 {
    crate::proc_macro_impl! {
        let ItemEnum {
            attrs: _,
            vis: _,
            enum_token: _,
            ident,
            generics,
            brace_token: _,
            mut variants,
        } = ItemEnum::parse.parse(item)?;

        let map = VariantStrings::from_variants(&mut variants)?;
        let (variant_idents, static_strs) = map.as_iters();

        Ok(quote!(
            impl #generics ::std::convert::AsRef<str> for #ident #generics {
                fn as_ref(&self) -> &str {
                    match self {
                        #(#ident::#variant_idents => #static_strs,)*
                    }
                }
            }
        ))
    }
}

struct VariantStrings(HashMap<Ident, LitStr>);

impl VariantStrings {
    fn from_variants(variants: &mut Punctuated<Variant, Comma>) -> syn::Result<Self> {
        let mut map = HashMap::new();
        for variant in variants.iter_mut() {
            if let Some(attr) = variant.attrs.pop_by_ident(STATIC_STRING_ATTRIBUTE) {
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

    fn as_iters(&self) -> (Vec<&Ident>, Vec<&LitStr>) {
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
