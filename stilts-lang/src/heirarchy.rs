use std::collections::HashMap;

use camino::Utf8PathBuf;

use crate::{TemplateGraph, TemplateNode, TemplateRef, error::ParentError};

#[derive(Default)]
pub struct Heirarchy {
    nodes: Vec<TemplateNode>,
    cycle_map: HashMap<Utf8PathBuf, HeirarchyIdx>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HeirarchyIdx(usize);

#[derive(Clone, Copy)]
pub struct HeirarchyRef<'a> {
    idx: usize,
    graph: &'a [TemplateNode],
}

impl Heirarchy {
    fn ref_of(&self, idx: usize) -> HeirarchyRef<'_> {
        HeirarchyRef {
            idx,
            graph: &self.nodes,
        }
    }

    fn check_for_cycle(&mut self, node: &TemplateNode) -> Result<(), ParentError<HeirarchyIdx>> {
        let Some(source_path) = &node.data.source_path else {
            return Ok(());
        };
        if let Some(idx) = self.cycle_map.get(source_path) {
            Err(ParentError::Circular {
                parentage: (idx.0..self.nodes.len()).map(HeirarchyIdx).collect(),
            })
        } else {
            Ok(())
        }
    }
}

impl TemplateGraph for Heirarchy {
    type Idx = HeirarchyIdx;
    type Ref<'g> = HeirarchyRef<'g>;

    fn get_ref(&self, idx: Self::Idx) -> Self::Ref<'_> {
        self.ref_of(idx.0)
    }

    fn insert_root(&mut self, node: TemplateNode) -> Self::Idx {
        let root = HeirarchyIdx(0);
        if let Some(source_path) = &node.data.source_path {
            self.cycle_map.insert(source_path.clone(), root);
        }
        self.nodes = vec![node];

        root
    }

    fn insert_parent(
        &mut self,
        child_idx: Self::Idx,
        node: TemplateNode,
    ) -> Result<Self::Idx, ParentError<Self::Idx>> {
        if child_idx.0 != self.nodes.len() - 1 {
            return Err(ParentError::Exists {
                child: child_idx,
                existing_parent: HeirarchyIdx(child_idx.0 - 1),
            });
        }
        self.check_for_cycle(&node)?;
        self.nodes.push(node);

        Ok(HeirarchyIdx(self.nodes.len() - 1))
    }
}

impl<'g> TemplateRef<'g> for HeirarchyRef<'g> {
    type Graph = Heirarchy;

    fn node(&self) -> &TemplateNode {
        &self.graph[self.idx]
    }

    fn parent(&self) -> Option<Self> {
        let idx = self.idx.checked_add(1)?;
        if idx < self.graph.len() {
            Some(Self { idx, ..*self })
        } else {
            None
        }
    }

    fn child(&self) -> Option<Self> {
        self.idx.checked_sub(1).map(|idx| Self { idx, ..*self })
    }
}
