use std::borrow::Cow;

use bitflags::bitflags;

#[cfg(not(feature = "sync-state"))]
type Lrc<T> = std::rc::Rc<T>;

#[cfg(feature = "sync-state")]
type Lrc<T> = std::sync::Arc<T>;

/// Template Delimiters
///
/// Defines the delimiters used while parsing templates to differentiate
/// between expressions and template content
#[derive(Clone, Debug)]
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

#[derive(Debug)]
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

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Delims {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <[String; 2]>::deserialize(deserializer).map(|[open, close]| Self::new(open, close))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Delims {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        [self.open(), self.close()].serialize(serializer)
    }
}
