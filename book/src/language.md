# Stilts Language

Stilts is a templating engine and as such supports many features
that popular template engines support. Here is where each feature
will be outlined and explained in detail with examples.

A template is made up of two components *content* and *expressions*.
Expressions are further divided into multiple categories but they either
control the logic of the template rendering or are replaced by values while rendering.
The content of a template is the static text that is manipulated by the engine
as defined by the expressions within that template.

All *expressions* in Stilts are directly comparable to their rust counter parts.
This allows nearly unlimited freedom in how users can manipulate their templates.

In Stilts an *expression* is either a single piece of code surrounded by the delimiters `{%` and `%}` e.g.
```stilts
{% include "other.html" %}
```

Or it is a multi-expression *block* which has an opening and *ending expression*
```stilts
{% if show_this %}
    <a></a>
{% end %}
```

Any item that requires an `{% end %}` *expression* will be referred to as a *block* in Stilts.

## Expression Table
---

Here is a quick reference to the different expressions Stilts has, if this is your first
reading you should just continue reading and not skip ahead, this is provided as a convenience.

- [Rust](./language/rust_expressions.md)
  - [Display](./language/rust_expressions.md#display)
  - [Statement](./language/rust_expressions.md#statement)
- [Control](./language/control_expressions.md)
  - [If](./language/control_expressions.md#if)
  - [Match](./language/control_expressions.md#match)
  - [For](./language/control_expressions.md#for)
  - [Macro](./language/control_expressions.md#macro)
- [Inheritance](./language/inheritance_expressions.md)
  - [Extends](./language/inheritance_expressions#extends)
  - [Block](./language/inheritance_expressions#block)
  - [Include](./language/inheritance_expressions#include)
