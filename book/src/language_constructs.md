# Language Constructs

The Stilts language has a few built in expressions that can be used
to create more expressive and useful templates.

## Control Structures

Control structures are expressions that control whether or not certain
parts of a template are rendered at runtime.

### If

If expressions are the same as rust if expressions except that they don't contain a block.
Instead they have a block of template content.
```html
<div>
    {% if user.logged_in() %}
        <a href="/account/{% user.name %}">Account</a>
    {% else %}
        <a href="/login">Login</a>
    {% end %}
</div>
```

Notice that since you can also just write rust expressions you can write a regular if statement.
This rule follows for all other control structures in Stilts.

```html
<div>
    {% _ = if user.logged_in() { user.name } else { "Anonymous" } %}
</div>
```

Woah what is that? Well currently in order for the parser to recognize that
if statement as an expression instead of a statement you have to add the `_ = `.
What this does is that it will render either `user.name` or `"Anonymous"` to
the template.

### For

For loops are useful for repeating a section of a template a number of times.
```html
<ul>
    {% for follower in user.followers %}
        <li>{% follower.name %}</li>
    {% end %}
</ul>
```

Stilts for loops are just rust for loops and expand into such so anything
that is valid in rust is valid in Stilts.

```html
<ul>
    {% 'outer for follower in user.followers %}
    <li>
        <ul>
            {% 'inner for post in follower.posts %}
            {% if post.title == "Hello World" { break 'outer; } %}
            {% end %}
        </ul>
    </li>
    {% end %}
</ul>
```

That loop would expand into something similar to
with some formatting code in there to write the template
contents:
```rust
'outer for follower in user.followers {
    // write text content
    'inner for post in follower.posts {
        if post.title == "Hello World" { break 'outer; }
    }
    // write text content
}
```

### Match

A match expression is useful quite often in rust to 
match patterns. In Stilts the syntax is a little different
but has all the same power.

```html
<div>
    {% match user.role %}
        {% when Role::Admin if user.name == "Jack" %}
            <a href="/admin">Admin Panel</a>
        {% when Role::Standard | Role::None %}
            <!-- Do nothing -->
    {% end %}
</div>
```

The main difference between this and a regular rust match
is the use of a `when` keyword. Here is how this gets expanded:

```rust
match user.role {
    Role::Admin if user.name == "Jack" => {
        // write text content
    },
    Role::Standard | Role::None => {},
}
```

## Macros

Macros aren't really control structures and are unrelated to inheritance, but are
a language feature that allows users to define a section of a template that can be
called multiple times.

They actually expand into rust functions that can take arguments.

```html
{% macro heading(arg: &str) %}
<h1>{% arg %}</h1>
{% end %}

{% call heading("My Heading") %}
{% call heading(s) %}
```

This would expand into something like:
```rust
fn heading(_w: &mut (impl Write + ?Sized), arg: &str) -> Result {
    _w.write_str("<h1>")?;
    write!(_w, "{}", arg);
    _w.write_str("</h1>")?;
}

heading(_w, "My Heading")?;
heading(_w, s)?;
```

It will use the same name as your configured [writer_name](./configuration.md)

## Inheritance

If you have used Jinja before you know that it has a system of inheritance to
help reduce rewriting boilerplate. A similar system is implemented in Askama,
and now here is Stilts.

### Extends

The extends expr tag is only allowed at the very beggining of a file, leading whitespace
is allowed but no other content.

```html
{% extends "base.html" %}

<!-- All of "base.html" is rendered first then the content of this template -->
```

The file name is always relative to the template root directory.

### Block

Due to how extends works, rendering the parent template first then the child,
blocks are a useful structure to allow child templates to override the contents
of a parent.

**parent.html:**
```html
<!DOCTYPE html>
<html>
    <head>
        {% block head %}
        <script>
            // some script
        </script>
        {% end %}
    </head>
    <body>
        {% block body %}
        Hello World!
        {% end %}
    </body>
</html>
```

**child.html:**
```html
{% extends "parent.html" %}

{% block head %}
    {{ super() }} <!-- This is another expr tag that is only allowed within blocks -->
    <script>
        // child script
    </script>
{% end %}

{% block body %}
    I overwrite what the parent had here
{% end %}
```

This combo will result in a final render for child.html that looks like:

```html
<!DOCTYPE html>
<html>
    <head>
        <script>
            // some script
        </script>
        <script>
        // child script
        </script>
    </head>
    <body>
        I overwrite what the parent had here
    </body>
</html>
```

When a child defines a block with the same name as a parent block it will
override the parent and render that instead. However if you want to preserve
what the parent had and just add on, you can use the `super()` expr tag.
It can even be used as many times as you want and it will repeatedly add the
parent content.

### Include

Includes can be used to pull in the entire contents of another template into
this template at a specified point.

**incuded.html:**
```html
<a>My Link</a>
```

**main.html:**
```html
{% for _ in 0..10 }
    {% include "included.html" %}
{% end %}
```

This would repeat the contents of the included
html 10 times.
