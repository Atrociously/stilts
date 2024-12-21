# Inheritance Expressions
Many good templating engines have some form of inheritance. Or a method by which templates can be built upon
one another. Thus increasing reusability, and reducing repetition. Stilts implements a system based upon the
template engines that inspired it.

## Extends
---

The *extends* expression informs the Stilts engine that the current template **extends**
the functionality of another template. The current template becomes a "child" of the 
template in the *extends* expression. A child template is any template which invokes the *extends* expression
in order to inherit from a "parent" template. A parent template is any template which has child templates that extend
from it. A template can be at once a parent and a child template if both those conditions are met.

The **extending** of functionality in the simplest case means that the content of the current template
is added to the end of the parent template and that is how it gets rendered. It does however get more
complicated with *block* expressions.

### base.html
```stilts
Hello from the parent!
```

### child.html
```stilts
{% extends "base.html" %}

Hello from the child!
```

### Output
This is what the output of rendering the child template would look like
```
Hello from the parent!

Hello from the child!
```

## Block
---

Distinct from the concept of *block expressions* a stilts *inheritance block* is used to define secions of templates
which can both be overriden by a potential child template and overrides a potential parent.

The same expression is used in parent and child templates to perform their related tasks. On the parent
defining a block means providing a section that any child templates **can** override, while in the
child template defining a block means overridding the block that is defined by the parent.

A *block* works in conjunction with the *extends* expressions to provide an inheritance structure to reduce
template code duplication. This is best accomplished by writing most boilerplate into a base template that other
child templates are able to extend and overwrite pieces of to create their own functionality.

A parent/base template defines as many blocks as it wants wherever it wants. It can even put code
into those blocks to provide default data in case a child template does not override the block.

#### base.html
```stilts
<!DOCTYPE html>
<html lang="en">
    <head>
        {% block head %}
        <title>This is extensible!</title>
        {% end %}
    </head>
    <body>
        {% block body %}
        {% end %}
    </body>
</html>
```

The child template when defining the same blocks is now overriding the blocks as defined
by the parent template. This means the code inside the child blocks is **essentially** injected
into the parent at the block definition.

#### child.html
```stilts
{% extends "base.html" %}

{% block head %}
    {% super() %} <!-- Take note of this expression it is explained below -->
    <script>
    </script>
{% end %}

{% block body %}
    <button>Hello World!</button>
{% end %}
```

The `{% super() %}` expression is a special expression which
can only be used inside blocks which allows the child template to bring back the content
of the parent block. If *super* is not called the content within the block defined by the
parent is completely overriden by the child template.

This is what the output of rendering the child template would look like. Since the child
template used the *super* expression in the `head` block the content of the parent template
was preserved while rendering the child.

#### Output
```html
<!DOCTYPE html>
<html lang="en">
    <head>
        <title>This is extensible!</title>
        <script>
        </script>
    </head>
    <body>
        <button>Hello World!</button>
    </body>
</html>
```

### Partial Rendering

Another special feature of blocks within a template is their ability to be rendered independently.
Take the previous `base.html` as an example to declare that template in a rust app the code would
look like the following.

```rust,numbered
#[derive(Template)]
#[stilts(path = "base.html")]
struct BaseTemplate {}
```

However there is commonly a need to render pieces of a larger template as a component. Breaking
very small pieces of template out into another file can lead to lots of small files being difficult
to manage. So to aleviate this issue stilts allows defining templates which only render a single block.

```rust,numbered
#[derive(Template)]
#[stilts(path = "base.html", block = "body")]
struct BodyOnly {}
```

This technique is useful for partial updates of a webpage when smaller components
need to be re-rendered server side. For larger or more complex components used in 
multiple places the `include` expression should be preffered.

## Include
---

An *include* expression is used to add the content from another template into a template
at a specified point in the template.

For example say you have a base template that all other templates inherit from
you can still break out some bits into smaller chunks specified in other files.
In this example we include two other template files `header.html`, and `socials.html`.
The contents of those two files would be inserted at the invocation of *include*.
```stilts
<!DOCTYPE html>
<html lang="en">
    <head>
        <!--A bunch of metadata and script stuff-->
    </head>
    <body>
        <header>{% include "header.html" %}</header>
        <main>{% block main %}{% end %}</main>
        <footer>{% include "socials.html" %}</footer>
    </body>
</html>
```

A new feature that Stilts adds to include expressions is the ability to specify arguments.
By default, include expressions will drag in all the variables required by them into the base
template. You can however avoid this by setting the values inside the template.

The arguments are simply added on at the end between a pair of curly braces, and it follows the
rust struct literal syntax.

```stilts
<!DOCTYPE html>
<html lang="en">
    <head>
        <!--A bunch of metadata and script stuff-->
    </head>
    <body>
        <header>{% include "header.html" {
            links: &[
                ("Home", "/"),
                ("Social Media", "http://external.website")
            ],
            active: "/"
        } %}</header>
        <main>{% block main %}{% end %}</main>
        <footer>{% include "socials.html" %}</footer>
    </body>
</html>
```
