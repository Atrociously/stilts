# The Template Macro

In previous instructions the rust code made use of the template [derive macro](https://doc.rust-lang.org/book/ch19-06-macros.html).
Invoking this macro is how our template code gets compiled into our rust code.
This macro performs a few different things depending on what options are provided,
however the core function is converting template code into an [`Template`](https://docs.rs/stilts/latest/stilts/trait.Template.html)
trait implementation.

In the instructions the `path` argument was used to load a template from a file.
This is the most common way of defining templates. Arguments are provided using
the same macro syntax and the `stilts` prefix followed by the args to provide.
```rust
#[derive(Template)]            // Use the derive macro
#[stilts(path = "index.html")] // Provides arguments to the derive macro
struct Example;                // The item which the trait is implemented on
```

## Macro Arguments
---

The template derive macro has multiple arguments which can be used to tweak
how the macro generates the template code. Some of the arguments are used
to override behavior described in the [configuration](./configuration.md) section.

Either **path** or **content** must be specified
- **path**: The path relative to the template root of the template to render
- **content**: The direct contents of the template provided by a string literal
- **escape**: Override the escaper detected by file extension with a specified one
- **trim**: Override the trim behavior defined in your config

### Examples:
Standard use case
```rust,numbered
#[derive(Template)]
#[stilts(path = "index.html")]
struct MyTemplate {
    my_data: String,
}
```

Using content instead of path
```rust,numbered
# use stilts::Template;
#[derive(Template)]
#[stilts(content = "My {% data %} Template")]
struct MyInlineTemplate {
    data: String,
}
```

An example of setting the trim and escape to something else. This forces
Stilts to not trim whitespace around expressions, and to use the [`Empty`](https://docs.rs/stilts/latest/stilts/escaping/struct.Empty.html)
escaper which does no escaping at all.
```rust,numbered
# use stilts::Template;
#[derive(Template)]
#[stilts(
    content = "Templates are fun",
    trim = false,
    escape = ::stilts::escaping::Empty
)]
struct MyOverridenTemplate {
    my_data: String,
}
```
