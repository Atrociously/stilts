//! The derive macro that generates the rust code for the templates

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use parse::TemplateInput;
use proc_macro::TokenStream;

mod config;
mod expand;
mod parse;
mod pathing;

pub(crate) const ATTR_NAME: &str = "stilts";

macro_rules! err {
    ($span:expr, $msg:expr) => {
        syn::Error::new(syn::spanned::Spanned::span(&$span), $msg)
    };
    ($msg:expr) => {
        syn::Error::new(proc_macro2::Span::call_site(), $msg)
    };
}
pub(crate) use err;

macro_rules! abort {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return e.to_compile_error().into(),
        }
    };
}
pub(crate) use abort;

use quote::quote;
use syn::DeriveInput;

/// Create a stilts template
///
/// Examples:
/// ```rust
/// #[derive(Template)]
/// #[stilts(path = "index.html")]
/// struct MyTemplate {
///     my_data: String,
/// }
/// ```
///
/// contents of index.html
/// ```
/// <button>{% my_data %}</button>
/// ```
#[proc_macro_derive(Template, attributes(stilts))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let input = abort!(TemplateInput::parse(input));

    let default = expand::default_template_impl(&input);
    match expand::template(input) {
        Ok(v) => v.into(),
        Err(err) => {
            let err = err.to_compile_error();
            quote! {
                #err
                #default
            }
            .into()
        }
    }
}
