#![warn(clippy::pedantic)]

use stilts::Template;

#[derive(Template)]
#[stilts(path = "sample.html")]
struct MyTemplate<'a> {
    a: &'a str,
}

#[derive(Template)]
#[stilts(content = "Literal {% a %} Template", trim = false)]
struct LitTemplate<'a> {
    a: &'a str,
}

#[test]
fn ensure_matches() {
    const EXPECTED: &str = r#"<!DOCTYPE html>
<html><head>i have stuffoverwrites</head>
    <body>
        <header>2my code content &lt;a&gt;&lt;&#x2F;a&gt;my code content &lt;a&gt;&lt;&#x2F;a&gt;</header>
        <main>Hello Word<a href="/">MY MAN</a>my code content <a></a></main>
        <footer>INSIDE MY MACOOFMYSTR</footer>
    </body>
</html>"#;

    let val = MyTemplate {
        a: "my code content <a></a>",
    }
    .render()
    .unwrap();

    assert_eq!(val, EXPECTED);
}

#[test]
fn ensure_lit_matches() {
    const EXPECTED: &str = "Literal cool Template";

    let val = LitTemplate { a: "cool" }.render().unwrap();

    assert_eq!(val, EXPECTED);
}
