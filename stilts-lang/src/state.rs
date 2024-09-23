use std::borrow::Cow;

use bitflags::bitflags;

type Lrc<T> = std::rc::Rc<T>;

/// Template Delimiters
///
/// Defines the delimiters used while parsing templates to differentiate
/// between expressions and template content
#[derive(Clone)]
pub struct Delims(Lrc<DelimContainer>);

impl Delims {
    pub fn new(open: impl Into<Cow<'static, str>>, close: impl Into<Cow<'static, str>>) -> Self {
        Self(Lrc::new(DelimContainer {
            open: open.into(),
            close: close.into(),
        }))
    }

    pub fn open(&self) -> &str {
        &self.0.open
    }

    pub fn close(&self) -> &str {
        &self.0.close
    }
}

struct DelimContainer {
    open: Cow<'static, str>,
    close: Cow<'static, str>,
}

impl Default for Delims {
    fn default() -> Self {
        Self(Lrc::new(DelimContainer {
            open: Cow::Borrowed("{%"),
            close: Cow::Borrowed("%}"),
        }))
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct State: u8 {
        const ALLOW_BLOCK     = 0x04;
        const ALLOW_EXTEND    = 0x02;
        const ALLOW_SUPERCALL = 0x01;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::ALLOW_BLOCK | Self::ALLOW_EXTEND
    }
}
