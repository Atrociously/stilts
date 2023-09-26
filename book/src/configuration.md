# Configuration

When the derive macros are expanded Stilts will read some optional configuration values from
your `Cargo.toml`. It uses the `package.metadata` section to read config parameters.

```toml
[package.metadata.stilts]
template_dir = "$CARGO_MANIFEST_DIR/templates"
trim = false
writer_name = "_w"

# the escape table
escape = { html = "::stilts::escaping::Html" }
# can also be defined like this
[package.metadata.stilts.escape]
js = "::my_crate::MyEscaper"
ts = "::my_crate::MyOtherEscaper"
```

Here is a list of all available configuration values and brief descriptions:

- **template_dir** The root directory of all your templates.
    - DEFAULT: `$CARGO_MANIFEST_DIR/templates`
- **trim** Whether when rendering text content if `trim()` should be called on it.
    - DEFAULT: `false`
- **writer_name** The identifier of the writer in the generated trait implementation.
    - DEFAULT: `_w`
- **escape** A table of file extensions to rust path specifiers which give the desired `Escaper` implementation
    - DEFAULT: `{ html = "::stilts::escaping::Html", htm = "::stilts::escaping::Html" }`

The escape table configuration extends or overrides the default so if you want to disable html
escaping you'll have to override the html/htm extension with `::stilts::escaping::Empty`.

## Crate Features

Stilts has a few features that can be optionally enabled.

- `err-narrate` Change the error messages to be optimized for narration! This is a very awesome feature from miette
- `err-fancy` Change the error messages to be fancy! These are great in the command line or while using a tool like [bacon](https://github.com/Canop/bacon)

All error message features are thanks to [miette](https://github.com/zkat/miette).
The default error messages are optimized for inline editor tooltips.
