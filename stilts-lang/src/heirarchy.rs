use crate::{ParentError, TemplateGraph, TemplateNode, TemplateRef, template::Config};

pub struct Heirarchy {
    nodes: Vec<TemplateNode>,
}

impl TemplateGraph for Heirarchy {
    type Ref<'g> = HeirarchyRef<'g>;

    fn insert(&mut self, node: TemplateNode) -> Self::Ref<'_> {
        self.nodes = Vec::new();

        todo!()
    }

    fn insert_parent<'g>(
        &'g mut self,
        child_ref: &Self::Ref<'g>,
        node: TemplateNode,
    ) -> Result<Self::Ref<'g>, ParentError<Self::Ref<'g>>> {
        if child_ref.idx != self.nodes.len() - 1 {
            return Err(ParentError::Exists {
                child: *child_ref,
                existing_parent: child_ref.parent().unwrap(),
            });
        }
        self.nodes.push(node);

        Ok(HeirarchyRef {
            idx: self.nodes.len() - 1,
            graph: &self.nodes,
        })
    }
}

#[derive(Clone, Copy)]
pub struct HeirarchyRef<'a> {
    idx: usize,
    graph: &'a [TemplateNode],
}

impl<'g> TemplateRef<'g> for HeirarchyRef<'g> {
    type Graph = Heirarchy;

    fn node(&self) -> &TemplateNode {
        &self.graph[self.idx]
    }

    fn parent(&self) -> Option<Self> {
        self.idx.checked_sub(1).map(|idx| Self { idx, ..*self })
    }

    fn child(&self) -> Option<Self> {
        let idx = self.idx.checked_add(1)?;
        if idx < self.graph.len() {
            Some(Self { idx, ..*self })
        } else {
            None
        }
    }
}
