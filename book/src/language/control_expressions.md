# Control Expressions

Control *expressions* are those which can control the logic of how content is displayed within a template.
For instance conditionally rendering a section of a template depending on the value of a variable, or
rendering a section multiple times based on a list of items. For most control expressions they share
(structure/properties/token stream) with their rust counterpart.

## If
---

A Stilts *if* block can be used to change what parts of a template are rendered based on some value.

For example, we can render a link only if some data is present to display the link.
```html
{% if data.is_some() %}
    <a href="{% data.unwrap().href %}">{% data.unwrap().name %}</a>
{% end %}
```

Now depending on whether `data.is_some()` is `true` or `false` the template
will either render the stuff inside the if block or not.

But having to unwrap multiple times is cumbersome, thankfully rust provides the [if let](https://doc.rust-lang.org/reference/expressions/if-expr.html#if-let-expressions)
convention for that. Stilts *if* blocks are basically equivalent to standard rust if statements, so any valid rust is valid in Stilts.
```html
{% if let Some(value) = data %}
    <a href="{% value.href %}">{% value.name %}</a>
{% end %}
```

Often it is useful to render something for multiple different cases for this you can use `else if` and `else`.
```stilts
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

A *match* block is used in much the same way as an *if* block, but match blocks can be used to [pattern match](https://doc.rust-lang.org/book/ch18-00-patterns.html).
The match block is functionally equivalent to a rust [match](https://doc.rust-lang.org/book/ch06-02-match.html).

```stilts
{% match data %}
    {% when Some(value) if !value.is_empty() %}
        <a href="{% value.href %}">{% value.name %}</a>
    {% when Some(value) %}
        <button onclick="noValue">{% value.name %}</button>
    {% when None %}
        <span>No Data</span>
{% end %}
```

Just like their rust counterparts Stilts matches are exhaustive, meaning that all possible cases
must be covered by the match arms. If you need, you can use the wildcard catch-all to provide a "default" case.

```stilts
{% match data %}
    {% when Some() if !value.is_empty() %}
        <a href="{% value.href %}">{% value.name %}</a>
    {% when _ %}
{% end %}
```

## For
---

The *for* block is an expression which is used to repeat parts of a template multiple times. Again like the above blocks
it is the same as the rust equivalent [for loop](https://doc.rust-lang.org/book/ch03-05-control-flow.html?highlight=loop#looping-through-a-collection-with-for).

This will loop over the items in a collection and render the contents for each item in the collection.
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
---

The *macro* block defines a section of template code that can be manually called to render in multiple locations
and with different arguments. This is most useful for reducing code duplication within a template.

Macros are not similar to rust macros, instead they are more like functions which can take args and
will always output template code.

This is a simple example where we have to do the same thing twice but with two different sets of 
data, and in two different locations within the template. We could write the whole for loop twice, 
or we could use a macro!
```stilts
{% macro list_users(users: &[User]) %}
<ul>
    {% for user in users %}
    <li>
        {% user.name %}
    </li>
    {% end %}
</ul>
{% end %}

<div class="active">
    {% call list_users(active_users) %}
</div>
<div class="inactive">
    {% call list_users(inactive_users) %}
</div>
```
