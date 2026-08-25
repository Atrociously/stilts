use cargo_metadata::MetadataCommand;
use serde::Deserialize;

use crate::err;

pub fn load() -> syn::Result<stilts_lang::Config> {
    let pkg = std::env::var("CARGO_PKG_NAME").unwrap();
    let metadata = MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .exec()
        .map_err(|e| err!(e))?;

    let meta = metadata
        .packages
        .iter()
        .find(|p| p.name == pkg)
        .and_then(|p| p.metadata.as_object())
        .and_then(|meta| meta.get("stilts"));

    let mut config = if let Some(meta) = meta {
        stilts_lang::Config::deserialize(meta).map_err(|e| err!(e))?
    } else {
        stilts_lang::Config::default()
    };
    config.expand_path(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    Ok(config)
}
