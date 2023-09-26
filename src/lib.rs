//! Stilts is a rust-centric type safe template engine. It allows the users to 
//! create templates with arbitrary rust code within them, with a Jinja-like syntax.
//! For more in depth documentation on the language check out the [book](https://atrociously.github.io/stilts/).
//!
//! It works using a derive macro that outputs template rendering code for the 
//! rust compiler to type check.
//! 
//! ## Example
//! By default stilts looks for templates in the `$CARGO_MANIFEST_DIR/templates directory`
//! this setting can be changed in the configuration.
//!
//! Defining a template:
//! ```no_run
//! #[derive(Template)]
//! #[stilts(path = "example.html")]
//! struct MyExample {
//!     value: String,
//! }
//! ```
//!
//! Using said template:
//! ```no_run
//! use stilts::Template;
//!
//! let my_template = MyExample {
//!     value: "Hello, World".to_string(),
//! };
//!
//! let s: String = my_template.render().unwrap();
//! ```
//!
//! ## Configuration
//! Configuration in stilts is done within your projects Cargo.toml this was done to reduce
//! the need for seperate configuration file inside your project workspace. You can set the
//! directory stilts searches for templates from, whether it will trim whitespace, the identifier
//! of the writer that the generated code will use, and add/override custom escape formats. There
//! is more in depth documentation [here](https://atrociously.github.io/stilts/configuration.html) in the book.
//!
//! ### Example
//! These are the default values for all current configuration options
//! ```toml
//! [package.metadata.stilts]
//! template_dir = "$CARGO_MANIFEST_DIR/templates"
//! trim = false
//! writer_name = "_w"
//! escape = { html = "::stilts::escaping::Html", "htm" = "::stilts::escaping::Html" }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::Write;

pub use extensions::{DebugExt, DisplayExt, SerializeExt};

/// Create a stilts template
///
/// The macro will derive an implementation of the [`Template`](trait@Template) trait
/// that executes the code within the template and renders it properly.
/// 
/// ## Attribute options
/// - **path** (required): The path relative to the template root of the template to render
/// - **escape**: Override the escaper detected by file extension with a specified one
///
/// ## Examples:
/// Standard use case
/// ```no_run
/// #[derive(Template)]
/// #[stilts(path = "index.html")]
/// struct MyTemplate {
///     my_data: String,
/// }
/// ```
/// 
/// An example of setting the escape to something else
/// ```no_run
/// #[derive(Template)]
/// #[stilts(path = "index.html", escape = ::stilts::escaping::Empty)]
/// struct MyOverridenTemplate {
///     my_data: String,
/// }
/// ```
pub use stilts_macros::Template;

pub mod escaping;
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
