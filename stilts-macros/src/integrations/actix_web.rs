use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn expand_integration(
    ident: &Ident,
    impl_gen: &ImplGenerics,
    type_gen: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> TokenStream {
    quote::quote! {
        impl #impl_gen ::actix_web::Responder for #ident #type_gen #where_clause {
            type Body = ::actix_web::body::BoxBody;

            fn respond_to(self, req: &::actix_web::HttpRequest) -> ::actix_web::HttpResponse<Self::Body> {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        let mut res = ::actix_web::HttpResponse::Ok();
                        if let Some(mime) = self.mime_str() {
                            res.insert_header((::actix_web::http::header::CONTENT_TYPE, ::actix_web::http::header::HeaderValue::from_static(mime)));
                        }
                        res.body(content)
                    }
                    Err(_) => ::actix_web::HttpResponse::InternalServerError().finish(),
                }
            }
        }
    }
}
