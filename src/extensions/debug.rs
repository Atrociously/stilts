use core::fmt::{Debug, Display};

/// An extension for types that implement [`Debug`]
pub trait DebugExt {
    /// Create a formatter that will implement [`Display`]
    /// but uses the implementation of [`Debug`] on this type
    fn debug(&self) -> ToDisplay<'_, Self>;
}

impl<T> DebugExt for T
where
    T: Debug + ?Sized,
{
    fn debug(&self) -> ToDisplay<'_, Self> {
        ToDisplay(self)
    }
}

pub struct ToDisplay<'a, T: ?Sized>(&'a T);

impl<T> Display for ToDisplay<'_, T>
where
    T: ?Sized + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
