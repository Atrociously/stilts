use std::{borrow::Borrow, str::CharIndices};

use winnow::stream::{
    AsBytes, AsChar, Compare, FindSlice, Location, Offset, Stream, StreamIsPartial,
};

#[derive(Clone, Copy)]
pub struct Located<'s> {
    start: usize,
    end: usize,
    full_input: &'s str,
}

impl<'s> Located<'s> {
    pub fn new(input: &'s str) -> Self {
        Self {
            start: 0,
            end: input.len(),
            full_input: input,
        }
    }

    pub fn here(&self) -> Self {
        Self {
            end: self.start,
            ..*self
        }
    }

    pub fn span(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    pub fn content(&self) -> &'s str {
        &self.full_input[self.start..self.end]
    }

    pub fn source(&self) -> &'s str {
        self.full_input
    }
}

impl<'s> Stream for Located<'s> {
    type Token = char;
    type Slice = Self;
    type IterOffsets = CharIndices<'s>;
    type Checkpoint = Self;

    fn iter_offsets(&self) -> Self::IterOffsets {
        self.content().iter_offsets()
    }

    fn eof_offset(&self) -> usize {
        self.content().len()
    }

    fn next_token(&mut self) -> Option<Self::Token> {
        let c = self.content().chars().next()?;
        self.start += c.len();
        Some(c)
    }

    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.content().offset_for(predicate)
    }

    fn offset_at(&self, tokens: usize) -> Result<usize, winnow::error::Needed> {
        self.content().offset_at(tokens)
    }

    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        let slice = Self {
            end: self.start + offset,
            ..*self
        };
        self.start += offset;
        slice
    }

    fn checkpoint(&self) -> Self::Checkpoint {
        *self
    }

    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        *self = *checkpoint;
    }

    fn raw(&self) -> &dyn std::fmt::Debug {
        self
    }
}

impl<'s> StreamIsPartial for Located<'s> {
    type PartialState = ();

    fn complete(&mut self) -> Self::PartialState {}
    fn restore_partial(&mut self, _: Self::PartialState) {}
    fn is_partial_supported() -> bool {
        false
    }
}

impl<'s> Offset for Located<'s> {
    fn offset_from(&self, start: &Self) -> usize {
        self.start - start.start
    }
}

impl<'s> Location for Located<'s> {
    fn location(&self) -> usize {
        self.start
    }
}

impl<'s> AsBytes for Located<'s> {
    fn as_bytes(&self) -> &[u8] {
        self.content().as_bytes()
    }
}

impl<'s, T> Compare<T> for Located<'s>
where
    &'s str: Compare<T>,
{
    fn compare(&self, t: T) -> winnow::stream::CompareResult {
        self.content().compare(t)
    }
}

impl<'s, T> FindSlice<T> for Located<'s>
where
    &'s str: FindSlice<T>,
{
    fn find_slice(&self, substr: T) -> Option<std::ops::Range<usize>> {
        self.content().find_slice(substr)
    }
}

impl<'s> AsRef<Located<'s>> for Located<'s> {
    fn as_ref(&self) -> &Located<'s> {
        self
    }
}

impl AsRef<str> for Located<'_> {
    fn as_ref(&self) -> &str {
        self.content()
    }
}

impl Borrow<str> for Located<'_> {
    fn borrow(&self) -> &str {
        self.content()
    }
}

impl PartialEq<str> for Located<'_> {
    fn eq(&self, other: &str) -> bool {
        self.content() == other
    }
}

impl PartialEq<&str> for Located<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.content() == *other
    }
}

impl PartialEq for Located<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.full_input, other.full_input) && self.content() == other.content()
    }
}

impl std::fmt::Debug for Located<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.content().fmt(f)
    }
}

impl std::fmt::Display for Located<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.content())
    }
}
