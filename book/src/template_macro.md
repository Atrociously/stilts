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
#[derive(Template)]
#[stilts(path = "index.html")]
struct Example;
```

## Macro Arguments



This generates an implementation of the `Template` trait which uses
the contents of the `index.html` file to determine how it is rendered.
There is however one more method of providing template content to the
macro. This is by providing the `content` attribute instead of a `path`.

```rust
#[derive(Template)]
#[stilts(content = "Hello from {% name %}!")]
struct MyTemplate {
    name: String
}
```

The essential process of the macro is to read in the template content
either from a file or directly from the code and translate that into
an implementation of the `Template` trait, which is used to then render
the template into a string at runtime. Stilts also provides a few integrations
with popular rust web libraries to ease the usage of templates in web code.

## Individual Sanitization

One final important feature of the template macro is that you can
customize how sanitization works on a per-template basis. 
The sanitization mechanism itself is covered in depth [here](./sanitization.md).

```rust
#[derive(Template)]
#[stilts(
    path = "index.html",
    escape = stilts::escaping::Empty,
)]
struct Example;
```

This instructs the macro to code utilizing a specific method
of sanitization. In this specific example it uses `Empty` which
is defined by stilts to do zero escaping of any kind. By default
stilts sanitizes for html.
