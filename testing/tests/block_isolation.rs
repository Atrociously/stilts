#![warn(clippy::pedantic)]

use stilts::Template;

#[derive(Template)]
#[stilts(path = "sample.html", block = "main")]
struct ChildTemplate<'a> {
    a: &'a str,
}

#[test]
fn ensure_child() {
    const EXPECTED: &str = r#"Hello Word<a href="/">MY MAN</a>my code content <a></a>"#;

    let val = ChildTemplate {
        a: "my code content <a></a>",
    }
    .render()
    .unwrap();

    assert_eq!(val, EXPECTED);
}

#[derive(Template)]
#[stilts(
    content = "<em> {% block main %} <strong> {% a %} </strong> {% end %} </em>",
    block = "main"
)]
struct SoloTemplate<'a> {
    a: &'a str,
}

#[test]
fn ensure_solo() {
    const EXPECTED: &str = r#"<strong>Hey</strong>"#;

    let val = SoloTemplate {
        a: "Hey",
    }
    .render()
    .unwrap();

    assert_eq!(val, EXPECTED);
}
