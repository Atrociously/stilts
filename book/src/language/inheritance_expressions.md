# Inheritance Expressions
Many good templating engines have some form of inheritance. Or a method by which templates can be built upon
one another. Thus increasing reusability, and reducing repetition. Stilts implements a system based upon the
template engines that inspired it.

## Extends
---
An extends *expression* is how a template asserts that it is based upon another template 
it *extends* the *base* templates content with it's own. Without the usage of *blocks* the
content of the child template is simply added to the end of the parent template.

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
which can be overriden by child templates.

### base.html
A parent/base template defines as many blocks as it wants, these blocks define overwriteable sections of template
that child templates can overwrite to inject code into the context provided by the parent template.

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

### child.html

You'll notice in this example the `{% super() %}` this is a special expression which
can only be used inside blocks which allows the child template to bring back the content
of the parent block. If *super* is not called the content within the block defined by the
parent is completely overriden by the child template.
```stilts
{% extends "base.html" %}

{% block head %}
    {% super() %}
    <script>
    </script>
{% end %}

{% block body %}
    <button>Hello World!</button>
{% end %}
```

### Output
This is what the output of rendering the child template would look like
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

## Include
---

An include expression is used to add the content from another template into a template
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
