use std::borrow::Cow;

use miette::{Diagnostic, SourceSpan};

use crate::locate::Located;

/// A parser result type inspired by noms IResult
/// I replaced the I with L for the Located type since it is the only
/// input allowed in the parser anyways
pub type LResult<'i, T, E = Error<'i>> = std::result::Result<(Located<'i>, T), ErrFlow<E>>;

/// A trait for parser generalized parser errors
pub trait ParseErr<'i>: Sized {
    #![allow(unused_variables)]

    /// Create a new error
    fn new(msg: impl Into<Cow<'i, str>>) -> Self;

    /// Set the msg of an error
    /// by default this will just create a new error
    fn msg(self, msg: impl Into<Cow<'i, str>>) -> Self {
        Self::new(msg)
    }

    /// Set the label of an error
    /// This is the text that will point at the spanned source
    fn label(self, label: impl Into<Cow<'i, str>>) -> Self {
        self
    }

    /// Set the span of the error
    fn span(self, span: Located<'i>) -> Self {
        self
    }
}

/// The flow of errors through the parser
///
/// I took inspiration from both nom and winnow for this
#[derive(Debug)]
pub enum ErrFlow<E> {
    /// The error is recoverable because you might be able to parse something else
    Backtrack(E),
    /// The error is unrecoverable because you are definitely parsing something but it is malformed
    Unrecoverable(E),
    /// There is not enough input to finish parsing something
    Incomplete(E),
}

impl<E> ErrFlow<E> {
    /// Extract the underlying error from the flow
    pub fn into_err(self) -> E {
        match self {
            Self::Backtrack(e) | Self::Unrecoverable(e) | Self::Incomplete(e) => e,
        }
    }

    /// Convert the error to backtrackable
    pub fn to_backtrack(self) -> Self {
        Self::Backtrack(self.into_err())
    }

    /// Convert the error to unrecoverable
    pub fn to_unrecoverable(self) -> Self {
        Self::Unrecoverable(self.into_err())
    }

    /// Apply a function to the inner error of the flow
    pub fn map<F>(self, f: F) -> ErrFlow<E>
    where
        F: FnOnce(E) -> E,
    {
        match self {
            Self::Backtrack(e) => Self::Backtrack(f(e)),
            Self::Unrecoverable(e) => Self::Unrecoverable(f(e)),
            Self::Incomplete(e) => Self::Incomplete(f(e)),
        }
    }

    /// Apply a function to the inner error only if the flow is [`Incomplete`](ErrFlow::Incomplete)
    pub fn map_incomplete<F>(self, f: F) -> ErrFlow<E>
    where
        F: FnOnce(E) -> E,
    {
        match self {
            Self::Incomplete(e) => Self::Incomplete(f(e)),
            e => e,
        }
    }
}

impl<'i, E: ParseErr<'i>> ErrFlow<E> {
    /// Set the message of the inner error
    pub fn msg(self, msg: impl Into<Cow<'i, str>>) -> Self {
        self.map(|e| e.msg(msg))
    }

    /// Set the label of the inner error
    pub fn label(self, label: impl Into<Cow<'i, str>>) -> Self {
        self.map(|e| e.label(label))
    }

    /// Set the span of the inner error
    pub fn span(self, span: Located<'i>) -> Self {
        self.map(|e| e.span(span))
    }
}

pub trait LResultExt<'i, T, E: ParseErr<'i>> {
    fn map_err_incomplete<F>(self, f: F) -> Self
    where
        F: FnOnce(E) -> E;

    fn with_msg(self, msg: impl Into<Cow<'i, str>>) -> Self;
    fn with_label(self, label: impl Into<Cow<'i, str>>) -> Self;
    fn with_span(self, span: Located<'i>) -> Self;
}

impl<'i, T, E: ParseErr<'i>> LResultExt<'i, T, E> for LResult<'i, T, E> {
    fn map_err_incomplete<F>(self, f: F) -> Self
    where
        F: FnOnce(E) -> E,
    {
        match self {
            Self::Err(e) => Self::Err(e.map_incomplete(f)),
            r => r,
        }
    }

    fn with_msg(self, msg: impl Into<Cow<'i, str>>) -> Self {
        self.map_err(|e| e.msg(msg))
    }

    fn with_label(self, label: impl Into<Cow<'i, str>>) -> Self {
        self.map_err(|e| e.label(label))
    }

    fn with_span(self, span: Located<'i>) -> Self {
        self.map_err(|e| e.span(span))
    }
}

/// The default error type for parsing
#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("{msg}")]
pub struct Error<'i> {
    msg: Cow<'i, str>,
    label: Cow<'i, str>,
    #[label("{label}")]
    span: Option<SourceSpan>,
    #[source_code]
    source_code: Cow<'i, str>,
}

impl<'i> Error<'i> {
    /// Create a new error with a given message
    pub fn new(msg: impl Into<Cow<'i, str>>) -> Self {
        Self {
            msg: msg.into(),
            label: Cow::Borrowed("here"),
            span: None,
            source_code: Cow::Borrowed(""),
        }
    }

    /// Set the message of the error
    pub fn msg(mut self, msg: impl Into<Cow<'i, str>>) -> Self {
        self.msg = msg.into();
        self
    }

    /// Set the label of the error
    pub fn label(mut self, label: impl Into<Cow<'i, str>>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the span of the error
    pub fn span(mut self, input: Located<'i>) -> Self {
        self.span = Some(input.span());
        self.source_code = input.full().into();
        self
    }

    /// Create an error from a [`syn::Error`] and a span
    /// the span should be the input given to the syn parser
    pub fn from_syn(input: Located<'i>, value: syn::Error) -> Self {
        let start = value.span().start();
        let end = value.span().end();

        let start = get_offset(input, start.line, start.column + 1);
        let end = get_offset(input, end.line, end.column + 1);

        Self::new(value.to_string()).span(input.slice(start..end))
    }

    /// Create a string to display the error in a simple format
    /// this is usefull for error messages inside editors
    pub fn display_simple(&self) -> String {
        let msg = &self.msg;
        if let Some(span) = self.span {
            let source = &self.source_code;
            let (line_num, col) = get_line_col(source, span.offset());
            let line = source
                .lines()
                .nth(line_num - 1)
                .or_else(|| source.lines().last())
                .unwrap_or_default()
                .trim();
            format!("{msg} [{line_num}:{col}] {line:?}")
        } else {
            format!("{msg}")
        }
    }

    /// Create an owned version of the error
    pub fn into_owned(self) -> Error<'static> {
        Error {
            msg: self.msg.into_owned().into(),
            span: self.span,
            label: self.label.into_owned().into(),
            source_code: self.source_code.into_owned().into(),
        }
    }
}

impl<'i> ParseErr<'i> for Error<'i> {
    fn new(msg: impl Into<Cow<'i, str>>) -> Self {
        Self::new(msg)
    }

    fn msg(self, msg: impl Into<Cow<'i, str>>) -> Self {
        self.msg(msg)
    }

    fn label(self, label: impl Into<Cow<'i, str>>) -> Self {
        self.label(label)
    }

    fn span(self, span: Located<'i>) -> Self {
        self.span(span)
    }
}

fn get_line_col(source: impl AsRef<str>, offset: usize) -> (usize, usize) {
    let source = source.as_ref();

    let mut line = 0;
    let mut col = 0;
    let mut cursor = 0;
    for ch in source.chars() {
        if cursor >= offset {
            break;
        }
        if ch == '\n' {
            col = 0;
            line += 1;
        } else {
            col += 1;
        }
        cursor += ch.len_utf8();
    }

    (line + 1, col)
}

#[allow(dead_code)]
fn get_offset(source: impl AsRef<str>, loc_line: usize, loc_col: usize) -> usize {
    let source = source.as_ref();
    // this code is ripped straight from miette but I needed direct access to the usize to
    // calculate the length
    let mut line = 0;
    let mut col = 0;
    let mut offset = 0;
    for char in source.chars() {
        if line + 1 >= loc_line && col + 1 >= loc_col {
            break;
        }
        if char == '\n' {
            col = 0;
            line += 1;
        } else {
            col += 1;
        }
        offset += char.len_utf8();
    }
    offset
}
