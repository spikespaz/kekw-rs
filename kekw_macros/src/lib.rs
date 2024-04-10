mod ext;
mod parsers;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::token::Mut;
use syn::{Error, Expr, Field, Ident, Index, Item, ItemEnum, ItemStruct};

use self::ext::*;
use self::parsers::{VariantExprs, VariantStrings};

// For the VariantStrings, DisplayStrings, DebugExprs and VariantFromStr derives.
static STATIC_STRING_ATTRIBUTE: &str = "static_str";

// For the DisplayStrings derive.
static DISPLAY_EXPRESSION_ATTRIBUTE: &str = "display";

// For the DebugExprs derive.
static DEBUG_EXPRESSION_ATTRIBUTE: &str = "debug";

// For the VariantFromStr derive.
static FROM_STRING_ATTRIBUTE: &str = "from_str";

// For the NewTypeDeref derive.
static DEREF_FIELD_ATTRIBUTE: &str = "deref";

// For the QueryParams derive.
static QUERY_PARAM_ATTRIBUTE: &str = "query_param";
static SKIP_NAMED_FIELD_META_IDENT: &str = "skip";
static SKIP_IF_PREDICATE_META_IDENT: &str = "skip_if";
static PROXY_PREDICATE_META_IDENT: &str = "proxy";

pub(crate) fn proc_macro_impl(tokens: impl FnOnce() -> syn::Result<TokenStream2>) -> TokenStream1 {
    tokens()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derivers implement `AsRef<str>`.
///
/// Define static strings per variant with the `static_str` attribute.
#[proc_macro_derive(VariantStrings, attributes(static_str))]
pub fn derive_variant_strings(item: TokenStream1) -> TokenStream1 {
    proc_macro_impl(|| {
        let ItemEnum {
            ident,
            generics,
            mut variants,
            ..
        } = ItemEnum::parse.parse(item)?;

        let map = VariantStrings::from_variants(STATIC_STRING_ATTRIBUTE, &mut variants)?;
        let (variants, values) = map.as_iters();

        Ok(quote!(
            impl #generics ::std::convert::AsRef<str> for #ident #generics {
                fn as_ref(&self) -> &str {
                    match self {
                        #(#ident::#variants => #values,)*
                    }
                }
            }
        ))
    })
}

macro_rules! impl_derive_format_strings {
    (
        $(#[$meta:meta])*
        derive: $derive:ident,
        trait: $trait:ident,
        fn: $ident:ident,
        attr: $attr_name:ident
    ) => {
        $(#[$meta])*
        #[proc_macro_derive($derive, attributes(static_str, debug))]
        pub fn $ident(item: TokenStream1) -> TokenStream1 {
            proc_macro_impl(|| {
                let ItemEnum {
                    ident,
                    generics,
                    mut variants,
                    ..
                } = ItemEnum::parse.parse(item)?;

                let mut str_map = VariantStrings::from_variants(STATIC_STRING_ATTRIBUTE, &mut variants)?;
                let expr_map = VariantExprs::from_variants($attr_name, &mut variants)?;

                expr_map.keys().for_each(|k| {
                    if str_map.contains_key(k) {
                        str_map.remove(k);
                    }
                });

                let (str_variants, str_values) = str_map.as_iters();
                let (variants, values) = expr_map.as_iters();

                Ok(quote!(
                    impl #generics ::std::fmt::$trait for #ident #generics {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                            match self {
                                #(#ident::#str_variants => f.write_str(#str_values),)*
                                #(#ident::#variants => ::std::fmt::$trait::fmt(&#values, f),)*
                            }
                        }
                    }
                ))
            })
        }
    };
}

impl_derive_format_strings!(
    /// Derivers implement [`std::fmt::Display`].
    ///
    /// Define expressions to use for formatting each variant with the `display` attribute.
    ///
    /// This derive is interoperable with [`derive@VariantStrings`]. When a variant
    /// does not does have the `display` attribute, but *does* have `static_str`,
    /// that literal will be used instead.
    derive: DisplayStrings,
    trait: Display,
    fn: derive_display_strings,
    attr: DISPLAY_EXPRESSION_ATTRIBUTE
);

