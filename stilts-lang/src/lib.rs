pub mod error;
mod file_system;
mod graph_load;
mod heirarchy;
mod pathing;
mod template;

pub use file_system::{FileSystem, StdFS};
pub use graph_load::load;
pub use heirarchy::Heirarchy;
pub use template::{TemplateConfig, TemplateData, TemplateNode, TemplateOptions, TemplateSource};

pub trait TemplateGraph {
    type Idx: Copy;
    type Ref<'graph>: TemplateRef<'graph>
    where
        Self: 'graph;

    fn get_ref(&self, idx: Self::Idx) -> Self::Ref<'_>;

    fn insert_root(&mut self, node: TemplateNode) -> Self::Idx;
    fn insert_parent(
        &mut self,
        child_idx: Self::Idx,
        node: TemplateNode,
    ) -> Result<Self::Idx, error::ParentError<Self::Idx>>;
}

pub trait TemplateRef<'graph>: Sized {
    type Graph: TemplateGraph<Ref<'graph> = Self> + 'graph;

    fn node(&self) -> &TemplateNode;

    fn parent(&self) -> Option<Self>;
    fn child(&self) -> Option<Self>;

    fn parentage(self) -> impl Iterator<Item = Self> {
        Parentage { cur: Some(self) }
    }
}

struct Parentage<Ref> {
    cur: Option<Ref>,
}

impl<'g, Ref: TemplateRef<'g>> Iterator for Parentage<Ref> {
    type Item = Ref;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.cur.take()?;
        self.cur = v.parent();
        Some(v)
    }
}
