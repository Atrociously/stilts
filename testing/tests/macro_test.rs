#![warn(clippy::pedantic)]

use stilts::Template;

#[derive(Template)]
#[stilts(path = "sample.html")]
struct MyTemplate<'a> {
    a: &'a str,
}

#[test]
fn print() {
    println!(
        "{}",
        MyTemplate {
            a: "my code content<a></a>",
        }
        .render()
        .unwrap()
    );
}
