//! Stilts-Lang is the parser / language definition for stilts
//! To parse a full stilts template see the [`Root`] parser.

#![deny(unsafe_code)]
#![deny(clippy::all)]
#![warn(missing_docs)]

use error::LResultExt;
use parse::{alt, eof, whitespace1, Parseable, Parser};
use syn::{punctuated::Punctuated, Label, LitStr, Token};

mod error;
mod locate;
/// The module containing the custom parser combinators used in parsing the language
pub mod parse;

pub use error::{ErrFlow, Error, LResult, ParseErr};
pub use locate::Located;

use crate::parse::{many_till, many_until, tag, take_till, take_until, whitespace0};

type CowStr<'a> = std::borrow::Cow<'a, str>;
type Delims<'a> = [&'a str; 2];

/// A context used to allow certain expressions in certain places
///
/// For instance the default is to only allow blocks within the
/// root and other blocks not within other expressions.
/// And extends is only allowed as the very first item when parsing root.
#[derive(Clone, Copy)]
pub struct Ctx {
    /// flag to determine whether blocks are allowed
    pub allow_blocks: bool,
    /// flag to determine whether extends are allowed
    pub allow_extends: bool,
}

impl Ctx {
    /// Allow blocks and extends
    pub const fn allow() -> Self {
        Self {
            allow_blocks: true,
            allow_extends: true,
        }
    }

    /// Only allow blocks
    pub const fn block_only() -> Self {
        Self {
            allow_blocks: true,
            allow_extends: false,
        }
    }

    /// Don't allow blocks or extends
    pub const fn deny() -> Self {
        Self {
            allow_blocks: false,
            allow_extends: false,
        }
    }
}

// The message to display when an expression reaches eof wihout encountering an end expression
const EXPECT_END_MSG: &str = "expected closing {% end %} expression";

fn parse_delimited<'i>(delims: Delims<'i>) -> impl FnMut(Located<'i>) -> LResult<'i, Located<'i>> {
    // consume a whole string to the end
    fn parse_string(input: Located) -> LResult<()> {
        // update the error with the desired information
        fn update_err<'a>(e: Error<'a>, s: Located<'a>) -> Error<'a> {
            e.msg("expected string to close within an expression")
                .label("string opened here")
                .span(s)
        }

        alt((
            tag("\"")
                .and_then(|rem, t| {
                    take_till(tag("\""))(rem).map_err_incomplete(|e| update_err(e, t))
                })
                .map(|_| ()),
            tag("r\"")
                .and_then(|rem, t| {
                    take_till(tag("\""))(rem).map_err_incomplete(|e| update_err(e, t))
                })
                .map(|_| ()),
            tag("r")
                .and_then(|rem, t| {
                    many_till(tag("#"), tag("\""))(rem).map(|(rem, (p, t2))| (rem, (p, t.join(t2))))
                })
                .and_then(|rem, (pounds, t)| {
                    // unwraps are fine because if there were no #s then the previous parser would
                    // match and we wouldn't be here
                    let pounds = pounds.first().unwrap().join(*pounds.last().unwrap());
                    let (rem, _) = take_till(tag(pounds))(rem)
                        .map_err_incomplete(|e| update_err(e, t))
                        .map_err(ErrFlow::to_unrecoverable)?;
                    LResult::<_, Error>::Ok((rem, ()))
                }),
        ))(input)
    }

    // just parse the start of the string
    fn string_start(input: Located) -> LResult<()> {
        alt((
            tag("\"").map(|_| ()),
            tag("r\"").map(|_| ()),
            tag("r")
                .and_then(|rem, _| many_till(tag("#"), tag("\""))(rem))
                .map(|_| ()),
        ))(input)
    }

    // parse the content between the delimiters and ignore delimiters within strings
    move |input| {
        // parse the first delimiter out
        let (input, _) = tag(delims[0])(input)?;

        // take until we hit the closing delim or a string expression
        let (mut input, mut inner) =
            take_until(alt((tag(delims[1]).map(|_| ()), string_start)))(input)
                .map_err(ErrFlow::to_unrecoverable)?;

        let mut res = tag(delims[1])(input);
        while let Err(ErrFlow::Backtrack(_)) = res {
            // if we hit a string expression then we consume it and continue from there
            let (next, _) = parse_string(input).map_err(ErrFlow::to_unrecoverable)?;
            let (next, next_inner) =
                take_until(alt((tag(delims[1]).map(|_| ()), string_start)))(next)
                    .map_err(ErrFlow::to_unrecoverable)?;

            // join the span with the new span
            inner = inner.join(next_inner);
            input = next;
            // check if we hit the end delimiter or another string
            res = tag(delims[1])(input)
        }

        match res {
            Ok((rem, _)) => Ok((rem, inner)),
            Err(e) => Err(e),
        }
    }
}

