use proc_macro2::Span;
use syn::{spanned::Spanned, Data, DeriveInput};
use syn::{Attribute, Generics, Ident, LitBool, LitStr, Path};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateSource {
    Literal(LitStr),
    File(LitStr),
}

impl TemplateSource {
    pub fn new_file(path: impl AsRef<str>) -> Self {
        Self::File(LitStr::new(path.as_ref(), Span::call_site()))
    }

    #[allow(dead_code)] // This is used in feature gated integrations
    pub fn as_path(&self) -> Option<String> {
        match self {
            Self::File(value) => Some(value.value()),
            _ => None,
        }
    }
}

pub struct TemplateAttrs {
    pub source: TemplateSource,
    pub escape: Option<Path>,
    pub trim: Option<bool>,
}

impl TemplateAttrs {
    pub fn parse(attrs: Vec<Attribute>) -> syn::Result<Self> {
        let attrs = attrs
            .into_iter()
            .filter(|attr| attr.path().is_ident(ATTR_NAME));

        let mut source = None;
        let mut escape = None;
        let mut trim = None;

        for attr in attrs {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") {
                    let value = meta.value()?;
                    let value: LitStr = value.parse()?;
                    source = Some(TemplateSource::File(value));
                }
                if meta.path.is_ident("content") {
                    let value = meta.value()?;
                    let value: LitStr = value.parse()?;
                    source = Some(TemplateSource::Literal(value));
                }
                if meta.path.is_ident("escape") {
                    let value = meta.value()?;
                    let value: Path = value.parse()?;
                    escape = Some(value);
                }
                if meta.path.is_ident("trim") {
                    let value = meta.value()?;
                    let value: LitBool = value.parse()?;
                    trim = Some(value.value)
                }
                Ok(())
            })?;
        }

        let source = source.ok_or_else(|| err!(r#"templates require a `path` or `content` attribute to find the template file e.g. `#[stilts(path = "index.html")]`"#))?;
        Ok(Self {
            source,
            escape,
            trim,
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
