//! The implementation of the parsing in this crate is meant to mostly be
//! unimportant to the utility of the crate. Therfore it may significantly change
//! if there are better ways to do it.
use std::marker::PhantomData;

use crate::{ErrFlow, Error, LResult, Located, ParseErr};

/// A generalized trait for defining a parser
///
/// I took inspiration from both nom and winnow
pub trait Parser<'i, T, E = Error<'i>>
where
    E: ParseErr<'i>,
{
    /// Attempt to parse the input into the desired output
    fn parse_next(&mut self, input: Located<'i>) -> LResult<'i, T, E>;

    /// A more suitable public facing parsing function
    fn parse(&mut self, input: &'i str) -> Result<T, E> {
        let input = Located::new(input);

        self.parse_next(input)
            .map(|(_, v)| v)
            .map_err(ErrFlow::into_err)
    }

    /// Map the successful result of one parser into something else
    fn map<O, F>(self, f: F) -> Map<Self, F, T>
    where
        Self: Sized,
        F: FnMut(T) -> O,
    {
        Map {
            parser: self,
            f,
            _marker: PhantomData,
        }
    }

    /// Run a function on the outputs of a parser and can be used to parse
    /// two things in succession
    fn and_then<O, F>(self, f: F) -> AndThen<Self, F, T>
    where
        Self: Sized,
        F: FnMut(Located<'i>, T) -> LResult<'i, O, E>,
    {
        AndThen {
            parser: self,
            f,
            _marker: PhantomData,
        }
    }
}

/// A utility struct for the [`Parser::map`] function
pub struct Map<P, F, T> {
    parser: P,
    f: F,
    _marker: PhantomData<T>,
}

impl<'i, P, F, T, O, E> Parser<'i, O, E> for Map<P, F, T>
where
    E: ParseErr<'i>,
    P: Parser<'i, T, E>,
    F: FnMut(T) -> O,
{
    fn parse_next(&mut self, input: Located<'i>) -> LResult<'i, O, E> {
        match self.parser.parse_next(input) {
            Ok((rem, v)) => Ok((rem, (self.f)(v))),
            Err(e) => Err(e),
        }
    }
}

/// A utility struct for the [`Parser::and_then`] function
pub struct AndThen<P, F, T> {
    parser: P,
    f: F,
    _marker: PhantomData<T>,
}

impl<'i, P, F, T, O, E> Parser<'i, O, E> for AndThen<P, F, T>
where
    E: ParseErr<'i>,
    P: Parser<'i, T, E>,
    F: FnMut(Located<'i>, T) -> LResult<'i, O, E>,
{
    fn parse_next(&mut self, input: Located<'i>) -> LResult<'i, O, E> {
        match self.parser.parse_next(input) {
            Ok((rem, v)) => (self.f)(rem, v),
            Err(e) => Err(e),
        }
    }
}

impl<'i, F, T, E> Parser<'i, T, E> for F
where
    E: ParseErr<'i>,
    F: FnMut(Located<'i>) -> LResult<'i, T, E>,
{
    fn parse_next(&mut self, input: Located<'i>) -> LResult<'i, T, E> {
        self(input)
    }
}

/// A trait to define a type that is parseable
pub trait Parseable<'i, E = Error<'i>>: Sized {
    /// Attempt to parse self from the input
    fn parse_next(input: Located<'i>) -> LResult<'i, Self, E>;

    /// A better public parsing function
    fn parse(input: &'i str) -> Result<Self, E> {
        Self::parse_next(Located::new(input))
            .map(|(_, v)| v)
            .map_err(ErrFlow::into_err)
    }
}

/// Parse the input pat as the next part of the input
pub fn tag<'i, E: ParseErr<'i>>(
    pat: impl AsRef<str> + 'i,
) -> impl FnMut(Located<'i>) -> LResult<'i, Located<'i>, E> + 'i {
    move |input: Located<'i>| {
        let pat = pat.as_ref();
        let pat_len = pat.len();

        match input.try_slice(..pat_len) {
            Some(next) if next.as_ref() == pat => Ok((input.slice(pat_len..), next)),
            Some(next) => Err(ErrFlow::Backtrack(E::new("unexpected value").span(next))),
            None => Err(ErrFlow::Incomplete(
                E::new("unexpected eof").span(input.slice(..0)),
            )),
        }
    }
}

/// Parse an optional amount of whitespace
pub fn whitespace0<'i, E: ParseErr<'i>>(input: Located<'i>) -> LResult<'i, (), E> {
    let offset = input
        .chars()
        .enumerate()
        .skip_while(|(_, ch)| ch.is_whitespace())
        .map(|(i, _)| i)
        .next()
        .unwrap_or(0);

    Ok((input.slice(offset..), ()))
}

/// Parse at least one char of whitespace then any more that follows
pub fn whitespace1<'i, E: ParseErr<'i>>(input: Located<'i>) -> LResult<'i, (), E> {
    let offset = input
        .chars()
        .enumerate()
        .take_while(|(_, ch)| ch.is_whitespace())
        .map(|(i, _)| i)
        .last()
        .ok_or_else(|| E::new("expected whitespace").span(input.slice(..0)))
        .map_err(ErrFlow::Backtrack)?;

    Ok((input.slice(offset + 1..), ()))
}

/// Parse any next char as valid
pub fn anychar<'i, E: ParseErr<'i>>(input: Located<'i>) -> LResult<'i, char, E> {
    // this is mostly used in testing functions
    input
        .chars()
        .next()
        .map(|ch| (input.slice(1..), ch))
        .ok_or(ErrFlow::Incomplete(
            E::new("unexpected eof").span(input.slice(..0)),
        ))
}

/// Parse the end of the file if there is still data then it will error
pub fn eof<'i, E: ParseErr<'i>>(input: Located<'i>) -> LResult<'i, (), E> {
    if input.chars().next().is_none() {
        Ok((input, ()))
    } else {
        Err(ErrFlow::Backtrack(E::new("not eof")))
    }
}

/// Parse f many times until the g parser succeeds
/// does not consume the input parsed by g
pub fn many_until<'i, F, G, E: ParseErr<'i>>(
    mut f: impl Parser<'i, F, E> + 'i,
    mut g: impl Parser<'i, G, E> + 'i,
) -> impl FnMut(Located<'i>) -> LResult<'i, Vec<F>, E> + 'i {
    move |input| {
        let mut part = input;
        let mut items = Vec::new();
        let mut res = g.parse_next(part);

        while let Err(ErrFlow::Backtrack(_)) = res {
            let (next, item) = f.parse_next(part)?;
            if part.offset() == next.offset() {
                return Err(ErrFlow::Backtrack(E::new(
                    "infinite loop in many_until f does not increment parser position",
                )));
            }
            part = next;
            items.push(item);
            res = g.parse_next(part);
        }

        match res {
            Ok(_) => Ok((part, items)),
            Err(e) => Err(e),
        }
    }
}

/// Parse f many times until the g parser succeeds
/// it will consume the input that g consumes and return
/// the value of both parsers
pub fn many_till<'i, F, G, E: ParseErr<'i>>(
    mut f: impl Parser<'i, F, E> + 'i,
    mut g: impl Parser<'i, G, E> + 'i,
) -> impl FnMut(Located<'i>) -> LResult<'i, (Vec<F>, G), E> + 'i {
    move |input| {
        let mut part = input;
        let mut items = Vec::new();
        let mut res = g.parse_next(part);

        while let Err(ErrFlow::Backtrack(_)) = res {
            let (next, item) = f.parse_next(part)?;
            if part.offset() == next.offset() {
                return Err(ErrFlow::Backtrack(
                    E::new("infinite loop in many_till f does not increment parser position")
                        .span(part.slice(..0)),
                ));
            }
            part = next;
            items.push(item);
            res = g.parse_next(part);
        }

        match res {
            Ok((rem, fin)) => Ok((rem, (items, fin))),
            Err(e) => Err(e),
        }
    }
}

/// Take the input until pat succeeds does not consume the recognized pat
pub fn take_until<'i, T, E: ParseErr<'i>>(
    mut pat: impl Parser<'i, T, E> + 'i,
) -> impl FnMut(Located<'i>) -> LResult<'i, Located<'i>, E> + 'i {
    move |input: Located<'i>| {
        let mut part = input;
        let mut res = pat.parse_next(part);
        while let Err(ErrFlow::Backtrack(_)) = res {
            part = part.try_slice(1..).ok_or_else(|| {
                ErrFlow::Incomplete(E::new("unexpected eof").span(part.slice(..0)))
            })?;
            res = pat.parse_next(part);
        }

        match res {
            Ok(_) => Ok((part, input.slice(..part.offset() - input.offset()))),
            Err(e) => Err(e),
        }
    }
}

/// Take the input until pat succeeds and consumes the recognized pat and returns the result
pub fn take_till<'i, T, E: ParseErr<'i>>(
    mut pat: impl Parser<'i, T, E> + 'i,
) -> impl FnMut(Located<'i>) -> LResult<'i, (Located<'i>, T), E> + 'i {
    move |input: Located<'i>| {
        let mut part = input;
        let mut res = pat.parse_next(part);
        while let Err(ErrFlow::Backtrack(_)) = res {
            part = part.try_slice(1..).ok_or_else(|| {
                ErrFlow::Incomplete(E::new("unexpected eof").span(part.slice(..0)))
            })?;
            res = pat.parse_next(part);
        }
        match res {
            Ok((rem, v)) => Ok((rem, (input.slice(..part.offset() - input.offset()), v))),
            Err(e) => Err(e),
        }
    }
}

/// Parse one of the input parsers in order given to the function
pub fn alt<'i, T, E: ParseErr<'i>>(
    mut list: impl Alt<'i, T, E> + 'i,
) -> impl FnMut(Located<'i>) -> LResult<'i, T, E> + 'i {
    move |input| list.choice(input)
}

/// The helper trait for the [`alt`] function
///
/// inspired by nom
pub trait Alt<'i, T, E: ParseErr<'i>> {
    /// Try to run each parser
    fn choice(&mut self, input: Located<'i>) -> LResult<'i, T, E>;
}

macro_rules! alt_impl {
    ($($gen:ident.$idx:tt)*; $fgen:ident.$fidx:tt) => {
        impl<'i, Output, Err, $($gen,)* $fgen> Alt<'i, Output, Err> for ($($gen,)* $fgen,)
        where
            Err: ParseErr<'i>,
            $($gen: Parser<'i, Output, Err>,)*
            $fgen: Parser<'i, Output, Err>,
        {
            fn choice(&mut self, input: Located<'i>) -> LResult<'i, Output, Err> {
                $(
                match self.$idx.parse_next(input) {
                    Err(ErrFlow::Backtrack(_)) => (),
                    o => return o,
                }
                )*

                // parse the final one without checking any backtracking
                self.$fidx.parse_next(input)
            }
        }
    };
}

alt_impl!(; A.0);
alt_impl!(A.0; B.1);
alt_impl!(A.0 B.1; C.2);
alt_impl!(A.0 B.1 C.2; D.3);
alt_impl!(A.0 B.1 C.2 D.3; E.4);
alt_impl!(A.0 B.1 C.2 D.3 E.4; F.5);
alt_impl!(A.0 B.1 C.2 D.3 E.4 F.5; G.6);
alt_impl!(A.0 B.1 C.2 D.3 E.4 F.5 G.6; H.7);
alt_impl!(A.0 B.1 C.2 D.3 E.4 F.5 G.6 H.7; I.8);

#[cfg(test)]
mod test {
    use crate::{parse::many_until, Error};

    use super::{
        anychar, many_till, tag, take_till, take_until, whitespace0, whitespace1, Located, Parser,
    };

    #[test]
    fn located_slice() {
        let s = Located::new("this text is good");

        let part = s.slice(0..4);
        assert!(part.as_ref() == "this");
        assert!(part.offset() == 0);
        assert!(part.len() == 4);

        let part = s.slice(5..9);
        assert!(part.as_ref() == "text");
        assert!(part.offset() == 5);
        assert!(part.len() == 4);

        let part = part.slice(1..=2);
        assert!(part.as_ref() == "ex");
        assert!(part.offset() == 6);
        assert!(part.len() == 2);

        let part = s.slice(1..);
        assert!(part.as_ref() == "his text is good");
        assert!(part.offset() == 1);
        assert!(part.len() == s.len() - 1);

        let part = s.slice(..2);
        assert!(part.as_ref() == "th");
        assert!(part.offset() == 0);
        assert!(part.len() == 2);
    }

    #[test]
    fn tag_parser() {
        let s = Located::new("my input");

        let (remaining, out) = tag::<Error>("my").parse_next(s).unwrap();
        assert!(out.as_ref() == "my");
        assert!(remaining.as_ref() == " input");
    }

    #[test]
    fn whitespace0_parser() {
        let s = Located::new("  s");
        let (remaining, _) = whitespace0::<Error>.parse_next(s).unwrap();
        assert!(remaining.as_ref() == "s");

        let s = Located::new("s");
        let (remaining, _) = whitespace0::<Error>(s).unwrap();
        assert!(remaining.as_ref() == "s");
    }

    #[test]
    fn whitespace1_parser() {
        let input = Located::new("   s");

        let (remaining, _) = whitespace1::<Error>(input).unwrap();
        assert!(remaining.as_ref() == "s");

        let input = Located::new("s ");
        whitespace1::<Error>(input).unwrap_err();

        let input = Located::new(" ");
        let (remaining, _) = whitespace1::<Error>(input).unwrap();
        assert!(remaining.as_ref() == "");
    }

    #[test]
    fn anychar_parser() {
        let input = Located::new("abcdef");

        let (remaining, ch) = anychar::<Error>(input).unwrap();
        assert!(remaining.as_ref() == "bcdef");
        assert!(ch == 'a');
    }

    #[test]
    fn take_until_parser() {
        let input = Located::new("abcdef");

        let (remaining, val) = take_until::<_, Error>(tag("d"))(input).unwrap();
        assert!(remaining.as_ref() == "def");
        assert!(val.as_ref() == "abc");

        let (remaining, val) = take_until::<_, Error>(tag("f"))(input).unwrap();
        assert!(remaining.as_ref() == "f");
        assert!(val.as_ref() == "abcde");

        let input = Located::new(" i %}");
        let (remaining, val) = take_until::<_, Error>(tag("%}"))(input).unwrap();
        assert!(remaining.as_ref() == "%}");
        assert!(val.as_ref() == " i ");
    }

    #[test]
    fn take_till_parser() {
        let input = Located::new("abcdef");

        let (remaining, (inner, tagg)) = take_till::<_, Error>(tag("d"))(input).unwrap();
        assert!(remaining.as_ref() == "ef");
        assert!(inner.as_ref() == "abc");
        assert!(tagg.as_ref() == "d");

        let (remaining, (inner, tagg)) = take_till::<_, Error>(tag("f"))(input).unwrap();
        assert!(remaining.as_ref() == "");
        assert!(inner.as_ref() == "abcde");
        assert!(tagg.as_ref() == "f");
    }

    #[test]
    fn many_until_parser() {
        let input = Located::new("abcdef");

        let (remaining, chars) = many_until(anychar::<Error>, tag("d"))(input).unwrap();
        assert!(remaining.as_ref() == "def");
        assert!(chars == vec!['a', 'b', 'c']);

        let (remaining, chars) = many_until(anychar::<Error>, tag("f"))(input).unwrap();
        assert!(remaining.as_ref() == "f");
        assert!(chars == vec!['a', 'b', 'c', 'd', 'e']);
    }

    #[test]
    fn many_till_parser() {
        let input = Located::new("abcdef");

        let (remaining, (chars, tagg)) = many_till(anychar::<Error>, tag("d"))(input).unwrap();
        assert!(remaining.as_ref() == "ef");
        assert!(chars == vec!['a', 'b', 'c']);
        assert!(tagg.as_ref() == "d");

        let (remaining, (chars, tagg)) = many_till(anychar::<Error>, tag("f"))(input).unwrap();
        assert!(remaining.as_ref() == "");
        assert!(chars == vec!['a', 'b', 'c', 'd', 'e']);
        assert!(tagg.as_ref() == "f");
    }
}
