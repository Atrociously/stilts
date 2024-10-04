# Inheritance Expressions
Many good templating engines have some form of inheritance. Or a method by which templates can be built upon
one another. Thus increasing reusability, and reducing repetition. Stilts implements a system based upon the
template engines that inspired it.

## Extends
An extends *expression* is how a template asserts that it is based upon another template 
it *extends* the *base* template's functionality in some way.

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
```
Hello from the parent!

Hello from the child!
```

## Block

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
TODO
