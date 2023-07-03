//! Stilts is a templating language for rust that lets users embed rust code directly into their
//! template.
//!

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::Write;

pub use extensions::{DebugExt, DisplayExt, SerializeExt};
pub use stilts_macros::Template;

mod extensions;

/// The main template trait that is implemented by the derive macro
pub trait Template {
    /// Required to run the render function which passes a string as the writer
    fn fmt(&self, writer: &mut (impl Write + ?Sized)) -> std::fmt::Result;

    /// Render the template to a string
    fn render(&self) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        self.fmt(&mut out)?;
        Ok(out)
    }
}
