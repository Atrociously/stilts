# Extension Traits

Extension traits are an existing concept in rust used to add functionality to types.
Stilts defines a few extension traits which are imported into the template rendering scope automatically.

Currently there are three traits exposing 5 methods which can be used to change how a variable
is rendered. You can view the [trait docs](https://docs.rs/stilts/latest/stilts/#traits) to see
how the traits are defined and implemented, but this page will cover the basics of how to use them.

## DebugExt
This trait is implemented for any type that implements [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html).
It adds a method `debug` which makes stilts render the type using it's `Debug` implementation instead
of it's [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) implementation which is the default.

### Example
```stilts
{% name.debug() %}
```

## DisplayExt
This is implemented on any type that implements [`Display`]
