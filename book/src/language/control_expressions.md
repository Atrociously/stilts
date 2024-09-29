# Control Expressions

Control *expressions* are those which can control the logic of how content is displayed within a template.
For instance conditionally rendering a section of a template depending on the value of a variable, or
rendering a section multiple times based on a list of items. For most control expressions they share
(structure/properties/token stream) with their rust counterpart.

## If

A Stilts if *block* can be used to change what parts of a template are rendered based on some value.

For example we can render a link only if some data is present to display the link.
```html
{% if data.is_some() %}
    <a href="{% data.unwrap().href %}">{% data.unwrap().name %}</a>
{% end %}
```

But having to unwrap multiple times is cumbersome, thankfully rust provides the [if let](https://doc.rust-lang.org/reference/expressions/if-expr.html#if-let-expressions)
system for that. Stilts if *blocks* are basically equivalent to standard rust if statements so any valid rust is valid in Stilts.
```html
{% if let Some(value) = data %}
    <a href="{% value.href %}">{% value.name %}</a>
{% end %}
```

Often it is useful to render something for multiple different cases for this you can use `else if` and `else`.
```html
{% if let Some(value) = data %}
    <a href="{% value.href %}">{% value.name %}</a>
{% else if let Some(value) = other %}
    <button onclick="{% value.clicked %}">{% value.name %}</button>
{% else %}
    <span>No Data</span>
{% end %}
```

## Match

Stilts is inspired by [Askama](https://github.com/djc/askama) which is based on [Jinja](https://jinja.palletsprojects.com/),
and something that Askama added that did not exist in Jinja is a match *block*. This was to aleviate dealing with rust
[enums](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html), this is a feature which Stilts improves upon.

```html
{% match data %}
    {% when  %}
{% end %}
```

## For
## Macro
