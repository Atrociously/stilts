//! Contains all the runtime related code for handling opt-out escaping in stilts
//!
//! ## Custom escape formats
//! If you have a need to escape something other than html you'll have to implement the [`Escaper`]
//! trait. This is the essential escape format mechanism, the code generator will wrap anything in
//! an expression with an [`Escaped`] this struct will conditionally use the specified [`Escaper`]
//! to perform the desired escaping.

use std::fmt::Display;

/// A struct that marks a type as safe meaning it can skip escaping
pub struct MarkedSafe<'a, T: ?Sized>(pub &'a T);

/// A wrapper type for conditionally escaping data
///
/// Uses the [`Display`] implementation of `T` and the [`Escaper`] implementation
/// of E to perform the escaping in it's implementation of [`Display`]
pub struct Escaped<'a, T: ?Sized, E = Empty> {
    value: &'a T,
    escaper: E,
}

/// A trait to implement language escaping generically
///
/// For an implementation of this trait see [`Html`] which uses [`html_escape`]
pub trait Escaper {
    /// Write the escaped contents of `T` to the formatter
    fn fmt<T: Display + ?Sized>(&self, value: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T, E> Escaped<'_, T, E>
where
    T: ?Sized,
    E: Escaper,
{
    /// Create a new escaped value, this is mostly used by the stilts code generator
    #[inline]
    pub fn new(value: &T, escaper: E) -> Escaped<'_, T, E> {
        Escaped {
            value,
            escaper,
        }
    }
}

impl<'a, T, E> Display for Escaped<'a, MarkedSafe<'a, T>, E>
where
    T: Display + ?Sized,
    E: Escaper,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.0.fmt(f)
    }
}

impl<T, E> Display for Escaped<'_, T, E>
where
    T: Display + ?Sized,
    E: Escaper,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.escaper.fmt(self.value, f)
    }
}

/// An empty escaper it is a no-op
pub struct Empty;
/// An html escaper it uses [`html_escape`]
pub struct Html;

impl Escaper for Empty {
    fn fmt<T: Display + ?Sized>(&self, value: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        value.fmt(f)
    }
}

impl Escaper for Html {
    fn fmt<T: Display + ?Sized>(&self, value: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = value.to_string();
        let res = html_escape::encode_safe(&value);
        f.write_str(&res)
    }
}
