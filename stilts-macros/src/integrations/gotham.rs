use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn expand_integration(path: &str, ident: &Ident, impl_gen: &ImplGenerics, type_gen: &TypeGenerics, where_clause: &Option<&WhereClause>) -> TokenStream {
    let mime = mime_guess::from_path(path).first_or_text_plain().to_string();
    quote::quote! {
        impl #impl_gen ::gotham::handler::IntoResponse for #ident #type_gen #where_clause {
            fn into_response(self, state: &::gotham::state::State) -> ::gotham::hyper::http::Response<::gotham::hyper::body::Body> {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        ::gotham::hyper::http::Response::builder()
                            .status(200)
                            .header(::gotham::hyper::http::header::CONTENT_TYPE, ::gotham::hyper::http::HeaderValue::from_static(#mime))
                            .body(content.into())
                            .unwrap()
                    }
                    Err(_) => ::gotham::hyper::http::Response::builder().status(500).body(::gotham::hyper::body::Body::empty().into()).unwrap(),
                }
            }
        } 
    }
}
