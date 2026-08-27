use std::collections::HashMap;

use camino::Utf8PathBuf;
use stilts_parser::{
    Delims,
    types::{ItemBlock, Root},
};

#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{error::TemplateError, file_system::FileSystem, pathing::expand_path};

#[derive(Debug)]
pub struct TemplateNode {
    pub data: TemplateData,
    pub escape_override: Option<syn::Path>,
    pub trim_override: Option<bool>,
    pub root: Root<'static>,
    pub blocks: HashMap<String, ItemBlock<'static>>,
}

impl TemplateNode {
    pub fn load<FS: FileSystem>(
        fs: &mut FS,
        cfg: &TemplateConfig,
        opts: TemplateOptions,
    ) -> Result<Self, TemplateError> {
        let data = match opts.source {
            TemplateSource::Literal(content) => Ok(TemplateData {
                content,
                source_path: None,
            }),
            TemplateSource::File(path) => {
                let path = cfg.template_dir.join(&path);
                Result::<_, TemplateError>::Ok(TemplateData {
                    content: fs.read_to_string(&path)?,
                    source_path: Some(path),
                })
            }
        }?;

        let root =
            stilts_parser::parse_template(&data.content, cfg.delimiters.clone())?.into_owned();

        Ok(Self {
            data,
            escape_override: opts.escape,
            trim_override: opts.trim,
            root,
            blocks: HashMap::new(),
        })
    }

    pub fn trim(&self, cfg: &TemplateConfig) -> bool {
        self.trim_override.unwrap_or(cfg.trim)
    }

    pub fn escape_implementation<'a>(&'a self, cfg: &'a TemplateConfig) -> &'a syn::Path {
        self.escape_override.as_ref().unwrap_or_else(|| {
            let ext = self
                .data
                .source_path
                .as_ref()
                .and_then(|p| p.extension())
                .unwrap_or_default();
            cfg.escaper(ext)
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TemplateData {
    pub source_path: Option<Utf8PathBuf>,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TemplateOptions {
    pub source: TemplateSource,
    pub escape: Option<syn::Path>,
    pub trim: Option<bool>,
    pub block: Option<String>,
}

impl TemplateOptions {
    pub fn with_source(&self, new_source: TemplateSource) -> Self {
        Self {
            source: new_source,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum TemplateSource {
    Literal(String),
    File(String),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct TemplateConfig {
    pub template_dir: Utf8PathBuf,
    pub trim: bool,
    pub delimiters: Delims,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deser::ident"))]
    pub writer_name: syn::Ident,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deser::escape_table"))]
    pub escape_table: HashMap<String, syn::Path>,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deser::path"))]
    pub default_escaper: syn::Path,
}

impl TemplateConfig {
    pub fn escaper(&self, ext: &str) -> &syn::Path {
        self.escape_table.get(ext).unwrap_or(&self.default_escaper)
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        TemplateConfig {
            template_dir: expand_path("$CARGO_MANIFEST_DIR/templates"),
            trim: false,
            delimiters: Delims::default(),
            writer_name: syn::Ident::new("_w", proc_macro2::Span::call_site()),
            escape_table: default_escape_table(),
            default_escaper: syn::parse_str("::stilts::escaping::Empty").unwrap(),
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

    use serde::{Deserialize, Deserializer, de::Error};

    pub fn ident<'de, D>(deserializer: D) -> Result<syn::Ident, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)
            .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
    }

    pub fn path<'de, D>(deserializer: D) -> Result<syn::Path, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| syn::parse_str(s).map_err(D::Error::custom))
    }
    pub fn escape_table<'de, D>(deserializer: D) -> Result<HashMap<String, syn::Path>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut table = super::default_escape_table();
        let parsed = HashMap::<String, Vec<String>>::deserialize(deserializer)?;

        for (path, extensions) in parsed {
            let path: syn::Path = syn::parse_str(&path).map_err(D::Error::custom)?;
            for ext in extensions {
                table.insert(ext, path.clone());
            }
        }
        Ok(table)
    }
}