/// The expressions that are available within the language
/// There are generally two types of expressions
/// A simple expression is some content between delimiters
/// A more complex expression is one that has other [`Item`]s
/// within it generally closed with an [`End`] for expample the [`BlockExpr`].
///
/// An example of a simple expression:
/// ```html
/// {% include "othertemplate" %}
/// ```
/// An example of a complex expression:
/// ```html
/// {% block myblock %}
///     Other items in here
///     {% include "othertemplate" %}
/// {% end %}
/// ```
#[derive(Clone, Debug)]
pub enum Expr<'a> {
    /// An [`extends`](ExtendsExpr) expression
    Extends(ExtendsExpr),
    /// A [`block`](BlockExpr) expression
    Block(BlockExpr<'a>),
    /// An [`include`](IncludeExpr) expression
    Include(IncludeExpr),
    /// A [`for`](ForExpr) expression
    For(ForExpr<'a>),
    /// An [`if`](IfExpr) expression
    If(IfExpr<'a>),
    /// A [`match`](MatchExpr) expression
    Match(MatchExpr<'a>),
    /// A [`macro`](MacroExpr) expression
    Macro(MacroExpr<'a>),
    /// A [`call`](CallMacroExpr) expression
    CallMacro(CallMacroExpr),
    /// A rust [`statement`](syn::Stmt)
    Stmt(syn::Stmt),
    /// A rust [`expression`](syn::Expr)
    Expr(syn::Expr),
}

impl<'i> Expr<'i> {
    fn fallback(input: Located<'i>) -> LResult<'i, Self> {
        let _ = tag("{%")(input)
            .map_err(|e| e.msg("expected delim for expresion").span(input.slice(..0)))?;

        // we now know we are parsing an expression so any errors beyond this point are
        // unrecoverable
        let (rem, inner) =
            parse_delimited(["{%", "%}"])(input).map_err(ErrFlow::to_unrecoverable)?;

        if let Ok(stmt) = syn::parse_str(inner.as_ref()) {
            Ok((rem, Expr::Stmt(stmt)))
        } else {
            let expr = syn::parse_str(inner.as_ref())
                .map_err(|err| Error::from_syn(inner, err))
                .map_err(ErrFlow::Unrecoverable)?;
            Ok((rem, Expr::Expr(expr)))
        }
    }

    /// The parser factory for an expression that requires the [`Ctx`]
    pub fn parser(ctx: Ctx) -> impl FnMut(Located<'i>) -> LResult<'i, Expr<'i>> {
        move |input| {
            alt((
                ExtendsExpr::parse_next.and_then(move |rem, extends| match ctx.allow_extends {
                    true => Ok((rem, Self::Extends(extends))),
                    false => Err(ErrFlow::Unrecoverable(
                        Error::new("extends expressions not allowed in this context")
                            .span(input.slice(..0)),
                    )),
                }),
                BlockExpr::parse_next.and_then(move |rem, block| match ctx.allow_blocks {
                    true => Ok((rem, Self::Block(block))),
                    false => Err(ErrFlow::Unrecoverable(
                        Error::new("blocks not allowed in this context").span(input.slice(..0)),
                    )),
                }),
                IncludeExpr::parse_next.map(Self::Include),
                ForExpr::parse_next.map(Self::For),
                IfExpr::parse_next.map(Self::If),
                MatchExpr::parse_next.map(Self::Match),
                MacroExpr::parse_next.map(Self::Macro),
                CallMacroExpr::parse_next.map(Self::CallMacro),
                Self::fallback,
            ))(input)
        }
    }
}

/// An item in the template
///
/// Can be either an [`expression`](Expr) or template content
/// template content is stored as a slice into the original input
/// but can be made into a [`String`].
#[derive(Clone, Debug)]
pub enum Item<'a> {
    /// An [`Expr`]
    Expr(Expr<'a>),
    /// Template content
    Content(CowStr<'a>),
}

impl<'i> Item<'i> {
    /// Force the item to become an owned version of itself
    ///
    /// this will allocate a string if it is content,
    /// and will allocate many strings for expressions that
    /// contain other items
    pub fn into_owned(self) -> Item<'static> {
        match self {
            Self::Content(c) => Item::Content(c.into_owned().into()),
            Self::Expr(Expr::Extends(e)) => Item::Expr(Expr::Extends(e)),
            Self::Expr(Expr::Include(i)) => Item::Expr(Expr::Include(i)),
            Self::Expr(Expr::Stmt(s)) => Item::Expr(Expr::Stmt(s)),
            Self::Expr(Expr::Expr(e)) => Item::Expr(Expr::Expr(e)),
            Self::Expr(Expr::Block(b)) => Item::Expr(Expr::Block(BlockExpr {
                name: b.name,
                content: b.content.into_iter().map(BlockItem::into_owned).collect(),
            })),
            Self::Expr(Expr::If(i)) => Item::Expr(Expr::If(IfExpr {
                cond: i.cond,
                then: i.then.into_iter().map(Item::into_owned).collect(),
                close: i.close.into_owned(),
            })),
            Self::Expr(Expr::For(f)) => Item::Expr(Expr::For(ForExpr {
                label: f.label,
                pat: f.pat,
                expr: f.expr,
                content: f.content.into_iter().map(Item::into_owned).collect(),
            })),
            Self::Expr(Expr::Match(m)) => Item::Expr(Expr::Match(MatchExpr {
                expr: m.expr,
                arms: m
                    .arms
                    .into_iter()
                    .map(|a| MatchArm {
                        pat: a.pat,
                        guard: a.guard,
                        content: a.content.into_iter().map(Item::into_owned).collect(),
                    })
                    .collect(),
            })),
            Self::Expr(Expr::Macro(m)) => Item::Expr(Expr::Macro(MacroExpr {
                name: m.name,
                args: m.args,
                content: m.content.into_iter().map(Item::into_owned).collect(),
            })),
            Self::Expr(Expr::CallMacro(c)) => Item::Expr(Expr::CallMacro(c)),
        }
    }
}

impl<'i> Item<'i> {
    /// A parser factory that will parse an item with the provided context
    pub fn parser(ctx: Ctx) -> impl FnMut(Located<'i>) -> LResult<'i, Item<'i>> {
        move |input| match tag("{%")(input) {
            Ok(_) => {
                let (rem, expr) = Expr::parser(ctx)(input)?;
                Ok((rem, Self::Expr(expr)))
            }
            Err(ErrFlow::Backtrack(_)) | Err(ErrFlow::Incomplete(_)) => {
                match take_until(tag("{%"))(input) {
                    Ok((rem, content)) => Ok((rem, Self::Content(content.into()))),
                    Err(ErrFlow::Incomplete(_)) => {
                        Ok((input.slice(input.len()..), Self::Content(input.into())))
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// An expression for templates to inherit from other templates
///
/// ```html
/// {% extends "myparent" %}
/// ```
#[derive(Clone, Debug)]
pub struct ExtendsExpr {
    /// the referenced parent template
    pub reference: LitStr,
}

impl<'i> Parseable<'i> for ExtendsExpr {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?;
        let (inner, _) = tag("extends")(inner)
            .map_err(ErrFlow::to_backtrack)?;
        let (inner, _) = whitespace1(inner).map_err(ErrFlow::to_unrecoverable)?;

        let reference = syn::parse_str(inner.as_ref())
            .map_err(|err| Error::from_syn(inner, err))
            .map_err(ErrFlow::Unrecoverable)?;

        Ok((input, Self { reference }))
    }
}

/// A block is useful to define inheritable zones within templates
///
/// base template
/// ```html
/// {% block myblock %}
///     This stuff will be rendered
///     unless overriden in the child
/// {% end %}
/// ```
///
/// child template
/// ```html
/// {% extends "base" %}
///
/// {% block myblock %}
///     This will override the base
///     {% super() %} but this will bring the parents contents back
///     {% super() %} and it can be called multiple times
/// {% endblock %}
/// ```
#[derive(Clone, Debug)]
pub struct BlockExpr<'a> {
    /// The name of the block
    pub name: syn::Ident,
    /// The inner content of the block
    pub content: Vec<BlockItem<'a>>,
}

impl<'i> Parseable<'i> for BlockExpr<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?;
        let (inner, block_tag) = tag("block")(inner)
            .map_err(ErrFlow::to_backtrack)?;
        let (inner, _) = whitespace1(inner).map_err(ErrFlow::to_unrecoverable)?;

        let name = syn::parse_str(inner.as_ref())
            .map_err(|err| Error::from_syn(inner, err))
            .map_err(ErrFlow::Unrecoverable)?;

        let (input, (content, _)) = many_till(BlockItem::parse_next, End::parse_next)(input)
            .map_err_incomplete(|e| e.span(block_tag).msg(EXPECT_END_MSG))
            .map_err(ErrFlow::to_unrecoverable)?;

        Ok((input, Self { name, content }))
    }
}

/// A block item is special compared to a regular item
/// because it can contain super calls as well as regular items
#[derive(Clone, Debug)]
pub enum BlockItem<'i> {
    /// A regular item
    Item(Item<'i>),
    /// A call to the super block
    SuperCall,
}

impl<'i> BlockItem<'i> {
    pub(crate) fn into_owned(self) -> BlockItem<'static> {
        match self {
            Self::Item(i) => BlockItem::Item(i.into_owned()),
            Self::SuperCall => BlockItem::SuperCall,
        }
    }
}

impl<'i> Parseable<'i> for BlockItem<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self, Error> {
        alt((
            SuperCall::parse_next.map(|_| Self::SuperCall),
            Item::parser(Ctx::block_only()).map(Self::Item),
        ))(input)
    }
}

/// A parser that will parse a super call expression
#[derive(Clone, Debug)]
pub struct SuperCall;

impl<'i> Parseable<'i> for SuperCall {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self, Error> {
        let (input, _) = tag("{%")(input)?;
        let (input, _) = whitespace0(input)?;
        let (input, _) = tag("super()")(input)?;
        let (input, _) = whitespace0(input)?;
        let (input, _) = tag("%}")(input).map_err(ErrFlow::to_unrecoverable)?;

        Ok((input, Self))
    }
}

/// An includes expression is used to
/// include the content of another template inside this template
///
/// ```html
/// {% includes "othertemplate" %}
/// ```
#[derive(Clone, Debug)]
pub struct IncludeExpr {
    /// The template that should be included
    pub includes: LitStr,
}

impl<'i> Parseable<'i> for IncludeExpr {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?;
        let (inner, _) = tag("include")(inner)
            .map_err(ErrFlow::to_backtrack)?;
        let (inner, _) = whitespace1(inner).map_err(ErrFlow::to_unrecoverable)?;

        let includes = syn::parse_str(inner.as_ref())
            .map_err(|err| Error::from_syn(inner, err))
            .map_err(ErrFlow::Unrecoverable)?;

        Ok((input, Self { includes }))
    }
}

/// A for expression that is similar to a regular rust for statement
///
/// The major difference is that this expression has template content
/// instead of a block. In fact notice that the language supports both
/// this expression and rust for statements as valid.
///
/// ```html
/// {% for i in 0..10 %}
///     My template content
/// {% end %}
///
/// This however is also valid
/// {% for i in 0..10 {
///     println!("my forloop")
/// } %}
/// ```
#[derive(Clone, Debug)]
pub struct ForExpr<'a> {
    /// An optional loop label
    pub label: Option<Label>,
    /// The pattern to destructure each loop item
    pub pat: syn::Pat,
    /// The loop expression that implements [`IntoIterator`]
    pub expr: syn::Expr,
    /// The inner content of the loop
    pub content: Vec<Item<'a>>,
}

impl<'i> Parseable<'i> for ForExpr<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?; // remove whitespace
        let open: ForOpen = syn::parse_str(inner.as_ref())
            .map_err(|err| Error::from_syn(inner, err))
            .map_err(ErrFlow::Backtrack)?;
        let (input, (content, _)) = many_till(Item::parser(Ctx::deny()), End::parse_next)(input)
            .map_err_incomplete(|e| e.span(inner).msg(EXPECT_END_MSG))
            .map_err(ErrFlow::to_unrecoverable)?;

        Ok((
            input,
            Self {
                label: open.label,
                pat: open.pat,
                expr: open.expr,
                content,
            },
        ))
    }
}

#[derive(Clone, Debug)]
struct ForOpen {
    pub label: Option<Label>,
    #[allow(dead_code)]
    for_token: Token![for],
    pub pat: syn::Pat,
    #[allow(dead_code)]
    in_token: Token![in],
    pub expr: syn::Expr,
}

impl syn::parse::Parse for ForOpen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            label: input.parse()?,
            for_token: input.parse()?,
            pat: input.call(syn::Pat::parse_single)?,
            in_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

/// An end expression used to close complex expressions
#[derive(Clone, Debug)]
pub struct End;

impl<'i> Parseable<'i> for End {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, _) = tag("{%")(input)?;
        let (input, _) = whitespace0(input)?;
        let (input, _) = tag("end")(input)
            .map_err(ErrFlow::to_backtrack)?;
        let (input, _) = whitespace0(input)?;
        let (input, _) = tag("%}")(input).map_err(ErrFlow::to_unrecoverable)?;

        Ok((input, Self))
    }
}