impl_derive_format_strings!(
    /// Derivers implement [`std::fmt::Debug`].
    ///
    /// Define expressions to use for formatting each variant with the `debug` attribute.
    ///
    /// This derive is interoperable with [`derive@VariantStrings`]. When a variant
    /// does not does have the `debug` attribute, but *does* have `static_str`,
    /// that literal will be used instead.
    derive: DebugExprs,
    trait: Debug,
    fn: derive_debug_exprs,
    attr: DEBUG_EXPRESSION_ATTRIBUTE
);

#[proc_macro_derive(VariantFromStr, attributes(static_str, from_str))]
pub fn derive_variant_from_str(item: TokenStream1) -> TokenStream1 {
    proc_macro_impl(|| {
        let ItemEnum {
            ident,
            generics,
            mut variants,
            ..
        } = ItemEnum::parse.parse(item)?;

        let mut str_map = VariantStrings::from_variants(STATIC_STRING_ATTRIBUTE, &mut variants)?;
        let from_map = VariantExprs::from_variants(FROM_STRING_ATTRIBUTE, &mut variants)?;

        from_map.keys().for_each(|k| {
            if str_map.contains_key(k) {
                str_map.remove(k);
            }
        });

        let (str_variants, str_values) = str_map.as_iters();
        let (variants, values) = from_map.as_iters();

        let err_ident = Ident::new(&format!("Parse{ident}Error"), Span::call_site());

        Ok(quote!(
            #[derive(::std::clone::Clone, ::std::fmt::Debug, ::kekw_macros::NewTypeFrom)]
            pub struct #err_ident(::std::string::String);

            impl ::std::fmt::Display for #err_ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    write!(f, "string did not match any variant: {}", self.0)
                }
            }

            impl ::std::error::Error for #err_ident {}

            impl #generics ::std::str::FromStr for #ident #generics {
                type Err = #err_ident;

                fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                    match s {
                        #(#str_values => ::std::result::Result::Ok(#ident::#str_variants),)*
                        #(#values => ::std::result::Result::Ok(#ident::#variants),)*
                        _ => ::std::result::Result::Err(#err_ident(s.to_owned()))
                    }
                }
            }
        ))
    })
}

#[proc_macro_derive(DerefNewType, attributes(deref))]
pub fn derive_deref_new_type(item: TokenStream1) -> TokenStream1 {
    proc_macro_impl(|| {
        let ItemStruct {
            ident,
            generics,
            fields,
            ..
        } = ItemStruct::parse.parse(item)?;

        let (field_index, field_ident, target_ty, mutable) = fields
            .iter()
            .enumerate()
            .find_map(|(i, field)| {
                field
                    .attrs
                    .get_by_ident(DEREF_FIELD_ATTRIBUTE)
                    .map(|attr| (i, &field.ident, &field.ty, attr.parse_args::<Mut>().is_ok()))
            })
            .ok_or_else(|| {
                Error::new(
                    fields.span(),
                    format!(
                        "require a single field to be marked with `#[{}]`",
                        DEREF_FIELD_ATTRIBUTE
                    ),
                )
            })?;

        macro_rules! impl_deref_new_type {
            ($field_name:ident) => {
                quote!(
                    impl #generics ::std::ops::Deref for #ident #generics {
                        type Target = #target_ty;

                        fn deref(&self) -> &Self::Target {
                            &self.#$field_name
                        }
                    }
                )
            };
            (mut $field_name:ident) => {
                quote!(
                    impl #generics ::std::ops::DerefMut for #ident #generics {
                        fn deref_mut(&mut self) -> &mut Self::Target {
                            &mut self.#$field_name
                        }
                    }
                )
            };
        }

        let mut tokens = TokenStream2::new();

        if let Some(field_ident) = field_ident {
            tokens.extend(impl_deref_new_type!(field_ident));
            if mutable {
                tokens.extend(impl_deref_new_type!(mut field_ident));
            }
        } else {
            let field_index = Index::from(field_index);
            tokens.extend(impl_deref_new_type!(field_index));
            if mutable {
                tokens.extend(impl_deref_new_type!(mut field_index));
            }
        }

        Ok(tokens)
    })
}

