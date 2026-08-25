use proc_macro2::TokenStream;
use quote::quote;
use stilts_lang::{StiltsSpec, TemplateGraph, TemplateOptions, TemplateRef};
use stilts_parser::types::{
    Expr, IfBranch, Item, ItemBlock, ItemFor, ItemIf, ItemMacro, ItemMatch, MatchArm,
};

use crate::err;
use crate::parse::TemplateInput;

fn format_err(e: stilts_lang::Error) -> syn::Error {
    let e = match e {
        stilts_lang::Error::Parse(err) => err,
        err => return err!(err),
    };
    #[cfg(not(any(feature = "narratable", feature = "fancy")))]
    return err!(e.display_simple());

    #[allow(unused_variables)]
    #[cfg(feature = "narratable")]
    let handler = miette::NarratableReportHandler::new();
    #[cfg(feature = "fancy")]
    let handler =
        miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
    #[cfg(any(feature = "narratable", feature = "fancy"))]
    {
        let mut s = String::new();
        let s = handler
            .render_report(&mut s, e.as_ref())
            .map(|_| s)
            .unwrap_or(e.display_simple());
        err!(s)
    }
}

fn expand_graph(config: stilts_lang::Config, opts: TemplateOptions) -> syn::Result<TokenStream> {
    let fs = stilts_lang::StdFs;
    let mut graph = stilts_lang::Heirarchy::new(fs, config.clone());
    let root = graph.load(opts).map_err(format_err)?;

    let mut expanded_blocks = Vec::new();
    let mut toks = TokenStream::new();
    let mut cur = root.parentage().last().or(Some(root)); // get topmost parent or else the root
    while let Some(t) = cur {
        if let Some(path) = t.node().data.source_path.as_ref().map(|p| p.as_str()) {
            toks.extend(quote! { ::core::include_bytes!(#path); });
        }
        toks.extend(expand_node(&config, &t, &expanded_blocks)?);
        expanded_blocks.extend(t.node().blocks.keys().map(|s| s.clone().into()));
        cur = t.children().next();
    }
    Ok(toks)
}

fn expand_block_inner(
    cfg: &stilts_lang::Config,
    node: &(impl TemplateRef<StiltsSpec> + Clone),
    block: &ItemBlock,
) -> syn::Result<TokenStream> {
    block
        .content
        .iter()
        .map(|bi| match bi {
            Item::Expr(Expr::SuperCall) => {
                let parent = node.parent();
                let pblock = parent
                    .as_ref()
                    .and_then(|p| p.node().blocks.get(block.name.as_ref()))
                    .cloned();
                if let Some((parent, pblock)) = parent.zip(pblock) {
                    expand_block_inner(cfg, &parent, &pblock)
                } else {
                    Ok(quote! {})
                }
            }
            item => expand_item(cfg, node, &item),
        })
        .collect()
}

fn expand_block(
    cfg: &stilts_lang::Config,
    node: &(impl TemplateRef<StiltsSpec> + Clone),
    block: &ItemBlock,
) -> syn::Result<TokenStream> {
    let deepest = deepest_child(node, &block.name);
    let block = deepest.node().blocks.get(block.name.as_ref()).unwrap();
    expand_block_inner(cfg, &deepest, block)
}

fn expand_if_branch(
    cfg: &stilts_lang::Config,
    node: &(impl TemplateRef<StiltsSpec> + Clone),
    close: &IfBranch,
) -> syn::Result<TokenStream> {
    match close {
        IfBranch::End => Ok(quote! {}),
        IfBranch::Else { content, .. } => {
            let items: TokenStream = content
                .iter()
                .map(|i| expand_item(cfg, node, i))
                .collect::<Result<_, _>>()?;
            Ok(quote! { else { #items } })
        }
        IfBranch::ElseIf {
            cond,
            content,
            branch,
        } => {
            let items: TokenStream = content
                .iter()
                .map(|i| expand_item(cfg, node, i))
                .collect::<Result<_, _>>()?;
            let close = expand_if_branch(cfg, node, branch)?;
            Ok(quote! {
                else if #cond {
                    #items
                } #close
            })
        }
    }
}

fn expand_match_arm(
    cfg: &stilts_lang::Config,
    node: &(impl TemplateRef<StiltsSpec> + Clone),
    arm: &MatchArm,
) -> syn::Result<TokenStream> {
    let pat = &arm.pat;
    let guard = arm.guard.as_ref().map(|g| quote! { if #g });
    let items: TokenStream = arm
        .content
        .iter()
        .map(|i| expand_item(cfg, node, i))
        .collect::<Result<_, _>>()?;
    Ok(quote! { #pat #guard => { #items } })
}

fn expand_item(
    cfg: &stilts_lang::Config,
    node: &(impl TemplateRef<StiltsSpec> + Clone),
    item: &Item,
) -> syn::Result<TokenStream> {
    let writer = &cfg.writer_name;
    let escaper = node.escape_implementation();
    match item {
        Item::Content(c) => {
            let c = match node.trim() {
                true => c.as_ref().trim(),
                false => c.as_ref(),
            };
            if !c.is_empty() && !c.chars().all(char::is_whitespace) {
                Ok(quote! { #writer.write_str(#c)?; })
            } else {
                Ok(quote! {})
            }
        }
        Item::Block(block_item) => expand_block(cfg, node, block_item),
        Item::For(ItemFor {
            label,
            pat,
            expr,
            content,
        }) => {
            let content: TokenStream = content
                .iter()
                .map(|i| expand_item(cfg, node, i))
                .collect::<Result<_, _>>()?;
            Ok(quote! {
                #label for #pat in #expr {
                    #content
                }
            })
        }
        Item::If(ItemIf {
            cond,
            content,
            branch,
        }) => {
            let items: TokenStream = content
                .iter()
                .map(|i| expand_item(cfg, node, i))
                .collect::<Result<_, _>>()?;
            let branch = expand_if_branch(cfg, node, branch)?;
            Ok(quote! {
                if #cond {
                    #items
                } #branch
            })
        }
        Item::Match(ItemMatch { expr, arms }) => {
            let arms: TokenStream = arms
                .iter()
                .map(|i| expand_match_arm(cfg, node, i))
                .collect::<Result<_, _>>()?;
            Ok(quote! {
                match #expr {
                    #arms
                }
            })
        }
        Item::Macro(ItemMacro {
            name,
            args,
            content,
        }) => {
            let writer = &cfg.writer_name;
            let writer_ty = quote! { &mut (impl ::core::fmt::Write + ?::core::marker::Sized) };
            let content = content
                .iter()
                .map(|i| expand_item(cfg, node, i))
                .collect::<Result<TokenStream, _>>()?;
            Ok(quote! {
                fn #name(#writer: #writer_ty, #args) -> ::core::fmt::Result {
                    #content
                    Ok(())
                };
            })
        }
        Item::Expr(Expr::Extends(_)) => Ok(quote! {}),
        Item::Expr(Expr::SuperCall) => Ok(quote! {}),
        Item::Expr(Expr::Include { reference, args }) => {
            let included = expand_graph(
                cfg.clone(),
                TemplateOptions {
                    source: stilts_lang::TemplateSource::File(reference.into()),
                    overrides: node.node().overrides.clone(),
                    only_block: None,
                },
            )?;
            let arg_assignments = args.into_iter().map(|arg| {
                let syn::FieldValue { member, expr, .. } = arg;
                quote! {let #member = #expr;}
            });
            Ok(quote! {
                {
                    #(#arg_assignments)*
                    #included
                }
            })
        }
        Item::Expr(Expr::MacroCall { name, args }) => {
            let writer = &cfg.writer_name;
            Ok(quote! {
                #name(#writer, #args)?;
            })
        }
        Item::Expr(Expr::Expr(expr)) => Ok(
            quote! { ::core::write!(#writer, "{}", ::stilts::escaping::Escaped::new(&#expr, #escaper))?; },
        ),
        Item::Expr(Expr::Stmt(stmt)) => Ok(quote! { #stmt }),
    }
}

fn expand_node(
    cfg: &stilts_lang::Config,
    node: &(impl TemplateRef<StiltsSpec> + Clone),
    prev: &[std::borrow::Cow<'_, str>],
) -> syn::Result<TokenStream> {
    node.node()
        .root
        .content
        .iter()
        .filter(|i| match i {
            Item::Block(ItemBlock { name, .. }) => !prev.contains(&name),
            _ => true,
        })
        .map(|i| expand_item(cfg, node, i))
        .collect()
}

fn deepest_child<T: TemplateRef<StiltsSpec> + Clone>(node: &T, block_name: impl AsRef<str>) -> T {
    let mut next = node.children().next();
    let mut cur = node.clone();
    while let Some(t) = next {
        if t.node().blocks.contains_key(block_name.as_ref()) {
            cur = t;
            next = cur.children().next();
        } else {
            break;
        }
    }
    cur
}

pub fn template(input: TemplateInput) -> syn::Result<TokenStream> {
    let TemplateInput {
        ident,
        generics,
        fields,
        attrs,
    } = input;

    let config = crate::config::load().map_err(|e| err!(format!("Stilts Config Error: {e}")))?;

    let mime_type = attrs
        .source
        .map_path(|p| mime_guess::from_path(p))
        .and_then(|g| g.first())
        .map(|m| m.to_string());
    let mime_type = match mime_type {
        Some(mt) => quote! { Some(#mt) },
        None => quote! { None },
    };

    //let graph = Graph::load(&config, attrs)?;
    //let template_code = graph.expand(&config)?;
    let writer = config.writer_name.clone();
    let template_code = expand_graph(config, attrs.into_options())?;

    let field_idents = fields.iter().map(|f| &f.ident);

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    #[allow(unused_mut)]
    let mut integrations = TokenStream::new();
    #[cfg(feature = "actix-web")]
    integrations.extend(crate::integrations::actix_web::expand_integration(
        ident,
        &impl_gen,
        &type_gen,
        &where_clause,
    ));
    #[cfg(feature = "axum")]
    integrations.extend(crate::integrations::axum::expand_integration(
        ident,
        &impl_gen,
        &type_gen,
        &where_clause,
    ));
    #[cfg(feature = "gotham")]
    integrations.extend(crate::integrations::gotham::expand_integration(
        ident,
        &impl_gen,
        &type_gen,
        &where_clause,
    ));
    #[cfg(feature = "rocket")]
    integrations.extend(crate::integrations::rocket::expand_integration(
        ident, generics,
    ));
    #[cfg(feature = "warp")]
    integrations.extend(crate::integrations::warp::expand_integration(
        ident,
        &impl_gen,
        &type_gen,
        &where_clause,
    ));

    Ok(quote! {
        #integrations
        impl #impl_gen ::stilts::Template for #ident #type_gen #where_clause {
            fn mime_str(&self) -> ::core::option::Option<&'static str> {
                #mime_type
            }

            fn fmt(&self, #writer: &mut (impl ::core::fmt::Write + ?::core::marker::Sized)) -> ::core::fmt::Result {
                use ::stilts::SerializeExt as _;
                use ::stilts::DisplayExt as _;
                use ::stilts::DebugExt as _;
                let Self {
                    #(#field_idents,)*
                    ..
                } = self;

                #template_code
                Ok(())
            }
        }
    })
}

pub fn default_template_impl(input: &TemplateInput) -> TokenStream {
    let TemplateInput {
        ident, generics, ..
    } = input;

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::stilts::Template for #ident #type_gen #where_clause {
            fn mime_str(&self) -> ::core::option::Option<&'static str> { None }

            fn fmt(&self, _w: &mut (impl ::core::fmt::Write + ?::core::marker::Sized)) -> ::core::fmt::Result {
                ::core::unimplemented!("compilation error within template")
            }
        }
    }
}