/// An if expresssion to optionally render template content
///
/// ```html
/// {% if some_value == 1 %}
///     Say something
/// {% else if some_value == 2 %}
///     Say something else
/// {% else %}
///     How did you get here?
/// {% end %}
/// ```
#[derive(Clone, Debug)]
pub struct IfExpr<'a> {
    /// The condition after the if
    pub cond: syn::Expr,
    /// The content inside the if branch
    pub then: Vec<Item<'a>>,
    /// The closing of the if statement
    pub close: IfClose<'a>,
}

impl<'i> Parseable<'i> for IfExpr<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self, Error> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?; // remove whitespace
        let open: IfOpen = syn::parse_str(inner.as_ref())
            .map_err(|err| Error::from_syn(inner, err))
            .map_err(ErrFlow::Backtrack)?;
        let (input, (then, close)) =
            many_till(Item::parser(Ctx::deny()), IfClose::parse_next)(input)
                .map_err_incomplete(|e| e.span(inner).msg(EXPECT_END_MSG))
                .map_err(ErrFlow::to_unrecoverable)?;

        Ok((
            input,
            Self {
                cond: open.cond,
                then,
                close,
            },
        ))
    }
}

#[derive(Clone, Debug)]
struct IfOpen {
    #[allow(dead_code)]
    if_token: Token![if],
    pub cond: syn::Expr,
}

