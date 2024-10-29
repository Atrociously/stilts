use std::collections::HashMap;

use cargo_metadata::camino::Utf8PathBuf;
use proc_macro2::TokenStream;
use quote::quote;
use stilts_lang::parse_template;
use stilts_lang::types::{Expr, IfBranch, Item, ItemBlock, ItemFor, ItemIf, ItemMacro, ItemMatch, MatchArm, Root};

use crate::config::Config;
use crate::err;
use crate::parse::{TemplateAttrs, TemplateInput, TemplateSource};

fn format_err(e: stilts_lang::Error) -> syn::Error {
    #[cfg(not(any(feature = "narratable", feature = "fancy")))]
    return err!(e.display_simple());

    #[allow(unused_variables)]
    #[cfg(feature = "narratable")]
    let handler = miette::NarratableReportHandler::new();
    #[cfg(feature = "fancy")]
    let handler = miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
    #[cfg(any(feature = "narratable", feature = "fancy"))]
    {
        let mut s = String::new();
        let s = handler
            .render_report(&mut s, &e)
            .map(|_| s)
            .unwrap_or(e.display_simple());
        err!(s)
    }
}

#[derive(Debug)]
struct Graph(Vec<TemplateNode>);

impl Graph {
    pub fn load(cfg: &Config, attrs: &TemplateAttrs) -> syn::Result<Self> {
        let mut graph = Vec::with_capacity(1);

        let data = read_template(cfg, &attrs.source)?;
        let root = parse_template(&data.content, cfg.delimiters.clone())
            .map(Root::into_owned)
            .map_err(format_err)?;

        let mut parent = Self::get_parent(&root);
        let blocks = Self::get_blocks(root.content.iter());
        let node = TemplateNode {
            data,
            blocks,
            root,
            escape_override: attrs.escape.clone(),
            trim_override: attrs.trim,
        };

        graph.push(node);

        fn check_dependency_cycle(
            cfg: &Config,
            graph: &[TemplateNode],
            path: &Utf8PathBuf,
        ) -> Result<(), syn::Error> {
            // check for graph cycle
            if graph.iter().any(|t| t.data.path.as_ref() == Some(path)) {
                let mut cycle_str = graph.iter().filter_map(|t| t.data.path.as_ref()).fold(
                    String::new(),
                    |mut acc, p| {
                        acc.push_str(&format!("{} -> ", relative_to(cfg, p)));
                        acc
                    },
                );
                cycle_str.push_str(&relative_to(cfg, path));
                Err(err!(format!("dependency cycle detected: {cycle_str}")))
            } else {
                Ok(())
            }
        }

        while let Some(p) = parent {
            let data = read_template(cfg, &TemplateSource::new_file(p))?;

            if let Some(path) = &data.path {
                check_dependency_cycle(cfg, &graph, path)?;
            }

            let root = parse_template(&data.content, cfg.delimiters.clone())
                .map(Root::into_owned)
                .map_err(format_err)?;
            let blocks = Self::get_blocks(&root.content);

            let node = TemplateNode {
                data,
                blocks,
                root,
                escape_override: attrs.escape.clone(),
                trim_override: attrs.trim,
            };
            parent = Self::get_parent(&node.root);
            graph.push(node);
        }

        graph.reverse();
        Ok(Self(graph))
    }

