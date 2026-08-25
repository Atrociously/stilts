mod error;
mod heirarchy;
pub mod language;
mod pathing;
mod template;

pub use error::ParentError;
pub use language::{Language, StiltsLanguage};
pub use template::{TemplateData, TemplateNode, TemplateOptions, TemplateSource};

pub trait TemplateGraph {
    type Ref<'graph>: TemplateRef<'graph>
    where
        Self: 'graph;

    fn insert(&mut self, node: TemplateNode) -> Self::Ref<'_>;
    fn insert_parent<'g>(
        &'g mut self,
        child_ref: &Self::Ref<'g>,
        node: TemplateNode,
    ) -> Result<Self::Ref<'g>, ParentError<Self::Ref<'g>>>;
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
