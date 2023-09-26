use std::{
    borrow::Cow,
    ops::{Bound, RangeBounds},
};

use miette::SourceSpan;

/// A struct that is capable of slicing into a given input
///
/// It maintains a reference to the entire input slice, but
/// manually keeps track of the slice bounds. It implements
/// [`AsRef<str>`] which will return a string slice with the bounds
/// specified by this span. You can extract the full text however
#[derive(Clone, Copy)]
pub struct Located<'a> {
    offset: usize,
    end_offset: usize,
    inner: &'a str,
}

impl<'i> Located<'i> {
    /// Create a new span from the entire contents of a string
    pub fn new(inner: &'i str) -> Self {
        Self {
            offset: 0,
            end_offset: inner.len(),
            inner,
        }
    }

    /// Get the total offset into the source string
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the total byte offset into the source string
    ///
    /// This requires iterating over the chars to count their byte lengths
    pub fn byte_offset(&self) -> usize {
        self.inner
            .chars()
            .take(self.offset)
            .fold(0, |acc, ch| acc + ch.len_utf8())
    }

    /// Get the byte length of the source string
    ///
    /// This requires iterating over the span to count char byte lengths
    pub fn byte_len(&self) -> usize {
        self.inner
            .chars()
            .skip(self.offset)
            .take(self.end_offset - self.offset)
            .fold(0, |acc, ch| acc + ch.len_utf8())
    }

    /// Create a [`miette`] [`SourceSpan`] from this it uses byte offset and length
    pub fn span(&self) -> SourceSpan {
        SourceSpan::new(self.byte_offset().into(), self.byte_len().into())
    }

    /// Try to slice relative to the current offset
    /// if slicing is not possible [`None`] will be returned
    pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Option<Self> {
        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(offset) => *offset,
            Bound::Excluded(offset) => *offset + 1,
        };
        let end = match range.end_bound() {
            Bound::Unbounded => self.end_offset - self.offset,
            Bound::Included(offset) => *offset + 1,
            Bound::Excluded(offset) => *offset,
        };

        // check the bounds are valid
        let start_oob = self.offset + start > self.end_offset;
        let end_oob = self.offset + end > self.end_offset;
        if start_oob || end_oob {
            return None;
        }

        Some(Self {
            offset: self.offset + start,
            end_offset: self.offset + end,
            inner: self.inner,
        })
    }

    /// Join two spans together
    ///
    /// this will create a new span starting at self and ending at the end of other
    ///
    /// # Panics
    /// - This function will panic if it is called on two spans with differing source code
    pub fn join(&self, other: Located<'i>) -> Self {
        if self.inner != other.inner {
            panic!("join called on two spans of differing origin")
        }
        Self {
            offset: self.offset,
            end_offset: other.end_offset,
            inner: self.inner,
        }
    }

    /// Slice the span will try to slice and unwrap the result
    ///
    /// # Panics
    /// - It will panic if slicing fails by going out of bounds
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        self.try_slice(range).unwrap()
    }

    /// Gets the contents of the spanned source code
    pub fn spanned(&self) -> &'i str {
        &self.inner[self.offset..self.end_offset]
    }

    /// Get the full contents of the source code disreguarding span
    pub fn full(&self) -> &'i str {
        self.inner
    }
}

impl AsRef<str> for Located<'_> {
    fn as_ref(&self) -> &str {
        self.spanned()
    }
}

impl std::ops::Deref for Located<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.spanned()
    }
}

impl<'a> From<Located<'a>> for Cow<'a, str> {
    fn from(value: Located<'a>) -> Self {
        Self::Borrowed(value.spanned())
    }
}

impl<'i> From<&'i str> for Located<'i> {
    fn from(value: &'i str) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Debug for Located<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Located")
            .field("offset", &self.offset)
            .field("end_offset", &self.end_offset)
            .field("inner", &self.as_ref())
            .finish()
    }
}

impl std::fmt::Display for Located<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