    // descend the template inheritance list rendering all of them sequentially
    // while performing necessary expansions
    pub fn expand(self, cfg: &Config) -> syn::Result<TokenStream> {
        let mut expanded_blocks = Vec::new();
        let mut toks = TokenStream::new();
        let mut cur = Some(TemplateRef(0, &self.0));
        while let Some(t) = cur {
            if let Some(path) = t.data.path.as_ref().map(|p| p.as_str()) {
                // this should ensure that the compiler knows that the code is dependent on this file
                toks.extend(quote! { ::core::include_bytes!(#path); });
            }
            toks.extend(t.expand(cfg, &expanded_blocks)?);
            expanded_blocks.extend(t.blocks.keys().map(|s| s.clone().into()));
            cur = t.child();
        }
        Ok(toks)
    }

    fn get_parent<'a>(root: &Root<'a>) -> Option<std::borrow::Cow<'a, str>> {
        match root.content.first() {
            Some(Item::Expr(Expr::Extends(e))) => Some(e.clone()),
            _ => None,
        }
    }

    // get all blocks including sub-blocks
    fn get_blocks<'a>(
        content: impl IntoIterator<Item = &'a Item<'static>>,
    ) -> HashMap<String, ItemBlock<'static>> {
        content
            .into_iter()
            .filter_map(|i| match i {
                Item::Block(b) => Some(b),
                _ => None,
            })
            .flat_map(|b| {
                // get the blocks within this block
                let content: Vec<_> = b
                    .content
                    .iter()
                    .filter(|i| matches!(i, Item::Block(..)))
                    /*.filter_map(|i| match i {
                        Item::Block(..) => Some(i),
                        //BlockItem::Item(i) if matches!(i, Item::Expr(Expr::Block(..))) => Some(i),
                        _ => None,
                    })*/
                    .collect();
                if !content.is_empty() {
                    let mut m = Self::get_blocks(content);
                    m.insert(b.name.to_string(), b.clone());
                    m
                } else {
                    HashMap::from([(b.name.to_string(), b.clone())])
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy)]
struct TemplateRef<'a>(usize, &'a [TemplateNode]);

impl<'a> TemplateRef<'a> {
    // get the parent node if one exists
    fn parent(self) -> Option<Self> {
        self.0.checked_sub(1).map(|idx| Self(idx, self.1))
    }

    // get the child node if one exists
    fn child(self) -> Option<Self> {
        let idx = self.0 + 1;
        if idx < self.1.len() {
            Some(Self(idx, self.1))
        } else {
            None
        }
    }

    // get the deepest child that contains the specified block_name
    // checks ALL nodes reguardless of if the direct parent did not contain
    // the block, returns the current template if none is found
    fn deepest_child(self, block_name: impl AsRef<str>) -> Self {
        let mut cur = self;
        let mut next = self.child();
        while let Some(t) = next {
            if t.blocks.contains_key(block_name.as_ref()) {
                cur = t;
            }
            next = next.unwrap().child();
        }
        cur
    }

    // expand a block directly (does not lookup the deepest child to expand)
    fn expand_block_inner(self, cfg: &Config, block: &ItemBlock) -> syn::Result<TokenStream> {
        block
            .content
            .iter()
            .map(|bi| match bi {
                Item::Expr(Expr::SuperCall) => {
                    let parent = self.parent();
                    let pblock = parent.as_ref().and_then(|p| p.blocks.get(block.name.as_ref()));
                    if let Some((parent, pblock)) = parent.zip(pblock) {
                        parent.expand_block_inner(cfg, pblock)
                    } else {
                        Ok(quote! {})
                    }
                },
                item => self.expand_item(cfg, item)
            })
            .collect()
    }

    // expand a block by navigating to the deepest child and
    // then using `expand_block_inner`
    fn expand_block(self, cfg: &Config, block: &ItemBlock) -> syn::Result<TokenStream> {
        let deepest = self.deepest_child(&block.name);
        let block = deepest.blocks.get(block.name.as_ref()).unwrap();
        deepest.expand_block_inner(cfg, block)
    }

