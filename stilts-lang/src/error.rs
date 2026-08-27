use crate::TemplateGraph;

#[derive(Debug)]
pub enum TemplateError {
    Parse(stilts_parser::Error<'static>),
    Read(std::io::Error),
}

pub enum ParentError<T> {
    Circular { parentage: Vec<T> },
    Exists { child: T, existing_parent: T },
}

pub enum Error<T> {
    Template(TemplateError),
    Parent(ParentError<T>),
}

// TODO: Polonius may make this code irrelevant
impl<T> ParentError<T> {
    pub(crate) fn resolve<G: TemplateGraph<Idx = T>>(self, graph: &G) -> ParentError<G::Ref<'_>> {
        match self {
            Self::Circular { parentage } => ParentError::Circular {
                parentage: parentage
                    .into_iter()
                    .map(|idx| graph.get_ref(idx))
                    .collect(),
            },
            Self::Exists {
                child,
                existing_parent,
            } => ParentError::Exists {
                child: graph.get_ref(child),
                existing_parent: graph.get_ref(existing_parent),
            },
        }
    }
}

impl<T> Error<T> {
    pub(crate) fn resolve<G: TemplateGraph<Idx = T>>(self, graph: &G) -> Error<G::Ref<'_>> {
        match self {
            Self::Template(error) => Error::Template(error),
            Self::Parent(error) => Error::Parent(error.resolve(graph)),
        }
    }
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => error.fmt(f),
            Self::Read(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for TemplateError {}

impl<'a> From<stilts_parser::Error<'a>> for TemplateError {
    fn from(value: stilts_parser::Error<'a>) -> Self {
        Self::Parse(value.into_owned())
    }
}

impl From<std::io::Error> for TemplateError {
    fn from(value: std::io::Error) -> Self {
        Self::Read(value)
    }
}

impl<Ref> From<TemplateError> for Error<Ref> {
    fn from(value: TemplateError) -> Self {
        Self::Template(value)
    }
}

impl<Ref> From<ParentError<Ref>> for Error<Ref> {
    fn from(value: ParentError<Ref>) -> Self {
        Self::Parent(value)
    }
}