impl syn::parse::Parse for IfOpen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            if_token: input.parse()?,
            cond: input.parse()?,
        })
    }
}

#[derive(Clone, Debug)]
struct ElseIfOpen {
    #[allow(dead_code)]
    else_token: Token![else],
    #[allow(dead_code)]
    if_token: Token![if],
    pub cond: syn::Expr,
}

impl syn::parse::Parse for ElseIfOpen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            else_token: input.parse()?,
            if_token: input.parse()?,
            cond: input.parse()?,
        })
    }
}

/// All possible ways to close an [`IfExpr`]
///
/// It can just be directly closed with [`End`]
/// or there can be an else clause or multiple
/// else if clauses even possibly followed up with
/// an else clause after them.
#[derive(Clone, Debug)]
pub enum IfClose<'a> {
    /// Immediatly close the if expression
    Close,
    /// We reached an else clause this must be
    /// followed with an [`End`] expression
    Else {
        /// The template content within the else branch
        then: Vec<Item<'a>>,
    },
    /// An else if expression which could
    /// be followed with any of the other close statements
    ElseIf {
        /// The condition after the else if
        cond: syn::Expr,
        /// The template content within this branch
        then: Vec<Item<'a>>,
        /// The next part of the if expression
        close: Box<IfClose<'a>>,
    },
}