    // expand the closing statement of an if expression
    fn expand_if_branch(self, cfg: &Config, close: &IfBranch) -> syn::Result<TokenStream> {
        match close {
            IfBranch::End => Ok(quote! {}),
            IfBranch::Else { content, .. } => {
                let items: TokenStream = content
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                Ok(quote! { else { #items } })
            }
            IfBranch::ElseIf { cond, content, branch } => {
                let items: TokenStream = content
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                let close = self.expand_if_branch(cfg, branch)?;
                Ok(quote! {
                    else if #cond {
                        #items
                    } #close
                })
            }
        }
    }

    // expand an arm of a match expression
    fn expand_match_arm(self, cfg: &Config, arm: &MatchArm) -> syn::Result<TokenStream> {
        let pat = &arm.pat;
        let guard = arm.guard.as_ref().map(|g| quote! { if #g });
        let items: TokenStream = arm
            .content
            .iter()
            .map(|i| self.expand_item(cfg, i))
            .collect::<Result<_, _>>()?;
        Ok(quote! { #pat #guard => { #items } })
    }

    // expand an item which may be one of many different things
    fn expand_item(self, cfg: &Config, item: &Item) -> syn::Result<TokenStream> {
        let writer = &cfg.writer_name;
        let escaper = self.escape_override.clone().unwrap_or_else(|| {
            cfg.escaper(
                self.data
                    .path
                    .as_ref()
                    .and_then(|p| p.extension())
                    .unwrap_or_default(),
            )
        });
        match item {
            Item::Content(c) => {
                let c = match self.trim_override.unwrap_or(cfg.trim) {
                    true => c.as_ref().trim(),
                    false => c.as_ref(),
                };
                if !c.is_empty() && !c.chars().all(char::is_whitespace) {
                    Ok(quote! { #writer.write_str(#c)?; })
                } else {
                    Ok(quote! {})
                }
            }
            Item::Block(block_item) => self.expand_block(cfg, block_item),
            Item::For(ItemFor { label, pat, expr, content }) => {
                let content: TokenStream = content
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                Ok(quote! {
                    #label for #pat in #expr {
                        #content
                    }
                })
            }
            Item::If(ItemIf { cond, content, branch }) => {
                let items: TokenStream = content
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                let branch = self.expand_if_branch(cfg, branch)?;
                Ok(quote! {
                    if #cond {
                        #items
                    } #branch
                })
            }
            Item::Match(ItemMatch { expr, arms }) => {
                let arms: TokenStream = arms
                    .iter()
                    .map(|i| self.expand_match_arm(cfg, i))
                    .collect::<Result<_, _>>()?;
                Ok(quote! {
                    match #expr {
                        #arms
                    }
                })
            }
            Item::Macro(ItemMacro { name, args, content }) => {
                let writer = &cfg.writer_name;
                let writer_ty = quote! { &mut (impl ::core::fmt::Write + ?::core::marker::Sized) };
                let content = content
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
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
                let attrs = TemplateAttrs {
                    source: TemplateSource::new_file(reference),
                    escape: self.escape_override.clone(),
                    trim: self.trim_override,
                };
                let graph = Graph::load(cfg, &attrs)?;
                let included = graph.expand(cfg)?;
                let arg_assignments = args.into_iter()
                    .map(|arg| {
                        let syn::FieldValue { member, expr, .. } = arg;
                        quote!{let #member = #expr;}
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
            },
            Item::Expr(Expr::Expr(expr)) => Ok(
                quote! { ::core::write!(#writer, "{}", ::stilts::escaping::Escaped::new(&#expr, #escaper))?; },
            ),
            Item::Expr(Expr::Stmt(stmt)) => Ok(quote! { #stmt }),
        }
    }

    // expand the whole template
    fn expand(self, cfg: &Config, prev: &[std::borrow::Cow<'_, str>]) -> syn::Result<TokenStream> {
        self.root
            .content
            .iter()
            .filter(|i| match i {
                Item::Block(ItemBlock { name, .. }) => !prev.contains(name),
                _ => true,
            })
            .map(|i| self.expand_item(cfg, i))
            .collect()
    }
}

impl<'a> std::ops::Deref for TemplateRef<'a> {
    type Target = TemplateNode;

    fn deref(&self) -> &Self::Target {
        &self.1[self.0]
    }
}

#[derive(Debug)]
struct TemplateNode {
    data: TemplateData,
    escape_override: Option<syn::Path>,
    trim_override: Option<bool>,
    blocks: HashMap<String, ItemBlock<'static>>,
    root: Root<'static>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TemplateData {
    path: Option<Utf8PathBuf>,
    content: String,
}

fn relative_to(config: &Config, path: &Utf8PathBuf) -> String {
    path.as_str().replace(config.template_dir.as_str(), "")
}

fn read_template(config: &Config, source: &TemplateSource) -> syn::Result<TemplateData> {
    match source {
        TemplateSource::File(path) => {
            let path = config.template_dir.join(path.value());
            Ok(TemplateData {
                content: std::fs::read_to_string(&path)
                    .map_err(|io| err!(format!("{io} while reading {}", path.clone())))?,
                path: Some(path),
            })
        }
        TemplateSource::Literal(content) => Ok(TemplateData {
            path: None,
            content: content.value(),
        }),
    }
}

pub fn template(input: TemplateInput) -> syn::Result<TokenStream> {
    let TemplateInput {
        ident,
        generics,
        fields,
        attrs,
    } = &input;

    let config = Config::load().map_err(|e| err!(format!("Stilts Config Error: {e}")))?;

    let mime_type = attrs.source.mime_type()
        .map(|m| m.to_string());
    let mime_type = match mime_type {
        Some(mt) => quote! { Some(#mt) },
        None => quote! { None }
    };

    let graph = Graph::load(&config, attrs)?;
    let template_code = graph.expand(&config)?;

    let writer = &config.writer_name;
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
        ident,
        generics,
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
                    #(#field_idents),*
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
