use std::borrow::Cow;
use std::sync::LazyLock;

use aho_corasick::AhoCorasick;
use winnow::ascii::{multispace0, space0, space1, take_escaped};
use winnow::combinator::{alt, cut_err, eof, opt, peek, preceded, repeat, repeat_till, terminated, trace};
use winnow::error::ParserError;
use winnow::stream::{AsChar, Compare, Stream, StreamIsPartial};
use winnow::token::{any, none_of, one_of, take, take_until, take_while};
use winnow::Parser;

use crate::error::{expect_end, At, Msg};
use crate::state::State;
use crate::types::{
    Expr, ForExpr, IfBranch, Item, ItemBlock, ItemFor, ItemIf, ItemMacro, ItemMatch,
    MacroCallExpr, MacroExpr, MatchArm, MatchArmExpr, Root,
};
use crate::{state::Delims, Input};
use crate::{Error, Located};

type PResult<'i, T> = winnow::PResult<T, Error<'i>>;

pub fn root<'i>(delims: &Delims) -> impl FnMut(&mut Input<'i>) -> PResult<'i, Root<'i>> {
    let delims = delims.clone();
    move |input| {
        let mut first = opt(preceded(multispace0, item(&delims))).parse_next(input)?;
        input.state &= !State::ALLOW_EXTEND; // Do not allow extend after first item!
        
        let mut content = repeat(0.., item(&delims))
            .fold(move || first.take().map(|v| vec![v]).unwrap_or_default(), |mut acc, item| {
                acc.push(item);
                acc
            })
            .parse_next(input)?;
        if let Some(final_content) = opt(repeat_till(1.., any, eof).map(|((), _)| ()).take()).parse_next(input)? {
            content.push(Item::Content(final_content.content().into()));
        }
        Ok(Root { content })
    }
}

pub fn item<'i>(delims: &Delims) -> impl Parser<Input<'i>, Item<'i>, Error<'i>> {
    trace(
        "item",
        alt((
            item_expr(delims).map(Item::Expr),
            item_block(delims).map(Item::Block),
            item_for(delims).map(Item::For),
            item_if(delims).map(Item::If),
            item_match(delims).map(Item::Match),
            item_macro(delims).map(Item::Macro),
            item_content(delims).map(Item::Content),
        )),
    )
}

pub fn item_content<'i>(
    delims: &Delims,
) -> impl Parser<Input<'i>, Cow<'i, str>, Error<'i>> {
    let delims = delims.clone();
    move |input: &mut Input<'i>| {
        trace(
            "content",
            take_until(1.., delims.open())
                .map(|l: Located<'_>| Cow::Borrowed(l.content())),
        )
        .parse_next(input)
    }
}

pub fn item_block<'i>(delims: &Delims) -> impl Parser<Input<'i>, ItemBlock<'i>, Error<'i>> {
    let delims = delims.clone();
    trace("block", move |input: &mut Input<'i>| {
        let (name, span) = delimited(
            &delims,
            preceded(("block", cut_err(space1)), cut_err(ident)),
        )
        .with_taken()
        .context(Msg("unable to parse block expression"))
        .context(At(input.here()))
        .parse_next(input)?;

        let saved = input.state;
        input.state |= State::ALLOW_SUPERCALL;
        //input.state &= !State::ALLOW_BLOCK;

        let content = cut_err(items_till(&delims, end(&delims)))
            .map(|v| v.0)
            .parse_next(input)
            .map_err(expect_end(span))?;
        input.state = saved;
        Ok(ItemBlock {
            name: Cow::Borrowed(name.content()),
            content,
        })
    })
}

pub fn item_for<'i>(delims: &Delims) -> impl Parser<Input<'i>, ItemFor<'i>, Error<'i>> {
    let delims = delims.clone();
    trace("for", move |input: &mut _| {
        let (open, span) = delimited(
            &delims,
            preceded(peek(("for", space1)), cut_err(parse_syn::<ForExpr>)),
        )
        .with_taken()
        .parse_next(input)?;

        let content = cut_err(items_till(&delims, end(&delims)))
            .map(|v| v.0)
            .parse_next(input)
            .map_err(expect_end(span))?;
        Ok(ItemFor {
            label: open.label,
            pat: open.pat,
            expr: open.expr,
            content,
        })
    })
}

