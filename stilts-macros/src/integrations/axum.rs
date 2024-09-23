use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn expand_integration(
    ident: &Ident,
    impl_gen: &ImplGenerics,
    type_gen: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> TokenStream {
    quote::quote! {
        impl #impl_gen ::axum::response::IntoResponse for #ident #type_gen #where_clause {
            fn into_response(self) -> ::axum::response::Response {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        let mut res = ::axum::response::Response::builder()
                            .status(200);
                        if let Some(mime) = self.mime_str() {
                            res = res.header(::axum::http::header::CONTENT_TYPE, ::axum::http::HeaderValue::from_static(mime));
                        }
                        res.body(::axum::body::Body::new(content))
                            .unwrap()
                    }
                    Err(_) => ::axum::response::IntoResponse::into_response(::axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
        }
    }
}
