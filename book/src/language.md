# Stilts Language Items

Stilts is a templating engine and as such supports many features
that popular template engines support. Here is where each feature
will be outlined and explained in detail with examples.

There are two major components that make up a template *content* and *expressions*.
Content is the static text that makes up the structure of the template. Expressions
are invocations to the template engine to perform some kind of manipulation of the source
text. This section covers all the kinds of *expressions* which can be used in Stilts.

All *expressions* in Stilts are direct one-to-one compatible with their rust counter parts.
This allows nearly unlimited freedom in how users can manipulate their templates.

In Stilts an *expression* is either a single piece of code surrounded by the delimiters `{%` and `%}` e.g.
```html
{% include "other.html" %}
```
Or it is a multi-expression *block* which has an opening and `end` *expression* e.g.
```html
{% if show_this %}
    <a></a>
{% end %}
```

Any item that requires an `end` *expression* will be referred to as a *block* in Stilts.

## Expression Table

Here is a quick reference to the different expressions Stilts has, if this is your first
reading you should just continue reading and not skip ahead, this is provided as a convenience.

- [Rust](./rust_expressions.md)
  - [Display](./rust_expressions.md#display)
  - [Statement](./rust_expressions.md#statement)
- [Control](./control_expressions.md)
  - [If](./control_expressions.md#if)
  - [Match](./control_expressions.md#match)
  - [For](./control_expressions.md#for)
  - [Macro](./control_expressions.md#macro)
- [Inheritance](./inheritance_expressions.md)
  - [Extends](./inheritance_expressions#extends)
  - [Block](./inheritance_expressions#block)
  - [Include](./inheritance_expressions#include)
