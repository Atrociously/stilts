use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn expand_integration(
    ident: &Ident,
    impl_gen: &ImplGenerics,
    type_gen: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> TokenStream {
    quote::quote! {
        impl #impl_gen ::warp::Reply for #ident #type_gen #where_clause {
            fn into_response(self) -> ::warp::reply::Response {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        if let Some(mime) = self.mime_str() {
                            let reply = ::warp::reply::with_header(content, "Content-Type", mime);
                            ::warp::reply::Reply::into_response(reply)
                        } else {
                            ::warp::reply::Reply::into_response(content)
                        }
                    }
                    Err(_) => ::warp::reply::Reply::into_response(::warp::http::StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
        }
    }
}
