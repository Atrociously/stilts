use std::collections::HashMap;

use camino::Utf8PathBuf;
use stilts_parser::Delims;

#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{Language, pathing::expand_path};

#[derive(Debug)]
pub struct TemplateNode {
    pub data: TemplateData,
    pub escape_override: Option<syn::Path>,
    pub trim_override: Option<bool>,
}

impl TemplateNode {
    pub fn load<L: Language>(lang: &L, cfg: &Config, opts: TemplateOptions) -> Result<Self, ()> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TemplateData {
    pub source_path: Option<Utf8PathBuf>,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TemplateOptions {
    pub source: TemplateSource,
    pub escape: Option<syn::Path>,
    pub trim: Option<bool>,
    pub block: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum TemplateSource {
    Literal(String),
    File(String),
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct Config {
    pub template_dir: Utf8PathBuf,
    pub trim: bool,
    pub delimiters: Delims,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deser::ident"))]
    pub writer_name: syn::Ident,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deser::escape_table"))]
    pub escape_table: HashMap<String, syn::Path>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            template_dir: expand_path("$CARGO_MANIFEST_DIR/templates"),
            trim: false,
            delimiters: Delims::default(),
            writer_name: syn::Ident::new("_w", proc_macro2::Span::call_site()),
            escape_table: default_escape_table(),
        }
    }
}

fn default_escape_table() -> HashMap<String, syn::Path> {
    [
        (
            "html".to_string(),
            syn::parse_str("::stilts::escaping::Html").unwrap(),
        ),
        (
            "htm".to_string(),
            syn::parse_str("::stilts::escaping::Html").unwrap(),
        ),
    ]
    .into()
}

#[cfg(feature = "serde")]
mod deser {
    use std::collections::HashMap;

    use serde::{Deserialize, Deserializer};

    pub fn ident<'de, D>(deserializer: D) -> Result<syn::Ident, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)
            .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
    }

    pub fn escape_table<'de, D>(deserializer: D) -> Result<HashMap<String, syn::Path>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut table = super::default_escape_table();
        let parsed = HashMap::<String, Vec<String>>::deserialize(deserializer)?;

        for (path, extensions) in parsed {
            let path: syn::Path =
                syn::parse_str(&path).map_err(<D::Error as serde::de::Error>::custom)?;
            for ext in extensions {
                table.insert(ext, path.clone());
            }
        }
        Ok(table)
    }
}
