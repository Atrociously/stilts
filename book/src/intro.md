# Stilts

_Stilts_ is a templating engine designed for the rust programming language that
makes use of rust's compile time to keep templates safe and correct.

## About This "Book"
---

This is a manual for using the _Stilts_ templating engine in code.
It will cover all the details necessary for a programmer to use and even
abuse the engine to accomplish their goals. This documentation will not cover
certain aspects of related materials when it would detract from the explanation
of a _Stilts_ concept. It will however link to reading on said related materials
whenever possible. Ideally this document can be understood by anybody even people who
fall outside the target audience.

### Target Audience
- A [Rust](https://www.rust-lang.org/) programmer.
  - If not you'll want to have [rust book](https://doc.rust-lang.org/stable/book/) as a reference.
- Needs a [Template Engine](https://en.wikipedia.org/wiki/Template_processor)
  usually to write HTML for a website, but Stilts works in other use cases as well.

## Use Cases
---

Stilts is primarily designed for templating HTML in web projects and has default settings
configured to cater to that use case. However, it certainly can be used and modified to
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
     is then run through the rust compiler meaning you maintain all the guarantees
     provided by the rust compiler.
2. **Pure rust inside templates**
   - Stilts is focused on making development in rust as simple and flexible as possible.
     Therefore, you are able to write arbitrary rust code anywhere inside your templates.
3. **Performant render times**
   - This, while not a primary focus of the Stilts engine, is a nice benefit you
     get when most of the work is done at compile time see [performance](./performance.md).

### Drawbacks

1. **No creation of templates on the fly at runtime**
   - This can be a big downside for many potential use cases. Many tools need to apply
     template rules to arbitrary text that is recieved at runtime, which Stilts
     simply cannot do due to its very nature.
2. **Longer iteration times**
   - Iterating on your design is important especially when working with UI/UX, so impairing
     iteration times can be a big problem for some people. Stilts impairs iteration
     times by forcing your entire application to recompile when minor template changes
     need to be made. It is however possible to [reduce iterating friction](./design_iteration.md).
3. **No Cross Language support**
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
---

The Stilts templating engine takes major inspiration from
[Askama](https://github.com/djc/askama). Askama is a more mature 
library with more history and support backing it, which Stilts does not have.
However, Stilts provides features that I believe are worth the change.

It took a lot of research on procedural macros to figure out
how to get this to work so big thanks to these resources.
- [Proc Macro Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Syn Documentation](https://docs.rs/syn/latest/syn/)
