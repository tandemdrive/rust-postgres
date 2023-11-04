use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, Error, GenericParam, Generics, Ident, Path, PathSegment, Type,
    TypeParamBound,
};

use crate::{case::RenameRule, overrides::Overrides};

pub struct Field {
    pub name: String,
    pub ident: Ident,
    pub type_: Type,
}

impl Field {
    pub fn parse(raw: &syn::Field, rename_all: Option<RenameRule>) -> Result<Field, Error> {
        let overrides = Overrides::extract(&raw.attrs, false)?;
        let ident = raw.ident.as_ref().unwrap().clone();

        // field level name override takes precendence over container level rename_all override
        let name = if let Some(n) = overrides.name {
            n
        } else {
            let name = ident.to_string();
            let stripped = name.strip_prefix("r#").map(String::from).unwrap_or(name);

            match rename_all {
                Some(rule) => rule.apply_to_field(&stripped),
                None => stripped,
            }
        };

        Ok(Field {
            name,
            ident,
            type_: raw.ty.clone(),
        })
    }
}

pub(crate) fn append_generic_bound(mut generics: Generics, bound: &TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(param) = param {
            param.bounds.push(bound.clone());
        }
    }
    generics
}

pub(crate) fn new_derive_path(last: PathSegment) -> Path {
    let mut path = Path {
        leading_colon: None,
        segments: Punctuated::new(),
    };
    path.segments
        .push(Ident::new("postgres_types", Span::call_site()).into());
    path.segments.push(last);
    path
}
