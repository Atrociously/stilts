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
                    .filter_map(|field| Field::parse(field))
                    .collect::<Result<_, _>>()?,
            }),
            Data::Enum(_) => Err(err!(input.ident, "enum templates are not supported")),
            Data::Union(_) => Err(err!(input.ident, "union templates are not supported")),
        }
    }
}

pub struct TemplateAttrs {
    pub source: stilts_lang::TemplateSource,
    pub escape: Option<Path>,
    pub trim: Option<bool>,
    pub block: Option<String>,
}

impl TemplateAttrs {
    pub fn into_options(self) -> stilts_lang::TemplateOptions {
        stilts_lang::TemplateOptions {
            overrides: stilts_lang::Overrides {
                escape_implementation: self.escape,
                trim: self.trim,
            },
            source: self.source,
            only_block: self.block,
        }
    }

    pub fn parse(attrs: Vec<Attribute>) -> syn::Result<Self> {
        let attrs = attrs
            .into_iter()
            .filter(|attr| attr.path().is_ident(ATTR_NAME));

        let mut source = None;
        let mut escape = None;
        let mut trim = None;
        let mut block = None;

        for attr in attrs {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") {
                    let value = meta.value()?;
                    let value: LitStr = value.parse()?;
                    source = Some(stilts_lang::TemplateSource::File(value.value().into()));
                }
                if meta.path.is_ident("content") {
                    let value = meta.value()?;
                    let value: LitStr = value.parse()?;
                    source = Some(stilts_lang::TemplateSource::Content(value.value()));
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
                if meta.path.is_ident("block") {
                    let value = meta.value()?;
                    let value: LitStr = value.parse()?;
                    block = Some(value.value());
                }
                Ok(())
            })?;
        }

        let source = source.ok_or_else(|| err!(r#"templates require a `path` or `content` attribute to find the template file e.g. `#[stilts(path = "index.html")]`"#))?;
        Ok(Self {
            source,
            escape,
            trim,
            block,
        })
    }
}

pub struct Field {
    pub ident: Ident,
}

impl Field {
    pub fn parse(field: syn::Field) -> Option<syn::Result<Self>> {
        let mut ignore = false;
        for attr in field.attrs {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("ignore") {
                    ignore = true;
                }
                Ok(())
            });
        }
        if ignore {
            return None;
        }
        let Some(ident) = field.ident else {
            return Some(Err(syn::Error::new(
                field.ty.span(),
                "only structs with named fields are supported",
            )));
        };
        Some(Ok(Self { ident }))
    }
}
