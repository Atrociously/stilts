use proc_macro2::TokenStream;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn expand_integration(path: &str, ident: &Ident, impl_gen: &ImplGenerics, type_gen: &TypeGenerics, where_clause: &Option<&WhereClause>) -> TokenStream {
    let mime = mime_guess::from_path(path).first_or_text_plain().to_string();
    quote::quote! {
        impl #impl_gen ::actix_web::Responder for #ident #type_gen #where_clause {
            type Body = ::actix_web::body::BoxBody;
            
            fn respond_to(self, req: &::actix_web::HttpRequest) -> ::actix_web::HttpResponse<Self::Body> {
                match ::stilts::Template::render(&self) {
                    Ok(content) => {
                        ::actix_web::HttpResponse::Ok()
                            .insert_header(("Content-Type", #mime))
                            .body(content)
                    }
                    Err(_) => ::actix_web::HttpResponse::InternalServerError().finish(),
                }
            }
        } 
    }
}
