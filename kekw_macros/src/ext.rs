use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Field, Fields, FieldsNamed, Ident, Item, ItemEnum, Path, PathSegment, Type};

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
        self.find_by_ident(ident).map(|i| self.remove(i))
    }
}

pub(crate) trait ItemExt {
    fn ident(&self) -> Option<&Ident>;
}

impl ItemExt for Item {
    fn ident(&self) -> Option<&Ident> {
        match self {
            Item::Const(it) => Some(&it.ident),
            Item::Enum(it) => Some(&it.ident),
            Item::ExternCrate(it) => Some(&it.ident),
            Item::Fn(it) => Some(&it.sig.ident),
            Item::Struct(it) => Some(&it.ident),
            Item::Trait(it) => Some(&it.ident),
            Item::TraitAlias(it) => Some(&it.ident),
            Item::Type(it) => Some(&it.ident),
            Item::Union(it) => Some(&it.ident),
            _ => None,
        }
    }
}

pub(crate) trait FieldsExt {
    fn named(&self) -> Option<&Punctuated<Field, Comma>>;

    fn named_mut(&mut self) -> Option<&mut Punctuated<Field, Comma>>;

    fn into_named(self) -> Option<Punctuated<Field, Comma>>;
}

impl FieldsExt for Fields {
    fn named(&self) -> Option<&Punctuated<Field, Comma>> {
        if let Self::Named(FieldsNamed { named, .. }) = self {
            Some(named)
        } else {
            None
        }
    }

    fn named_mut(&mut self) -> Option<&mut Punctuated<Field, Comma>> {
        if let Self::Named(FieldsNamed { named, .. }) = self {
            Some(named)
        } else {
            None
        }
    }

    fn into_named(self) -> Option<Punctuated<Field, Comma>> {
        if let Self::Named(FieldsNamed { named, .. }) = self {
            Some(named)
        } else {
            None
        }
    }
}