#[proc_macro_derive(NewTypeFrom)]
pub fn derive_new_type_from(item: TokenStream1) -> TokenStream1 {
    proc_macro_impl(|| {
        let ItemStruct {
            ident,
            generics,
            fields,
            ..
        } = ItemStruct::parse.parse(item)?;

        if fields.len() != 1 {
            Err(Error::new(fields.span(), "must have a single field"))
        } else {
            let from_ty = &fields.iter().next().unwrap().ty;
            Ok(quote!(
                impl #generics ::std::convert::From<#from_ty> for #ident #generics {
                    fn from(other: #from_ty) -> Self {
                        Self(other)
                    }
                }
            ))
        }
    })
}

#[proc_macro_derive(DeserializeFromStr)]
pub fn derive_deserialize_from_str(item: TokenStream1) -> TokenStream1 {
    proc_macro_impl(|| {
        let item = Item::parse.parse(item)?;
        let ident = item.ident().ok_or_else(|| {
            Error::new(item.span(), "this derive only applies to type definitions")
        })?;
        let visitor_ident = Ident::new(&format!("{ident}Visitor"), Span::call_site());

        Ok(quote!(
            struct #visitor_ident;

            impl<'de> ::serde::de::Visitor<'de> for #visitor_ident {
                type Value = #ident;

                fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    v.parse().or_else(|e| ::std::result::Result::Err(E::custom(e)))
                }

                fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    ::std::write!(
                        f,
                        "a string that can be parsed by `<{} as FromStr>::from_str`",
                        ::std::stringify!(#ident)
                    )
                }
            }

            impl<'de> ::serde::de::Deserialize<'de> for #ident {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: ::serde::de::Deserializer<'de>,
                {
                    deserializer.deserialize_str(#visitor_ident)
                }
            }
        ))
    })
}

#[proc_macro_derive(QueryParams, attributes(query_param))]
pub fn derive_query_params(item: TokenStream1) -> TokenStream1 {
    proc_macro_impl(|| {
        let ItemStruct {
            ident,
            generics,
            fields,
            ..
        } = ItemStruct::parse.parse(item)?;

        let fields = fields.named().ok_or_else(|| {
            Error::new(
                fields.span(),
                "can only derive `QueryPairs` for structs which have named fields",
            )
        })?;

        let mut fmt_body = TokenStream2::new();

        for (i, field) in fields.into_iter().enumerate() {
            let Field { attrs, ident, .. } = field;
            let field_ident = ident.as_ref().unwrap();

            let mut skip = false;
            let mut skip_if = None;
            let mut proxy = None;

            if let Some(attr) = attrs.get_by_ident(QUERY_PARAM_ATTRIBUTE) {
                attr.parse_nested_meta(|meta| {
                    #[allow(clippy::unit_arg)]
                    if meta.path.is_ident(SKIP_NAMED_FIELD_META_IDENT) {
                        Ok(skip = true)
                    } else if meta.path.is_ident(SKIP_IF_PREDICATE_META_IDENT) {
                        Ok(skip_if = Some(meta.value()?.parse::<Expr>()?))
                    } else if meta.path.is_ident(PROXY_PREDICATE_META_IDENT) {
                        Ok(proxy = Some(meta.value()?.parse::<Expr>()?))
                    } else {
                        Err(Error::new(meta.path.span(), "unknown meta"))
                    }
                })?;
            };

            if skip {
                continue;
            }

            let format = if i == 0 {
                format!("{field_ident}={{:#}}")
            } else {
                format!("&{field_ident}={{:#}}")
            };

            let value_expr = if let Some(proxy) = proxy {
                quote!((#proxy)(&self.#field_ident))
            } else {
                quote!(&self.#field_ident)
            };

            let tokens = if let Some(skip_if) = skip_if {
                quote!(
                    if !(#skip_if)(&self.#field_ident) {
                        write!(f, #format, #value_expr)?;
                    }
                )
            } else {
                quote!(
                    write!(f, #format, #value_expr)?;
                )
            };

            fmt_body.extend(tokens);
        }

        Ok(quote!(
            impl #generics ::std::fmt::Display for #ident #generics {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    #fmt_body
                    Ok(())
                }
            }
        ))
    })
}
