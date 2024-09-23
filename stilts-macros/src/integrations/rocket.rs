use proc_macro2::{Ident, Span};
use quote::quote;
use syn::Generics;

pub fn expand_integration(
    ident: &Ident,
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let type_gen = generics.split_for_impl().1;
    let mut generics = generics.clone();
    let req_lifetime = syn::Lifetime::new("'rocket_request", Span::call_site());
    let res_lifetime = syn::Lifetime::new("'rocket_response", Span::call_site());

    let mut res_lifetime_param = syn::LifetimeParam::new(res_lifetime);

    res_lifetime_param.colon_token = Some(syn::Token![:](Span::call_site()));
    res_lifetime_param.bounds = syn::punctuated::Punctuated::new();
    res_lifetime_param.bounds.push(req_lifetime.clone());
    generics.params.push(syn::GenericParam::Lifetime(syn::LifetimeParam::new(req_lifetime)));
    generics.params.push(syn::GenericParam::Lifetime(res_lifetime_param));

    let (impl_gen, _, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::rocket::response::Responder<'rocket_request, 'rocket_response> for #ident #type_gen #where_clause {
            fn respond_to(self, request: &'rocket_request ::rocket::request::Request<'_>) -> ::rocket::response::Result<'rocket_response> {
                match ::stilts::Template::render(&self) {
                    Ok(body) => {
                        let mut res = ::rocket::response::Response::new();
                        let mime = self.mime_str()
                            .and_then(::rocket::http::ContentType::parse_flexible);
                        if let Some(content_type) = mime {
                            res.set_header(content_type);
                        }
                        res.set_sized_body(body.len(), ::std::io::Cursor::new(body));
                        Ok(res)
                    }
                    Err(_) => {
                        Err(::rocket::http::Status::InternalServerError)
                    }
                }
            }
        }
    }
}
