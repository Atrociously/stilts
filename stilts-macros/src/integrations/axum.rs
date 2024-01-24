use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

use crate::parse::TemplateSource;

pub fn expand_integration(
    source: &TemplateSource,
    ident: &Ident,
    impl_gen: &ImplGenerics,
    type_gen: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> TokenStream {
    let mime = mime_guess::from_path(source.as_path().unwrap_or_default())
        .first_or_text_plain()
        .to_string();
    quote::quote! {
        impl #impl_gen ::axum::response::IntoResponse for #ident #type_gen #where_clause {
            fn into_response(self) -> ::axum::response::Response {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        ::axum::response::Response::builder()
                            .status(200)
                            .header(::axum::http::header::CONTENT_TYPE, ::axum::http::HeaderValue::from_static(#mime))
                            .body(::axum::body::Body::new(content))
                            .unwrap()
                    }
                    Err(_) => ::axum::response::IntoResponse::into_response(::axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
        }
    }
}
