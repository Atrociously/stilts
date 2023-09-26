# Creating Templates

The way templates are created in your rust code is a concept
taken from how [Askama](https://github.com/djc/askama) does it.
It is a really clever way to write templates that have a signature
and are type safe.

If you have used Askama before then you will know how to use Stilts
within your rust code.

```rust
#[derive(Template)]
#[stilts(path = "index.html")]
struct Example {
    name: String,
}
```

This will generate a trait implementation for the struct `Example`.
`stilts()` is an attribute for the derive macro, and is used to
specify the path of the template.

Stilts now implements opt-out escaping and as such presents an
option for the macro to override the escaping implementation.

```rust
#[derive(Template)]
#[stilts(path = "index.html", escape = stilts::escaping::Empty)]
struct Example {

}
```
The above override would effectively disable escaping for the whole template.

If you have a custom file type that you want to have escaped you can implement
the `Escaper` trait and use the macro attribute or set a mime type in the config.
