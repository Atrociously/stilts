use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn expand_integration(path: &str, ident: &Ident, impl_gen: &ImplGenerics, type_gen: &TypeGenerics, where_clause: &Option<&WhereClause>) -> TokenStream {
    let mime = mime_guess::from_path(path).first_or_text_plain().to_string();
    quote::quote! {
        impl #impl_gen ::axum::response::IntoResponse for #ident #type_gen #where_clause {
            fn into_response(self) -> ::axum::response::Response {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        ::axum::response::Response::builder()
                            .status(200)
                            .header(::axum::http::header::CONTENT_TYPE, ::axum::http::HeaderValue::from_static(#mime))
                            .body(::axum::body::boxed(content))
                            .unwrap()
                    }
                    Err(_) => ::axum::response::IntoResponse::into_response(::axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
        } 
    }
}