impl<'i> IfClose<'i> {
    pub(crate) fn into_owned(self) -> IfClose<'static> {
        match self {
            Self::Else { then } => IfClose::Else {
                then: then.into_iter().map(Item::into_owned).collect(),
            },
            Self::ElseIf { cond, then, close } => IfClose::ElseIf {
                cond,
                then: then.into_iter().map(Item::into_owned).collect(),
                close: Box::new(close.into_owned()),
            },
            Self::Close => IfClose::Close,
        }
    }
}

impl<'i> Parseable<'i> for IfClose<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        if let Ok((rem, _)) = End::parse_next(input) {
            return Ok((rem, Self::Close));
        }

        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?;
        let (else_if_inner, _) = tag("else")(inner)?; // make sure that we have an else expresion
        let (else_if_inner, _) = whitespace1(else_if_inner)?;

        if tag::<Error>("if")(else_if_inner).is_ok() {
            // else if expression
            let open: ElseIfOpen = syn::parse_str(inner.as_ref())
                .map_err(|err| Error::from_syn(inner, err))
                .map_err(ErrFlow::Unrecoverable)?;
            let (remaining, (then, close)) =
                many_till(Item::parser(Ctx::deny()), Self::parse_next)(input)
                    .map_err_incomplete(|e| e.span(inner).msg(EXPECT_END_MSG))
                    .map_err(ErrFlow::to_unrecoverable)?;
            Ok((
                remaining,
                Self::ElseIf {
                    cond: open.cond,
                    then,
                    close: Box::new(close),
                },
            ))
        } else {
            // standard else expression
            syn::parse_str::<Token![else]>(inner.as_ref())
                .map_err(|err| Error::from_syn(inner, err))
                .map_err(ErrFlow::Unrecoverable)?;
            let (remaining, (then, _)) =
                many_till(Item::parser(Ctx::deny()), End::parse_next)(input)
                    .map_err_incomplete(|e| e.span(inner))
                    .map_err(ErrFlow::to_unrecoverable)?;
            Ok((remaining, Self::Else { then }))
        }
    }
}

