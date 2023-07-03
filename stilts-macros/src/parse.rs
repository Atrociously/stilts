use syn::{spanned::Spanned, Data, DeriveInput};
use syn::{Attribute, Generics, Ident, LitStr};

use crate::{err, ATTR_NAME};

pub struct TemplateInput {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: TemplateAttrs,
    pub fields: Vec<Field>,
}

impl TemplateInput {
    pub fn parse(input: DeriveInput) -> syn::Result<Self> {
        match input.data {
            Data::Struct(data) => Ok(Self {
                ident: input.ident,
                generics: input.generics,
                attrs: TemplateAttrs::parse(input.attrs)?,
                fields: data
                    .fields
                    .into_iter()
                    .map(Field::parse)
                    .collect::<Result<_, _>>()?,
            }),
            Data::Enum(_) => Err(err!(input.ident, "enum templates are not supported")),
            Data::Union(_) => Err(err!(input.ident, "union templates are not supported")),
        }
    }
}

pub struct TemplateAttrs {
    pub path: LitStr,
}

impl TemplateAttrs {
    pub fn parse(attrs: Vec<Attribute>) -> syn::Result<Self> {
        let attrs = attrs
            .into_iter()
            .filter(|attr| attr.path().is_ident(ATTR_NAME));

        let mut path = None;

        for attr in attrs {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") {
                    let value = meta.value()?;
                    let value: LitStr = value.parse()?;
                    path = Some(value);
                }
                Ok(())
            })?;
        }

        Ok(Self {
            path: path.ok_or_else(|| err!(r#"templates require a `path` attribute to find the template file e.g. `#[stilts(path = "index.html")]`"#))?
        })
    }
}

pub struct Field {
    pub ident: Ident,
}

impl Field {
    pub fn parse(field: syn::Field) -> syn::Result<Self> {
        Ok(Self {
            ident: field.ident.ok_or_else(|| {
                syn::Error::new(
                    field.ty.span(),
                    "only structs with named fields are supported",
                )
            })?,
        })
    }
}
