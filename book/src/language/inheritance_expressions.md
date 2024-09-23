# Inheritance Expressions
Many good templating engines have some form of inheritance. Or a method by which templates can be built upon
one another. Thus increasing reusability, and reducing repetition. Stilts implements a system based upon the
template engines that inspired it.

## Extends
An extends *expression* is how a template asserts that it is based upon another template 
it *extends* the *base* template's functionality in some way.

### base.html
```html
<!DOCTYPE html>
<html lang="en">
    <head>
        <!--A bunch of metadata stuff-->
        {% block head %}
        {% end %}
    </head>
    <body>
        {% block body %}
        {% end %}
    </body>
</html>
```

### child.html
```html
{% extends "base.html" %}

{% block head %}
    <script>
    </script>
{% end %}

{% block body %}
    <button>Hello World!</button>
{% end %}
```

## Block

## Include