/// A match expression can be used to pattern match within the template
///
/// ```html
/// {% match my_result %}
///     {% when Ok(_) %}
///         Some template stuff
///     {% when Err(_) %}
///         Some error handling
/// {% end %}
/// ```
#[derive(Clone, Debug)]
pub struct MatchExpr<'a> {
    /// The expression to pattern match
    pub expr: syn::Expr,
    /// The arms of the expression
    pub arms: Vec<MatchArm<'a>>,
}

impl<'i> Parseable<'i> for MatchExpr<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;
        let (input, _) = whitespace0(input)?;

        let (inner, _) = whitespace0(inner)?; // remove whitespace
        let open: MatchOpen = syn::parse_str(inner.as_ref())
            .map_err(|err| Error::from_syn(inner, err))
            .map_err(ErrFlow::Backtrack)?;
        let (input, (arms, _)) = many_till(MatchArm::parse_next, End::parse_next)(input)
            .map_err_incomplete(|e| e.span(inner).msg(EXPECT_END_MSG))
            .map_err(ErrFlow::to_unrecoverable)?;

        Ok((
            input,
            Self {
                expr: open.expr,
                arms,
            },
        ))
    }
}

#[derive(Clone, Debug)]
struct MatchOpen {
    #[allow(dead_code)]
    match_token: Token![match],
    pub expr: syn::Expr,
}

