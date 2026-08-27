use std::{borrow::Cow, collections::HashMap};

use stilts_parser::types::{Expr, Item, ItemBlock, Root};

use crate::{
    FileSystem, TemplateConfig, TemplateGraph, TemplateNode, TemplateOptions, TemplateSource,
    error::Error,
};

pub fn load<'a, G: TemplateGraph, FS: FileSystem>(
    graph: &'a mut G,
    fs: &mut FS,
    cfg: &TemplateConfig,
    opts: TemplateOptions,
) -> Result<G::Ref<'a>, Error<G::Ref<'a>>> {
    let mut node = TemplateNode::load(fs, cfg, opts.clone())?;
    let mut parent = get_parent(&node.root);

    // isolate a single block when specified
    if let Some(block) = &opts.block {
        let Some(block) = node.blocks.remove(block) else {
            todo!()
        };

        let blocks = get_blocks(&block.content);
        node.blocks = blocks;
        node.root.content = block.content;

        // blocks don't support inheritance
        parent = None;
    }

    let root = graph.insert_root(node);

    let mut cur = root;
    let mut parent_err = None;
    while let Some(p) = parent {
        let source = TemplateSource::File(p.into_owned());
        let node = TemplateNode::load(fs, cfg, opts.with_source(source))?;

        parent = get_parent(&node.root);
        // TODO: Polonius may fix this
        cur = match graph.insert_parent(cur, node) {
            Ok(v) => v,
            Err(e) => {
                parent_err = Some(e);
                break;
            }
        }
    }

    if let Some(e) = parent_err {
        return Err(Error::Parent(e.resolve(graph)));
    }

    Ok(graph.get_ref(root))
}

fn get_parent<'a>(root: &Root<'a>) -> Option<Cow<'a, str>> {
    match root.content.first() {
        Some(Item::Expr(Expr::Extends(e))) => Some(e.clone()),
        _ => None,
    }
}

fn get_blocks<'a, 'b>(
    content: impl IntoIterator<Item = &'a Item<'b>>,
) -> HashMap<String, ItemBlock<'b>>
where
    'b: 'a,
{
    content
        .into_iter()
        .filter_map(|i| match i {
            Item::Block(b) => Some(b),
            _ => None,
        })
        .flat_map(|b| {
            let content: Vec<_> = b
                .content
                .iter()
                .filter(|i| matches!(i, Item::Block(..)))
                .collect();
            if !content.is_empty() {
                let mut m = get_blocks(content);
                m.insert(b.name.to_string(), b.clone());
                m
            } else {
                HashMap::from([(b.name.to_string(), b.clone())])
            }
        })
        .collect()
}
