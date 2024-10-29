use stilts::Template;

#[derive(Template)]
#[stilts(path = "custom_table.html")]
struct CustomTable;

#[test]
fn ensure_args_passthrough() {
    const EXPECTED: &str = r"<table><tr><td>1, 1</td><td>1, 2</td><td>1, 3</td></tr><tr><td>2, 1</td><td>2, 2</td><td>2, 3</td></tr><tr><td>3, 1</td><td>3, 2</td><td>3, 3</td></tr></table>";

    assert_eq!(CustomTable.render().unwrap(), EXPECTED);
}