pub fn item_if<'i>(delims: &Delims) -> impl Parser<Input<'i>, ItemIf<'i>, Error<'i>> {
    let delims = delims.clone();
    trace("if", move |input: &mut _| {
        let (cond, span) = delimited(
            &delims,
            preceded(("if", space1), cut_err(parse_syn::<syn::Expr>)),
        )
        .with_taken()
        .parse_next(input)?;
        let (content, branch) = items_till(&delims, if_branch(&delims))
            .parse_next(input)
            .map_err(expect_end(span))?;
        Ok(ItemIf {
            cond,
            content,
            branch,
        })
    })
}

fn if_branch<'i>(delims: &Delims) -> impl Parser<Input<'i>, IfBranch<'i>, Error<'i>> {
    let delims1 = delims.clone();
    let delims2 = delims.clone();
    trace(
        "branch",
        alt((
            move |input: &mut _| {
                let (cond, span) = delimited(
                    &delims1,
                    preceded(("else", space1, "if", space1), cut_err(parse_syn)),
                )
                .with_taken()
                .parse_next(input)?;
                let (content, branch) = items_till(&delims1, if_branch(&delims1))
                    .parse_next(input)
                    .map_err(expect_end(span))?;
                Ok(IfBranch::ElseIf {
                    cond,
                    content,
                    branch: Box::new(branch),
                })
            },
            move |input: &mut _| {
                let span = delimited(&delims2, ("else", space0))
                    .take()
                    .parse_next(input)?;
                let content = items_till(&delims2, end(&delims2))
                    .map(|v| v.0)
                    .parse_next(input)
                    .map_err(expect_end(span))?;
                Ok(IfBranch::Else { content })
            },
            end(delims).map(|_| IfBranch::End),
        )),
    )
}

pub fn item_match<'i>(delims: &Delims) -> impl Parser<Input<'i>, ItemMatch<'i>, Error<'i>> {
    let delims = delims.clone();
    trace("match", move |input: &mut _| {
        let delims2 = delims.clone();
        let (expr, span) = terminated(
            delimited(&delims, preceded(("match", space1), cut_err(parse_syn))),
            multispace0,
        )
        .with_taken()
        .parse_next(input)?;
        let arms = repeat_till(
            0..,
            trace("arm", move |input: &mut _| {
                let (arm, span) = delimited(
                    &delims2,
                    preceded(("when", space1), cut_err(parse_syn::<MatchArmExpr>)),
                )
                .with_taken()
                .parse_next(input)?;

                let content = items_till(
                    &delims2,
                    peek(alt((match_arm_test(&delims2), end(&delims2)))),
                )
                .map(|v| v.0)
                .parse_next(input)
                .map_err(expect_end(span))?;
                Ok(MatchArm {
                    pat: arm.pat,
                    guard: arm.guard.map(|v| v.1),
                    content,
                })
            }),
            end(&delims),
        )
        .map(|(arms, _)| arms)
        .parse_next(input)
        .map_err(expect_end(span))?;
        Ok(ItemMatch { expr, arms })
    })
}

fn match_arm_test<'i>(delims: &Delims) -> impl Parser<Input<'i>, (), Error<'i>> {
    delimited(delims, ("when", space1)).void()
}

pub fn item_macro<'i>(delims: &Delims) -> impl Parser<Input<'i>, ItemMacro<'i>, Error<'i>> {
    let delims = delims.clone();
    trace(
        "macro",
        move |input: &mut _| {
            let (mcr, span) = delimited(
                &delims,
                preceded(("macro", space1), cut_err(parse_syn::<MacroExpr>)),
            ).with_taken().parse_next(input)?;
            let content = items_till(&delims, end(&delims)).map(|v| v.0).parse_next(input)
                .map_err(expect_end(span))?;
            Ok(ItemMacro {
                name: mcr.name,
                args: mcr.args,
                content
            })
        }
    )
}

