use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn expand_integration(
    ident: &Ident,
    impl_gen: &ImplGenerics,
    type_gen: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> TokenStream {
    quote::quote! {
        impl #impl_gen ::gotham::handler::IntoResponse for #ident #type_gen #where_clause {
            fn into_response(self, state: &::gotham::state::State) -> ::gotham::hyper::http::Response<::gotham::hyper::body::Body> {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        let mut res = ::gotham::hyper::Response::builder()
                            .status(200);
                        if let Some(mime) = self.mime_str() {
                            res = res.header(::gotham::hyper::http::header::CONTENT_TYPE, ::gotham::hyper::http::HeaderValue::from_static(mime));
                        }
                        res.body(content.into())
                            .unwrap()
                    }
                    Err(_) => ::gotham::hyper::Response::builder().status(500).body(::gotham::hyper::body::Body::empty().into()).unwrap(),
                }
            }
        }
    }
}
