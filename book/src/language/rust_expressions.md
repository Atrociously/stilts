# Rust Expressions

## Display
---

A display expression is one that has already been shown earlier in the book and is the
simplest of all the expressions. It instructs the engine to write some variable data into
the template at a specific point.

```html
<span>{% my_variable %}</span>
```

This is the most simple case, however this *expression* is more flexible than just rendering
a variable name. It actually allows any arbitrary rust [**expression**](https://doc.rust-lang.org/reference/expressions.html) 
inside the delimiters. A rust **expression** is a separate concept from a Stilts *expression*.
A stilts *expression* only exists within the context of a template, and must be surrounded by the stilts delimiters.

For a display expression to be valid the rust expression within the delimiters must evaluate to a type which implements
[`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html). That is however the only limitation, there is
absolutely no limitation on syntax. For example inside the delimiters here is a fairly complex rust.

```html
<script>
    let data = {% my_data.iter().filter(|x| x.allowed).collect::<Vec<_>>().json() %};
</script>
```

As a general rule code between the delimiters is not required to be on a single line so the
previous example could just as easily be formatted as follows.

```html
<script>
    let data = {% my_data.iter()
        .filter(|x| x.allowed)
        .collect::<Vec<_>>()
        .json() %};
</script>
```

There is no rule on how to properly format template code, so that comes down to aesthetic preference.
The other thing these examples show off is the `json` function, this is one of a few convenience
functions that Stilts provides via ["Extension Traits"](../extension_traits.md).

## Statement
---

A statement *expression* is very similar to a display *expression* except that it does not
render anything to the template. It is rust code that gets run at that point during template
rendering but does not insert anything into the template. The way this works is by using
a rust [**statement**](https://doc.rust-lang.org/reference/statements.html) to distinguish whether
there is a value to be rendered or not.
In rust **expressions** must always produce a value, **statements** however produce no values.
This mechanism should be familiar to most rust programmers, as it is how `return` can be omitted
at the end of functions by just ending the function with an **expression**.

For example by simply adding a semicolon to the previous display expression it becomes a statement.
Doing this causes the value to **not** be rendered to the output.
```stilts
{% my_data.iter()
    .filter(|x| x.allowed)
    .collect::<Vec<_>>(); %}
```

Why would you want to write template expressions that neither render a value or affect the render logic?
Well the answer is variable declaration/modification and ["side effects"](https://en.wikipedia.org/wiki/Side_effect_(computer_science)).
If you need to introduce a variable for any reason you can do so using a statement. As for side effects
those are probably more rare than variable, but if some action needs to be performed without affecting
the template then use a **statement**.

In the following example we declare a mutable variable data using a statement,
then remove an element from the array without affecting the template by using another statement.
```stilts
{% let mut data = my_data.iter()
    .filter(|x| x.allowed)
    .collect::<Vec<_>>(); %}
<div>Some templatate content</div>
{% data.pop(); %}
<a>{% data.pop().unwrap().name %}</a>
```