impl syn::parse::Parse for MatchOpen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            match_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

/// A single match arm of a [`MatchExpr`]
#[derive(Clone, Debug)]
pub struct MatchArm<'a> {
    /// The pattern that this arm matches
    pub pat: syn::Pat,
    /// An optional if guard statement
    pub guard: Option<syn::Expr>,
    /// The content of the template for this arm
    pub content: Vec<Item<'a>>,
}

#[derive(Clone, Debug)]
struct MatchArmOpen {
    pat: syn::Pat,
    guard: Option<(Token![if], syn::Expr)>,
}

impl syn::parse::Parse for MatchArmOpen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pat = syn::Pat::parse_multi(input)?;
        if let Ok(if_token) = input.parse() {
            let expr = input.parse()?;
            Ok(Self {
                pat,
                guard: Some((if_token, expr)),
            })
        } else {
            Ok(Self {
                pat,
                guard: None,
            })
        }
    }
}

impl<'i> Parseable<'i> for MatchArm<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self, Error> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?;
        let (inner, _) = tag("when")(inner)?;

        let open: MatchArmOpen = syn::parse_str(inner.as_ref())
        //let pat = syn::parse::Parser::parse_str(syn::Pat::parse_multi, inner.as_ref())
            .map_err(|err| Error::from_syn(inner, err))
            .map_err(ErrFlow::Unrecoverable)?;

        let (remaining, content) = many_until(
            Item::parser(Ctx::deny()),
            alt((
                End::parse_next.map(|_| ()),
                MatchArm::parse_next.map(|_| ()),
            )),
        )(input)
        .map_err(ErrFlow::to_unrecoverable)?;

        Ok((remaining, Self {
            pat: open.pat,
            guard: open.guard.map(|g| g.1),
            content
        }))
    }
}

/// A macro expression
#[derive(Clone, Debug)]
pub struct MacroExpr<'a> {
    /// The name of the macro
    pub name: syn::Ident,
    /// The macro arguments
    pub args: Punctuated<syn::FnArg, Token![,]>,
    /// The content of the macro
    pub content: Vec<Item<'a>>,
}

