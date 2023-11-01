use std::collections::HashMap;

use cargo_metadata::camino::Utf8PathBuf;
use proc_macro2::TokenStream;
use quote::quote;
use stilts_lang::{BlockExpr, BlockItem, Expr, ExtendsExpr, IfClose, Item, MatchArm, Root};

use crate::config::Config;
use crate::err;
use crate::parse::{TemplateInput, TemplateAttrs};

fn format_err(e: stilts_lang::Error) -> syn::Error {
    #[cfg(not(any(feature = "narratable", feature = "fancy")))]
    return err!(e.display_simple());

    #[allow(unused_variables)]
    #[cfg(feature = "narratable")]
    let handler = miette::NarratableReportHandler::new();
    #[cfg(feature = "fancy")]
    let handler = miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode());
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

        let (path, content) = read_template(cfg, &attrs.path.value())?;
        let root = Root::parse(&content)
            .map(Root::into_owned)
            .map_err(format_err)?;

        let mut parent = Self::get_parent(&root);
        let blocks = Self::get_blocks(root.content.iter());
        let node = TemplateNode {
            path,
            blocks,
            root,
            escape_override: attrs.escape.clone(),
        };

        graph.push(node);

        while let Some(p) = parent {
            let (path, content) = read_template(cfg, &p.reference.value())?;

            // check for graph cycle
            if graph.iter().any(|t| t.path == path) {
                let mut cycle_str =
                    graph
                        .iter()
                        .map(|t| &t.path)
                        .fold(String::new(), |mut acc, p| {
                            acc.push_str(&format!("{} -> ", relative_to(cfg, p)));
                            acc
                        });
                cycle_str.push_str(&relative_to(cfg, &path));
                return Err(err!(format!("dependency cycle detected: {cycle_str}")));
            }

            let root = Root::parse(&content)
                .map(Root::into_owned)
                .map_err(format_err)?;
            let blocks = Self::get_blocks(&root.content);

            let node = TemplateNode {
                path,
                blocks,
                root,
                escape_override: attrs.escape.clone(),
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
            let path = t.path.as_str();
            // this should ensure that the compiler knows that the code is dependent on this file
            toks.extend(quote! { ::core::include_bytes!(#path); });
            toks.extend(t.expand(cfg, &expanded_blocks)?);
            expanded_blocks.extend(t.blocks.keys().cloned());
            cur = t.child();
        }
        Ok(toks)
    }

    fn get_parent(root: &Root) -> Option<ExtendsExpr> {
        match root.content.get(0) {
            Some(Item::Expr(Expr::Extends(e))) => Some(e.clone()),
            _ => None,
        }
    }

    // get all blocks including sub-blocks
    fn get_blocks<'a>(
        content: impl IntoIterator<Item = &'a Item<'static>>,
    ) -> HashMap<syn::Ident, BlockExpr<'static>> {
        content
            .into_iter()
            .filter_map(|i| match i {
                Item::Expr(Expr::Block(b)) => Some(b),
                _ => None,
            })
            .flat_map(|b| {
                // get the blocks within this block
                let content: Vec<_> = b
                    .content
                    .iter()
                    .filter_map(|i| match i {
                        BlockItem::Item(i) if matches!(i, Item::Expr(Expr::Block(..))) => Some(i),
                        _ => None,
                    })
                    .collect();
                if !content.is_empty() {
                    let mut m = Self::get_blocks(content);
                    m.insert(b.name.clone(), b.clone());
                    m
                } else {
                    HashMap::from([(b.name.clone(), b.clone())])
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
    fn deepest_child(self, block_name: &'a syn::Ident) -> TemplateRef {
        let mut cur = self;
        let mut next = self.child();
        while let Some(t) = next {
            if t.blocks.get(block_name).is_some() {
                cur = t;
            }
            next = next.unwrap().child();
        }
        cur
    }

    // expand a block directly (does not lookup the deepest child to expand)
    fn expand_block_inner(self, cfg: &Config, block: &BlockExpr) -> syn::Result<TokenStream> {
        block
            .content
            .iter()
            .map(|bi| match bi {
                BlockItem::Item(i) => self.expand_item(cfg, i),
                BlockItem::SuperCall => {
                    let parent = self.parent();
                    let pblock = parent.as_ref().and_then(|p| p.blocks.get(&block.name));
                    if let Some((parent, pblock)) = parent.zip(pblock) {
                        parent.expand_block_inner(cfg, pblock)
                    } else {
                        Ok(quote! {})
                    }
                }
            })
            .collect()
    }

    // expand a block by navigating to the deepest child and
    // then using `expand_block_inner`
    fn expand_block(self, cfg: &Config, block: &BlockExpr) -> syn::Result<TokenStream> {
        let deepest = self.deepest_child(&block.name);
        let block = deepest.blocks.get(&block.name).unwrap();
        deepest.expand_block_inner(cfg, block)
    }

    // expand the closing statement of an if expression
    fn expand_if_close(self, cfg: &Config, close: &IfClose) -> syn::Result<TokenStream> {
        match close {
            IfClose::Close => Ok(quote! {}),
            IfClose::Else { then: items, .. } => {
                let items: TokenStream = items
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                Ok(quote! { else { #items } })
            }
            IfClose::ElseIf { cond, then, close } => {
                let items: TokenStream = then
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                let close = self.expand_if_close(cfg, close)?;
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
        let escaper = self.escape_override.clone()
            .unwrap_or_else(|| cfg.escaper(self.path.extension().unwrap_or_default()));
        match item {
            Item::Content(c) => {
                let c = match cfg.trim {
                    true => c.as_ref().trim(),
                    false => c.as_ref(),
                };
                if !c.is_empty() && !c.chars().all(char::is_whitespace) {
                    Ok(quote! { #writer.write_str(#c)?; })
                } else {
                    Ok(quote! {})
                }
            }
            Item::Expr(Expr::Expr(expr)) => Ok(quote! { ::core::write!(#writer, "{}", ::stilts::escaping::Escaped::new(&#expr, #escaper))?; }),
            Item::Expr(Expr::Stmt(stmt)) => Ok(quote! { #stmt }),
            Item::Expr(Expr::For(for_expr)) => {
                let label = &for_expr.label;
                let pat = &for_expr.pat;
                let expr = &for_expr.expr;
                let items: TokenStream = for_expr
                    .content
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                Ok(quote! {
                    #label for #pat in #expr {
                        #items
                    }
                })
            }
            Item::Expr(Expr::If(if_expr)) => {
                let cond = &if_expr.cond;
                let items: TokenStream = if_expr
                    .then
                    .iter()
                    .map(|i| self.expand_item(cfg, i))
                    .collect::<Result<_, _>>()?;
                let close = self.expand_if_close(cfg, &if_expr.close)?;
                Ok(quote! {
                    if #cond {
                        #items
                    } #close
                })
            }
            Item::Expr(Expr::Match(match_expr)) => {
                let expr = &match_expr.expr;
                let arms: TokenStream = match_expr
                    .arms
                    .iter()
                    .map(|i| self.expand_match_arm(cfg, i))
                    .collect::<Result<_, _>>()?;
                Ok(quote! {
                    match #expr {
                        #arms
                    }
                })
            }
            Item::Expr(Expr::Macro(macro_expr)) => {
                let writer = &cfg.writer_name;
                let writer_ty = quote! { &mut (impl ::core::fmt::Write + ?::core::marker::Sized) };
                let name = &macro_expr.name;
                let args = &macro_expr.args;
                let content = macro_expr
                    .content
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
            Item::Expr(Expr::CallMacro(call_expr)) => {
                let writer = &cfg.writer_name;
                let name = &call_expr.name;
                let args = &call_expr.args;
                Ok(quote! {
                    #name(#writer, #args)?;
                })
            }
            Item::Expr(Expr::Include(include_expr)) => {
                let attrs = TemplateAttrs {
                    path: syn::LitStr::new(&include_expr.includes.value(), proc_macro2::Span::call_site()),
                    escape: self.escape_override.clone(),
                };
                let graph = Graph::load(cfg, &attrs)?;
                graph.expand(cfg)
            }
            Item::Expr(Expr::Block(block_expr)) => self.expand_block(cfg, block_expr),
            Item::Expr(Expr::Extends(_)) => Ok(quote! {}),
        }
    }

    // expand the whole template
    fn expand(self, cfg: &Config, prev: &[syn::Ident]) -> syn::Result<TokenStream> {
        self.root
            .content
            .iter()
            .filter(|i| match i {
                Item::Expr(Expr::Block(b)) => !prev.contains(&b.name),
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
    path: Utf8PathBuf,
    escape_override: Option<syn::Path>,
    blocks: HashMap<syn::Ident, BlockExpr<'static>>,
    root: Root<'static>,
}

fn relative_to(config: &Config, path: &Utf8PathBuf) -> String {
    path.as_str().replace(config.template_dir.as_str(), "")
}

fn read_template(config: &Config, file: &str) -> syn::Result<(Utf8PathBuf, String)> {
    let path = config.template_dir.join(file);
    std::fs::read_to_string(&path)
        .map_err(|io| err!(format!("{io} while reading {}", path.clone())))
        .map(|c| (path, c))
}

pub fn template(input: TemplateInput) -> syn::Result<TokenStream> {
    let TemplateInput {
        ident,
        generics,
        fields,
        attrs,
    } = &input;

    let config = Config::load().map_err(|e| err!(e))?;

    let graph = Graph::load(&config, attrs)?;
    let template_code = graph.expand(&config)?;

    let writer = &config.writer_name;
    let field_idents = fields.iter().map(|f| &f.ident);

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    #[allow(unused_mut)]
    let mut integrations = TokenStream::new();
    #[cfg(feature = "actix-web")]
    integrations.extend(crate::integrations::actix_web::expand_integration(&attrs.path.value(), ident, &impl_gen, &type_gen, &where_clause));
    #[cfg(feature = "axum")]
    integrations.extend(crate::integrations::axum::expand_integration(&attrs.path.value(), ident, &impl_gen, &type_gen, &where_clause));
    #[cfg(feature = "gotham")]
    integrations.extend(crate::integrations::gotham::expand_integration(&attrs.path.value(), ident, &impl_gen, &type_gen, &where_clause));

    Ok(quote! {
        #integrations
        impl #impl_gen ::stilts::Template for #ident #type_gen #where_clause {
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
            fn fmt(&self, _w: &mut (impl ::core::fmt::Write + ?::core::marker::Sized)) -> ::core::fmt::Result {
                ::core::unimplemented!("compilation error within template")
            }
        }
    }
}
