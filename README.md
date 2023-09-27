# Stilts

[![Documentation](https://docs.rs/stilts/badge.svg)](https://docs.rs/stilts)
[![Package](https://img.shields.io/crates/v/stilts.svg)](https://crates.io/crates/stilts)

Stilts is a templating language inspired by [Askama](https://github.com/djc/askama).
That means it features all the type safety the rust compiler will give you.
It is still loosly related to [Jinja](https://jinja.palletsprojects.com), however it
has adopted a more rust-like expression syntax.

This project is very early on, I just started it because I liked askama but I felt that
it could be improved by allowing a more rust specific syntax in the template expressions.
If you have suggestions, features, questions, or anything else please feel free to open
an issue. I could especially use extra help writing unit tests for stuff, and would be
open to working with others to get that done.

### Guide
Check out the [book](https://atrociously.github.io/stilts) for in depth documentation.

### How it works
Stilts uses a procedural derive macro on a struct to generate template rendering code
which results in the template code being checked by the rust compiler for correctness.

#### A Quick Example
Here is what some rust code defining a template looks like

```rust
use stilts::Template;

#[derive(Template)] // This derive macro will generate the template rendering code
#[stilts(path = "index.html")] // based on the contents of the given template path
struct IndexPage<'a> {
    some_data: &'a str, // The fields on a struct are the variables 
}                       // you want to use in your template
```

Here is what `index.html` could look like
```html
<!DOCTYPE html>
<html>
    <body>
        <h1>My Index Page</h1>
        {% some_data %} <!--This will print some_data to the template here using 
                            the types implementaion of the Display trait-->
    </body>
</html
```

#### About Stilts Expressions

In Stilts an expression is made up of two variants
`single` and `block` expressions. A single expression is
anything inside the delimiters `{%%}`. Whereas a block
expression has an opening single expression and a closing
single expression with some kind of content in-between. For example:

```
{% for i in 0..10 %}
    This will be repeated 10 times
    {% i %}
    And I can put other expressions inside
{% end %}
```

Something you might notice if you have used Askama or Jinja before
is that normally to render the value of something you have to use
a different set of delimiters namely something like `{{ some_data }}`.
But here in Stilts there is only one set of delimiters `{% some_data %}`.

So how do we determine when the user wants to render something or just
write some code? Well the answer is by determining if the rust code inside
the delimiters is an [expression](https://doc.rust-lang.org/reference/expressions.html)
or a [statement](https://doc.rust-lang.org/reference/statements.html). In rust an expression always
produces a value and a statement doesn't.

Therefore if as a user we want to write some code but not render it to the template
all we have to do is add a semicolon. `{% some_data; %}` is a statement now not an
expression which means Stilts will insert that into the template rendering code as code
to run and not render it to the resulting template. This is familiar to many rust developers
as it is the same way we can omit the `return` keyword inside of functions by just ending it
with an expression.

What this also means is that you can put any arbitrary rust expression or statement inside of the
delimiters. For example `{% let myval = some_data.split(' ').filter(|s| s != "abcd"); %}`.
But you aren't limited to a single line either you can split them into as many lines as you want.
```
{% fn my_useful_func() {
    // do some stuff
} %}
```

### Features
- The syntax is similar to Jinja but also more closely tied to rust
- Utilize rust's type system to verify your code
- Achieves performance comparable to Askama which is already very good
- Good error messages and formatting especially with optional fancy feature
- Works on stable rust

### Supported Template Constructs
- Inheritance
- for loops
- if/ else if /else
- match expressions
- includes statements
- variables (as a result of allowing arbitrary rust expressions and statements)
- useful extension traits imported into scope
- Configurable Opt-out HTML escaping

### Goals
- Create a templating language that is both familiar to jinja users and rust developers
- Have good error reporting and messages with useful error locations.

### Other Stuff
In the tooling directory there is a work-in-progress tree-sitter parser implementation.
If you know better how to get that working that would be super cool.
