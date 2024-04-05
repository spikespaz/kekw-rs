mod ext;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Expr, ExprLit, Ident, ItemEnum, Lit, LitStr, Variant};

use self::ext::*;

static STATIC_STRING_ATTRIBUTE: &str = "static_str";
static DISPLAY_EXPRESSION_ATTRIBUTE: &str = "display";
static DEBUG_EXPRESSION_ATTRIBUTE: &str = "debug";

#[proc_macro_derive(VariantStrings, attributes(static_str))]
pub fn derive_variant_strings(item: TokenStream1) -> TokenStream1 {
    crate::proc_macro_impl! {
        let ItemEnum {
            ident,
            generics,
            mut variants,
            ..
        } = ItemEnum::parse.parse(item)?;

        let map = VariantStrings::from_variants(STATIC_STRING_ATTRIBUTE, &mut variants)?;
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

macro_rules! impl_derive_format_strings {
    ($trait:ident, $derive:ident, $ident:ident, $attr_name:ident) => {
        #[proc_macro_derive($derive, attributes(static_str, debug))]
        pub fn $ident(item: TokenStream1) -> TokenStream1 {
            crate::proc_macro_impl! {
                let ItemEnum {
                    ident,
                    generics,
                    mut variants,
                    ..
                } = ItemEnum::parse.parse(item)?;

                let mut str_map = VariantStrings::from_variants(STATIC_STRING_ATTRIBUTE, &mut variants)?;
                let expr_map = VariantExprs::from_variants($attr_name, &mut variants)?;

                expr_map.keys().for_each(|k| if str_map.contains_key(k) { str_map.remove(k); });

                let (str_variants, str_values) = str_map.as_iters();
                let (expr_variants, expr_values) = expr_map.as_iters();

                Ok(quote!(
                    impl #generics ::std::fmt::$trait for #ident #generics {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                            match self {
                                #(#ident::#str_variants => f.write_str(#str_values),)*
                                #(#ident::#expr_variants => ::std::fmt::$trait::fmt(&#expr_values, f),)*
                            }
                        }
                    }
                ))
            }
        }
    };
}

impl_derive_format_strings!(
    Display,
    DisplayStrings,
    derive_display_strings,
    DISPLAY_EXPRESSION_ATTRIBUTE
);
impl_derive_format_strings!(
    Debug,
    DebugExprs,
    derive_debug_exprs,
    DEBUG_EXPRESSION_ATTRIBUTE
);

struct VariantStrings(HashMap<Ident, LitStr>);

struct VariantExprs(HashMap<Ident, Expr>);

impl VariantStrings {
    fn from_variants<I>(ident: &I, variants: &mut Punctuated<Variant, Comma>) -> syn::Result<Self>
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

impl VariantExprs {
    fn from_variants<I>(ident: &I, variants: &mut Punctuated<Variant, Comma>) -> syn::Result<Self>
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

    fn as_iters(&self) -> (Vec<&Ident>, Vec<&Expr>) {
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
