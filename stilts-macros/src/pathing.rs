use std::borrow::Cow;

use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

pub fn expand_path(path: impl Into<Utf8PathBuf>) -> Utf8PathBuf {
    let path = path.into();
    let path: Utf8PathBuf = path
        .components()
        .map(|c| {
            if c.as_str().starts_with('$') {
                let s = std::env::var(&c.as_str()[1..]).unwrap_or_else(|_| c.as_str().to_string());
                Cow::Owned(Utf8PathBuf::from(s))
            } else {
                Cow::Borrowed(Utf8Path::new(c.as_str()))
            }
        })
        .collect();

    if path.is_relative() {
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let root = Utf8PathBuf::from(root);
        root.join(path)
    } else {
        path
    }
}
