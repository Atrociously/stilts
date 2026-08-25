use std::borrow::Cow;

use miette::{Diagnostic, SourceSpan};
use winnow::{error::{ErrMode, ErrorKind}, stream::Stream};

use crate::Located;

/// The default error type for parsing
#[derive(Clone, Debug, thiserror::Error, Diagnostic)]
#[error("{msg}")]
pub struct Error<'i> {
    msg: Cow<'i, str>,
    label: Cow<'i, str>,
    #[label("{label}")]
    span: Option<SourceSpan>,
    #[source_code]
    source_code: Cow<'i, str>,
    #[related]
    related: Vec<Self>,
    pub(crate) kind: Option<ErrorKind>
}

impl<'i> Error<'i> {
    /// Create a new error with a given message
    pub fn new(msg: impl Into<Cow<'i, str>>) -> Self {
        Self {
            msg: msg.into(),
            label: Cow::Borrowed("here"),
            span: None,
            source_code: Cow::Borrowed(""),
            related: Vec::new(),
            kind: None,
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
        self.span = Some(input.span().into());
        self.source_code = input.source().into();
        self
    }

    /// Create an error from a [`syn::Error`] and a span
    /// the span should be the input given to the syn parser
    pub fn from_syn(mut input: Located<'i>, value: syn::Error) -> Self {
        let start = value.span().start();
        let end = value.span().end();

        let start = get_offset(input, start.line, start.column + 1);
        let end = get_offset(input, end.line, end.column + 1);

        input.next_slice(start);
        let span = input.next_slice(end);
        Self::new(value.to_string()).span(span)
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
            label: self.label.into_owned().into(),
            source_code: self.source_code.into_owned().into(),
            related: self.related.into_iter()
                .map(Self::into_owned)
                .collect(),
            ..self
        }
    }

    pub fn backtrack(self) -> ErrMode<Self> {
        ErrMode::Backtrack(self)
    }

    pub fn cut(self) -> ErrMode<Self> {
        ErrMode::Cut(self)
    }
}

impl<'i, I> winnow::error::ParserError<I> for Error<'i>
where
    I: Stream + AsRef<Located<'i>>,
{
    fn from_error_kind(input: &I, kind: winnow::error::ErrorKind) -> Self {
        let mut err = Self::new(kind.description().to_string()).span(input.as_ref().here());
        err.kind = Some(kind);
        err
    }

    fn append(
        self,
        _input: &I,
        _token_start: &<I as winnow::stream::Stream>::Checkpoint,
        _kind: winnow::error::ErrorKind,
    ) -> Self {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Msg<'i>(pub &'i str);
#[derive(Clone, Debug)]
pub struct At<'i>(pub Located<'i>);

impl<'i, I: Stream> winnow::error::AddContext<I, Msg<'i>> for Error<'i> {
    fn add_context(
        self,
        _input: &I,
        _token_start: &<I as Stream>::Checkpoint,
        context: Msg<'i>,
    ) -> Self {
        self.msg(context.0)
    }
}

impl<'i, I: Stream> winnow::error::AddContext<I, At<'i>> for Error<'i> {
    fn add_context(
        self,
        _input: &I,
        _token_start: &<I as Stream>::Checkpoint,
        context: At<'i>,
    ) -> Self {
        self.span(context.0)
    }
}

impl<'i, I: Stream> winnow::error::AddContext<I, Self> for Error<'i> {
    fn add_context(
        mut self,
        _input: &I,
        _token_start: &<I as Stream>::Checkpoint,
        context: Self,
    ) -> Self {
        self.related.push(context);
        self
    }
}

pub(crate) fn expect_end<'i>(
    open_expr: Located<'i>,
) -> impl FnOnce(ErrMode<Error<'i>>) -> ErrMode<Error<'i>> {
    move |errmode| {
        let slice_bad = matches!(&errmode, ErrMode::Backtrack(e) | ErrMode::Cut(e) if e.kind == Some(ErrorKind::Slice));
        if errmode.is_incomplete() || slice_bad {
            ErrMode::Cut(
                Error::new("expected closing {% end %} expression")
                    .label("opening block expression")
                    .span(open_expr),
            )
        } else {
            errmode
        }
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
