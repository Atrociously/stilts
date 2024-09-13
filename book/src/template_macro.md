# The Template Macro

In previous examples we see usage of the template derive macro.
This macro invocation is how templates are declared in Stilts.
The previous examples showed templates rendered from external files.

```rust
#[derive(Template)]
#[stilts(path = "index.html")]
struct Example;
```

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
