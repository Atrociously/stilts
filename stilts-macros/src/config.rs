use std::collections::HashMap;

use cargo_metadata::{camino::Utf8PathBuf, MetadataCommand};
use serde::{Deserialize, Deserializer};

use crate::{err, pathing::expand_path};

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub template_dir: Utf8PathBuf,
    pub trim: bool,
    #[serde(deserialize_with = "Config::deserialize_writer_name")]
    pub writer_name: syn::Ident,
    #[serde(rename = "escape", deserialize_with = "Config::deserialize_escape_table")]
    escape_table: HashMap<String, syn::Path>,
}

impl Config {
    pub fn load() -> syn::Result<Self> {
        let pkg = std::env::var("CARGO_PKG_NAME").unwrap();
        let metadata = MetadataCommand::new()
            .manifest_path("Cargo.toml")
            .exec()
            .map_err(|e| err!(e))?;

        let meta = metadata.packages.iter()
            .find(|p| p.name == pkg)
            .and_then(|p| p.metadata.as_object())
            .and_then(|meta| meta.get("stilts"));

        if let Some(meta) = meta {
            let mut config = Config::deserialize(meta).map_err(|e| err!(e))?;
            config.template_dir = crate::pathing::expand_path(config.template_dir);
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn escaper(&self, ext: &str) -> syn::Path {
        self.escape_table.get(ext)
            .cloned()
            .unwrap_or_else(|| syn::parse_str("::stilts::escaping::Empty").unwrap())
    }

    fn deserialize_writer_name<'de, D>(deserializer: D) -> Result<syn::Ident, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)
            .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
    }

    // extend the default table with the parsed version users can overwrite things if they want
    fn deserialize_escape_table<'de, D>(deserializer: D) -> Result<HashMap<String, syn::Path>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut table = Self::default_escape_table();
        let parsed = HashMap::<String, String>::deserialize(deserializer)?;

        for (key, val) in parsed {
            let key = key.parse().map_err(<D::Error as serde::de::Error>::custom)?;
            let val = syn::parse_str(&val).map_err(<D::Error as serde::de::Error>::custom)?;
            table.insert(key, val);
        }
        Ok(table)
    }

    fn default_escape_table() -> HashMap<String, syn::Path> {
        [
            ("html".to_string(), syn::parse_str("::stilts::escaping::Html").unwrap()),
            ("htm".to_string(), syn::parse_str("::stilts::escaping::Html").unwrap()),
        ].into()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            template_dir: expand_path("$CARGO_MANIFEST_DIR/templates"),
            trim: false,
            writer_name: syn::Ident::new("_w", proc_macro2::Span::call_site()),
            escape_table: Self::default_escape_table(),
        }
    }
}