impl<'i> Parseable<'i> for MacroExpr<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?;
        let (inner, _) = tag("macro")(inner)
            .map_err(ErrFlow::to_backtrack)?;
        let (inner, _) = whitespace1(inner)
            .map_err(|e| e.msg("at least one space required after macro keyword"))
            .map_err(ErrFlow::to_unrecoverable)?;

        let (args, (name, _)) = take_till(tag("("))(inner)
            .map_err(|e| e.msg("expected opening parenthesis for macro arguments"))
            .map_err(ErrFlow::to_unrecoverable)?;

        let name = syn::parse_str(name.as_ref())
            .map_err(|err| Error::from_syn(name, err))
            .map_err(ErrFlow::Unrecoverable)?;

        let (_, (args, _)) = take_till(tag(")"))(args)
            .map_err_incomplete(|e: Error| e.msg("expected ) to end list of arguments"))
            .map_err(ErrFlow::to_unrecoverable)?;

        let arg_parser = Punctuated::<syn::FnArg, Token![,]>::parse_terminated;
        let args = syn::parse::Parser::parse_str(arg_parser, args.as_ref())
            .map_err(|err| Error::from_syn(args, err))
            .map_err(ErrFlow::Unrecoverable)?;

        let (input, (content, _)) = many_till(Item::parser(Ctx::deny()), End::parse_next)(input)
            .map_err_incomplete(|e| e.span(inner).msg(EXPECT_END_MSG))
            .map_err(ErrFlow::to_unrecoverable)?;

        Ok((
            input,
            Self {
                name,
                args,
                content,
            },
        ))
    }
}

/// A macro call expression
#[derive(Clone, Debug)]
pub struct CallMacroExpr {
    /// The name of the macro to call
    pub name: syn::Ident,
    /// The arguments to the macro
    pub args: Punctuated<syn::Expr, Token![,]>,
}

struct CallMacroArgs {
    args: Punctuated<syn::Expr, Token![,]>,
}

impl syn::parse::Parse for CallMacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _paren = syn::parenthesized!(content in input);
        let args = Punctuated::parse_terminated(&content)?;
        Ok(Self {
            args,
        })
    }
}

impl<'i> Parseable<'i> for CallMacroExpr {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self, Error> {
        let (input, inner) = parse_delimited(["{%", "%}"])(input)?;

        let (inner, _) = whitespace0(inner)?;
        let (inner, _) = tag("call")(inner)
            .map_err(ErrFlow::to_backtrack)?;
        let (inner, _) = whitespace1(inner)
            .map_err(|e| e.msg("at least one space required after call keyword"))
            .map_err(ErrFlow::to_unrecoverable)?;

        let (args, name) = take_until(tag("("))(inner)
            .map_err(|e| e.msg("expected opening parenthesis for call arguments"))
            .map_err(ErrFlow::to_unrecoverable)?;

        let name = syn::parse_str(name.as_ref())
            .map_err(|err| Error::from_syn(name, err))
            .map_err(ErrFlow::Unrecoverable)?;

        let CallMacroArgs { args } = syn::parse_str(&args)
            .map_err(|err| Error::from_syn(args, err))
            .map_err(ErrFlow::Unrecoverable)?;

        Ok((input, Self { name, args }))
    }
}

/// The root of a template
/// just contains a list of [`Item`]s
#[derive(Clone, Debug)]
pub struct Root<'a> {
    /// The contents of the parsed template
    pub content: Vec<Item<'a>>,
}

impl<'i> Root<'i> {
    /// Convert the template into an owned version
    /// this will likely allocate several strings but
    /// probably won't be that expensive
    pub fn into_owned(self) -> Root<'static> {
        let content = self.content.into_iter().map(Item::into_owned).collect();
        Root { content }
    }
}

impl<'i> Root<'i> {
    /// Parse a template with an owned error type
    pub fn parse(input: &'i str) -> Result<Self, Error<'static>> {
        <Root as Parseable>::parse(input).map_err(Error::into_owned)
    }
}

impl<'i> Parseable<'i> for Root<'i> {
    fn parse_next(input: Located<'i>) -> LResult<'i, Self> {
        let (input, _) = whitespace0(input)?;
        let (input, first) = Item::parser(Ctx::allow())(input)?;
        let (input, (content, _)) = many_till(Item::parser(Ctx::block_only()), eof)(input)?;
        let mut v = Vec::with_capacity(content.len() + 1);
        v.push(first);
        v.extend(content);
        Ok((input, Self { content: v }))
    }
}
