#![warn(clippy::pedantic)]

use stilts::Template;

#[derive(Template)]
#[stilts(path = "sample.html", block = "main")]
struct MyTemplate<'a> {
    a: &'a str,
}

#[test]
fn ensure_matches() {
    const EXPECTED: &str = r#"Hello Word<a href="/">MY MAN</a>my code content <a></a>"#;

    let val = MyTemplate {
        a: "my code content <a></a>",
    }
    .render()
    .unwrap();

    assert_eq!(val, EXPECTED);
}
