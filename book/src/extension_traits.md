# Extension Traits

Extension traits are an existing concept in rust used to add functionality to types.
Stilts defines a few extension traits which are imported into the template rendering scope automatically.

Currently, there are three traits exposing 5 methods which can be used to change how a variable
is rendered. You can view the [trait docs](https://docs.rs/stilts/latest/stilts/#traits) to see
how the traits are defined and implemented, but this page will cover the basics of how to use them.

## DebugExt
---

This trait is implemented for any type that implements [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html).
It adds a method `debug` which makes stilts render the type using its `Debug` implementation instead
of it's [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) implementation which is the default.

### Example
```stilts
{% name.debug() %}
```

## DisplayExt
---

This is implemented on any type that implements [`Display`] and is provides multiple functions.

The functions currently provided by this trait are:
- `safe` Which marks the value as safe to render without running through a sanitizer.
- `lowercase` Changes the output of the type to all lowercase.
- `uppercase` Changes the output of the type to all uppercase.

> **Warning** Only use the `safe` function on data that is verifiably HTML safe.
> Not following this rule opens you up to [XSS](https://owasp.org/www-community/attacks/xss/) attacks!
> Anything involving user input is an example of where you want to be very careful using `safe`.

## SerializeExt
---

This only provides one function, and it is implemented on any type which implements
[serde](https://github.com/serde-rs/serde) [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html).
It adds the `json` function which converts the type into a [JSON](https://en.wikipedia.org/wiki/JSON)
string. This is most useful for adding data to a javascript script inside the template.

### Example
One thing with the default escaping scheme you will usually also have to mark the JSON output
as safe so that the quotation marks don't get replaced with `&quot;`. Be sure to only do this
if you can trust the data! Most data submitted by a user should not be marked as safe unless
it has already also been processed and made safe.
```stilts
<script>
    const DATA = {% my_template_data.json().safe() %};
</script>
```
