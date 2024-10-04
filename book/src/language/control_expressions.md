# Control Expressions

Control *expressions* are those which can control the logic of how content is displayed within a template.
For instance conditionally rendering a section of a template depending on the value of a variable, or
rendering a section multiple times based on a list of items. For most control expressions they share
(structure/properties/token stream) with their rust counterpart.

## If
---

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
{% if show_link == "yes" %}
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
---

A *match* block is used in much the same way as an *if* block, but match blocks can be used to pattern match.

### TODO: add more detail on this
```html
{% match data %}
    {% when Some(value) if !value.is_empty() %}
        <a href="{% value.href %}">{% value.name %}</a>
    {% when Some(value) %}
        <button onclick="noValue">{% value.name %}</button>
    {% when None %}
        <span>No Data</span>
{% end %}
```

## For
---

The *for* block is an expression which is used to repeat parts of a template multiple times.

### TODO: add more detail on this
```stilts
<table>
{% for row in table %}
    <tr>
    {% for col in row %}
        <td>{% col %}</td>
    {% end %}
    </tr>
{% end %}
</table>
```

## Macro

The *macro* block is also used to repeat parts of a template but instead of in sequence the repetitions
can be controlled and have arguments.

### TODO: add more detail on this
```stilts
{% macro list_user(user: User) %}
<li>
    {% user.name %}
</li>
{% end %}

<ul>
{% for user in users %}
    {% call list_user(user) %}
{% end %}
</ul>
```
