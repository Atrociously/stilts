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

Currently the attribute only gets the path. When more options are
added in the future they will be expanded upon here.
