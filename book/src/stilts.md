# Stilts

Stilts is a templating engine and language that provides compile time correctness guarantees.

## About This "Book"

This documentation is here to help programmers take full advantage of all of the features
that Stilts provides. Stilts will be the only focus of this documentation, if explanation
of something falls out of the purview of the Stilts templating engine this document should
link to a proper explanation of that topic. The goal of this is to keep this book simple
and straightforward to read, there should be little to no fluff outside of this page.

### Assumtions About You, The Reader
- Knows what a [template engine](https://en.wikipedia.org/wiki/Template_processor) is
- Familiar with [Rust](https://www.rust-lang.org/)
  - If you aren't I would recommend reading [the book](https://doc.rust-lang.org/book/)

## Quick Sample

Here is a quick sample of some Stilts template and construction.

### template.html
```html
<div>
    {% for user in users %}
    <a href="{% user.profile.safe() %}">{% user.name %}</a>
    {% end %}
</div>
```

### template.rs
```rust
use stilts::Template;

#[derive(Template)]
#[stilts(path = "template.html")]
struct QuickExample {
    users: Vec<User>,
}

struct User {
    profile: Url,
    name: String,
}
```

## Use Cases

Stilts is primarily designed for templating html in web projects and has default settings
configured to cater to that use case. However it certainly can be used and modified to
serve any kind of templating purpose. As with any tool or library the design decisions
made in Stilts require making certain tradeoffs.

Stilts is designed for use in systems where templates are tightly coupled with code.
The most common case is web design, but there are other cases where this tight coupling
can be useful. For instance a project which generates some generic code based on a few
parameters. If the code generated follows a specific rigid structure and is tweaked
by some inputs, then Stilts can work well for that.

### Benefits

1. **Compile time guarantees**
   - Stilts generates rust code based off of the templates you write which
     is then ran through the rust compiler meaning you maintain all the guarantees
     provided by the rust compiler.
2. **Pure rust inside templates**
   - Stilts is focused on making development in rust as simple and flexible as possible.
     Therefore you are able to write arbitrary rust code anywhere inside your templates.
3. **Performant render times**
   - This while not a primary focus of the Stilts engine is a nice benefit you
     get when most of the work is done at compile time.

### Drawbacks

1. **No creation of templates on the fly at runtime**
   - This can be a big downside for many potential use cases. Many tools need to apply
     template rules to arbitrary text that is recieved at runtime, which Stilts
     simply cannot do due to it's very nature.
2. **Longer iteration times**
   - Iteration is important especially when working with UI/UX, so impairing
     iteration times can be a big problem for some people. Stilts impairs iteration
     times by forcing your entire application to recompile when minor template changes
     need to be made. This effect can however be [reduced](./iterating_reccomendation.md).
3. **Cross Language support**
   - Stilts is as rust first and only system. Similar projects could be made for other
     languages, but they would not follow the syntax or rules that rust enforces.
     As such if you are looking for a templating engine that can be used across multiple
     programming languages Stilts cannot fill that role. Look at something like
     [Jinja2](https://jinja.palletsprojects.com) 
     which has implementations in many languages with consistent syntax.

Stilts cannot perform runtime template creation / parsing, if you need that
you should look for some other engines [here](https://www.arewewebyet.org/topics/templating/).
Some notable runtime engines would include Tera, Handlebars, Liquid, and Minijinja.

## Important Mentions
The Stilts templating engine takes major inspiration from
[Askama](https://github.com/djc/askama). Askama is an older
library with more history and support behind it, which Stilts cannot claim to have.
However Stilts provides features that I believe are worth the change.

It took a lot of research on procedural macros to figure out
how to get this to work so big thanks to these resources.
- [Proc Macro Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Syn Documentation](https://docs.rs/syn/latest/syn/)