pub fn item_expr<'i>(delims: &Delims) -> impl Parser<Input<'i>, Expr<'i>, Error<'i>> {
    fn expr_extends<'i>(input: &mut Located<'i>) -> PResult<'i, Cow<'i, str>> {
        preceded(("extends", cut_err(space1)), cut_err(string_contents))
            .context(Msg("unable to parse extends expression"))
            .context(At(input.here()))
            .parse_next(input)
    }
    fn expr_include<'i>(input: &mut Located<'i>) -> PResult<'i, Cow<'i, str>> {
        preceded(("include", cut_err(space1)), cut_err(string_contents))
            .context(Msg("unable to parse include expression"))
            .context(At(input.here()))
            .parse_next(input)
    }

    fn expr_super_call<'i>(input: &mut Located<'i>) -> PResult<'i, ()> {
        ("super", "(", space0, ")", space0).void()
            .parse_next(input)
    }

    fn expr_macro_call<'i>(input: &mut Located<'i>) -> PResult<'i, MacroCallExpr> {
        preceded(("call", space1), cut_err(parse_syn)).parse_next(input)
    }

    trace("expr", delimited(
        delims,
        alt((
            expr_extends.map(Expr::Extends),
            expr_include.map(Expr::Include),
            expr_super_call.map(|_| Expr::SuperCall),
            expr_macro_call.map(|call| Expr::MacroCall {
                name: call.name,
                args: call.args,
            }),
            parse_syn.map(Expr::Stmt),
            parse_syn.map(Expr::Expr),
        )),
    ))
}

fn parse_syn<'i, T>(input: &mut Located<'i>) -> PResult<'i, T>
where
    T: syn::parse::Parse,
{
    syn::parse_str(input.content()).map_err(|e| Error::from_syn(*input, e).backtrack())
}

fn items_till<'i, P, O>(
    delims: &Delims,
    mut terminate: P,
) -> impl FnMut(&mut Input<'i>) -> PResult<'i, (Vec<Item<'i>>, O)>
where
    P: Parser<Input<'i>, O, Error<'i>>,
{
    let delims = delims.clone();
    move |input| repeat_till(0.., item(&delims), terminate.by_ref()).parse_next(input)
}

fn end<'i>(delims: &Delims) -> impl Parser<Input<'i>, (), Error<'i>> {
    delimited(delims, "end".void())
}

pub fn ident<'i, I, E>(input: &mut I) -> winnow::PResult<<I as Stream>::Slice, E>
where
    I: Stream<Token = char> + StreamIsPartial + 'i,
    E: ParserError<I>,
{
    (
        one_of(|c: char| c.is_alpha() || c == '_'),
        take_while(0.., |c: char| c.is_alphanum() || c == '_'),
    )
        .take()
        .parse_next(input)
}

pub fn string_contents<'i, I, E>(input: &mut I) -> winnow::PResult<Cow<'i, str>, E>
where
    I: Stream<Token = char> + StreamIsPartial + Compare<char> + 'i,
    I::Slice: AsRef<Located<'i>> + Stream + StreamIsPartial + Compare<char>,
    <I::Slice as Stream>::Token: AsChar + Copy,
    E: ParserError<I> + ParserError<I::Slice>,
{
    static AHO: LazyLock<AhoCorasick> =
        LazyLock::new(|| AhoCorasick::new(["\\\"", "\\\\"]).unwrap());
    const CHARS: [char; 2] = ['"', '\\'];
    let s = preceded(
        '"',
        take_escaped(take(1u8).and_then(none_of(CHARS)), '\\', one_of(CHARS)),
    )
    .parse_next(input)?;

    let value = maybe_replace(&AHO, s.as_ref().content(), &["\"", "\\"]);
    Ok(value)
}

/// Parse some content between a pair of delimiters provided by [Delims]
pub fn delimited<I, O, E, P>(
    delims: &Delims,
    mut parser: P,
) -> impl FnMut(&mut I) -> winnow::PResult<O, E>
where
    I: Stream
        + StreamIsPartial
        + Compare<char>
        + for<'a> Compare<&'a str>
        + for<'a> winnow::stream::FindSlice<&'a str>,
    I::Token: AsChar,
    E: ParserError<I>,
    P: Parser<<I as Stream>::Slice, O, E>,
{
    let delims = delims.clone();
    move |input| {
        (delims.open(), space0).parse_next(input)?;
        let mut content = take_until(1.., delims.close()).parse_next(input)?;
        delims.close().parse_next(input)?;
        parser.parse_next(&mut content)
    }
}

fn maybe_replace<'h, B: AsRef<str>>(aho: &AhoCorasick, haystack: &'h str, replace_with: &[B]) -> Cow<'h, str> {
    if aho.is_match(haystack) {
        Cow::Owned(aho.replace_all(haystack, replace_with))
    } else {
        Cow::Borrowed(haystack)
    }
}
