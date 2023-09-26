use stilts::Template;

#[derive(Template)]
#[stilts(path = "different_filetype.js")]
struct OtherTypeTemplate {
    my_rust_val: String,
}

#[derive(Template)]
#[stilts(path = "different_filetype.js", escape = stilts::escaping::Empty)]
struct OverrideTemplate {
    my_rust_val: String,
}

struct CustomEscaper;

impl stilts::escaping::Escaper for CustomEscaper {
    fn fmt<T: std::fmt::Display + ?Sized>(&self, _value: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CUSTOM")
    }
}

#[test]
fn ensure_escapes() {
    const EXPECTED: &str = r#"function my_js_func() {
    return "CUSTOM";
}"#;

    let val = OtherTypeTemplate {
        my_rust_val: "Hello, World!".to_string(),
    }.render().unwrap();

    assert_eq!(val, EXPECTED);
}

#[test]
fn ensure_override() {
    const EXPECTED: &str = r#"function my_js_func() {
    return "Hello, World!";
}"#;

    let val = OverrideTemplate {
        my_rust_val: "Hello, World!".to_string(),
    }.render().unwrap();

    assert_eq!(val, EXPECTED);
}
