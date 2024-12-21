use std::fmt::Display;

use crate::escaping::MarkedSafe;

/// An extension for types that implement [`Display`]
pub trait DisplayExt {
    /// Mark the value as safe from escaping
    fn safe(&self) -> MarkedSafe<'_, Self>;

    /// Convert the display output to lowercase
    fn lowercase(&self) -> String;

    /// Convert the display output to uppercase
    fn uppercase(&self) -> String;
}

impl<T> DisplayExt for T
where
    T: Display,
{
    fn safe(&self) -> MarkedSafe<'_, Self> {
        MarkedSafe(self)
    }

    fn lowercase(&self) -> String {
        self.to_string().to_lowercase()
    }

    fn uppercase(&self) -> String {
        self.to_string().to_uppercase()
    }
}

/// Wrapper around a template which implements [`Display`](std::fmt::Display)
pub struct DisplayTemplate<'a, T: ?Sized>(pub &'a T);

impl<'a, T: crate::Template> std::fmt::Display for DisplayTemplate<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
