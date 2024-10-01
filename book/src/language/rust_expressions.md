# Rust Expressions

## Display

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

The other thing these examples show off is the `json` function, this is one of a few convenience
functions that Stilts provides via "Extension Traits", these are explained more [here](./extension_traits.md).

## Statement

A statement *expression* is very similar to a display *expression* except that it does not
render anything to the template. It is rust code that gets run at that point during template
rendering but does not insert anything into the template. The way this works is by using
a rust [**statement**](https://doc.rust-lang.org/reference/statements.html) to distinguish whether
there is a value to be rendered or not.
In rust **expressions** must always produce a value, **statements** however produce no values.
This mechanism should be familiar to most rust programmers, as it is how `return` can be omitted
at the end of functions by just ending the function with an **expression**.

```stilts
{% let mut data = my_data.iter()
    .filter(|x| x.allowed)
    .collect::<Vec<_>>(); %}
```

The difference between this example and the previous examples are that in previous examples
we were rendering the data into a json array that was inserted into a javascript context. In
this example the `let` is completely inside the delimiters, meaning that we are declaring a variable
in rust which can be used later inside the template.

```stilts
{% let mut data = my_data.iter()
    .filter(|x| x.allowed)
    .collect::<Vec<_>>(); %}
<div>Some templatate content</div>
<a>{% data.pop().unwrap().name %}</a>
```

### A peek under the hood

To completely understand what is happening here this section will take a peek under the hood
and to see what Stilts generates from a template like this. The code here will be simplified
and omit things to get the general point across simply.

```rust
impl Template for MyTemplate {
    fn fmt(&self, writer: &mut impl Writer) -> fmt::Result<()> {
        // Destructures into individual fields as variables
        let Self {
            my_data,
        } = self;
        // Statement expressions get translated into pure rust statements
        let mut data = my_data
            .filter(|x| x.allowed)
            .collect::<Vec<_>>();
        // Template content gets written to the writer between expressions
        writer.write_str("<div>Some template content</div>\n<a>")?;
        // Display expressions get translated into write calls using the types Display implementation
        write!(writer, "{}", data.pop().unwrap().name)?;
        writer.write_str("</a>")?;
    }
}
```
