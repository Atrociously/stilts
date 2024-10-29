//! Stilts Lang is the parser for the stilts language
//! Parse a full template with [`parse_template`] the
//! output is is an ast whose definition is in the [`types`] module

pub use error::Error;
pub use located::Located;
pub use state::Delims;
use state::State;
use types::Root;

pub(crate) type Input<'i> = winnow::Stateful<Located<'i>, State>;

mod error;
mod located;
mod parse;
mod state;
pub mod types;

/// Parse a template
///
/// The template lives as long as the input, if you want you can convert it into
/// an owned version using [`Root::into_owned`]. This is practically guaranteed to
/// do a bunch of allocations, but it really shouldn't take that long
pub fn parse_template(input: &str, delims: Delims) -> Result<Root<'_>, Error<'_>> {
    use winnow::Parser;
    let input = winnow::Stateful {
        input: Located::new(input),
        state: State::default(),
    };
    parse::root(&delims)
        .parse(input)
        .map_err(|err| err.into_inner())
}

#[cfg(test)]
mod test {
    use crate::{parse_template, types::{Expr, IfBranch, Item, ItemBlock, ItemFor, ItemIf, ItemMacro, ItemMatch, MatchArm, Root}, Delims};
    use pretty_assertions::assert_eq;
    use syn::{parse::Parser as _, punctuated::Punctuated};

    const TEMPLATE: &str = r###"
{% extends "base.html" %}

{% fn my_func(s: &str) -> String {
    let mut out = "OOF".to_string();
    out.push_str(s);
    out
} %}

{% macro my_mac(time: std::time::Duration) %}
    INSIDE MY MAC
{% end %}

{% block head %}
    {% a %}
    {% super() %}
    overwrites
{% end %}

{% block header %}
    {% for i in 0..10 %}
        {% match i %}
            {% when 2 if i != 0 %}
                {% i.json() %}
            {% when 3 | 4 %}
                {% a %}
            {% when _ %}
        {% end %}
    {% end %}
    {% if true %}
        {% a %}
    {% else %}
        {% a %}
    {% end %}
{% end %}

{% block main %}
    {% "Hello Word" %}
    {% include "other.html" %}
    {% a %}
{% end %}

{% block footer %}
    {% call my_mac(std::time::Duration::from_secs(50)) %}
    {% my_func(s) %}
{% end %}
"###;

    #[test]
    pub fn parse_example_template() {
        let res = parse_template(TEMPLATE, Delims::default()).unwrap();
        let expects = Root {
            content: vec![
                Item::Expr(Expr::Extends("base.html".into())),
                Item::Content("\n\n".into()),
                Item::Expr(Expr::Stmt(syn::parse_str(r#"fn my_func(s: &str) -> String {
                    let mut out = "OOF".to_string();
                    out.push_str(s);
                    out
                }"#).unwrap())),
                Item::Content("\n\n".into()),
                Item::Macro(ItemMacro {
                    name: syn::parse_str("my_mac").unwrap(),
                    args: Punctuated::parse_terminated.parse_str("time: std::time::Duration").unwrap(),
                    content: vec![Item::Content("\n    INSIDE MY MAC\n".into())],
                }),
                Item::Content("\n\n".into()),
                Item::Block(ItemBlock {
                    name: "head".into(),
                    content: vec![
                        Item::Content("\n    ".into()),
                        Item::Expr(Expr::Expr(syn::parse_str("a").unwrap())),
                        Item::Content("\n    ".into()),
                        Item::Expr(Expr::SuperCall),
                        Item::Content("\n    overwrites\n".into()),
                    ]
                }),
                Item::Content("\n\n".into()),
                Item::Block(ItemBlock {
                    name: "header".into(),
                    content: vec![
                        Item::Content("\n    ".into()),
                        Item::For(ItemFor {
                            label: None,
                            pat: syn::Pat::parse_single.parse_str("i").unwrap(),
                            expr: syn::parse_str("0..10").unwrap(),
                            content: vec![
                                Item::Content("\n        ".into()),
                                Item::Match(ItemMatch {
                                    expr: syn::parse_str("i").unwrap(),
                                    arms: vec![
                                        MatchArm {
                                            pat: syn::Pat::parse_multi.parse_str("2").unwrap(),
                                            guard: Some(syn::parse_str("i != 0").unwrap()),
                                            content: vec![
                                                Item::Content("\n                ".into()),
                                                Item::Expr(Expr::Expr(syn::parse_str("i.json()").unwrap())),
                                                Item::Content("\n            ".into()),
                                            ],
                                        },
                                        MatchArm {
                                            pat: syn::Pat::parse_multi.parse_str("3 | 4").unwrap(),
                                            guard: None,
                                            content: vec![
                                                Item::Content("\n                ".into()),
                                                Item::Expr(Expr::Expr(syn::parse_str("a").unwrap())),
                                                Item::Content("\n            ".into()),
                                            ],
                                        },
                                        MatchArm {
                                            pat: syn::Pat::parse_multi.parse_str("_").unwrap(),
                                            guard: None,
                                            content: vec![Item::Content("\n        ".into())],
                                        }
                                    ]
                                }),
                                Item::Content("\n    ".into()),
                            ],
                        }),
                        Item::Content("\n    ".into()),
                        Item::If(ItemIf {
                            cond: syn::parse_str("true").unwrap(),
                            content: vec![
                                Item::Content("\n        ".into()),
                                Item::Expr(Expr::Expr(syn::parse_str("a").unwrap())),
                                Item::Content("\n    ".into()),
                            ],
                            branch: IfBranch::Else {
                                content: vec![
                                    Item::Content("\n        ".into()),
                                    Item::Expr(Expr::Expr(syn::parse_str("a").unwrap())),
                                    Item::Content("\n    ".into()),
                                ],
                            },
                        }),
                        Item::Content("\n".into()),
                    ]
                }),
                Item::Content("\n\n".into()),
                Item::Block(ItemBlock {
                    name: "main".into(),
                    content: vec![
                        Item::Content("\n    ".into()),
                        Item::Expr(Expr::Expr(syn::parse_str(r#""Hello Word""#).unwrap())),
                        Item::Content("\n    ".into()),
                        Item::Expr(Expr::Include {
                            reference: "other.html".into(),
                            args: Punctuated::parse_terminated.parse_str("").unwrap()
                        }),
                        Item::Content("\n    ".into()),
                        Item::Expr(Expr::Expr(syn::parse_str("a").unwrap())),
                        Item::Content("\n".into())
                    ],
                }),
                Item::Content("\n\n".into()),
                Item::Block(ItemBlock {
                    name: "footer".into(),
                    content: vec![
                        Item::Content("\n    ".into()),
                        Item::Expr(Expr::MacroCall {
                            name: syn::parse_str("my_mac").unwrap(),
                            args: Punctuated::parse_terminated.parse_str("std::time::Duration::from_secs(50)").unwrap(),
                        }),
                        Item::Content("\n    ".into()),
                        Item::Expr(Expr::Expr(syn::parse_str("my_func(s)").unwrap())),
                        Item::Content("\n".into()),
                    ],
                }),
                Item::Content("\n".into()),
            ]
        };
        assert_eq!(res, expects);
    }
}
