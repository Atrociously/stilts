# Configuration

When the derive macros are expanded Stilts will read some optional configuration values from
your `Cargo.toml`. It uses the `package.metadata` section to read config parameters.

```toml
[package.metadata.stilts]
template_dir = "$CARGO_MANIFEST_DIR/templates"
trim = false
writer_name = "_w"
```

Here is a list of all available configuration values and brief descriptions:

- **template_dir** The root directory of all your templates.
    - DEFAULT: `$CARGO_MANIFEST_DIR/templates`
- **trim** Whether when rendering text content if `trim()` should be called on it.
    - DEFAULT: `false`
- **writer_name** The identifier of the writer in the generated trait implementation.
    - DEFAULT: `_w`

## Crate Features

Stilts has a few features that can be optionally enabled.

- `err-narrate` Change the error messages to be optimized for narration
- `err-fancy` Change the error messages to be fancy!

All error message features are thanks to [miette](https://github.com/zkat/miette).
The default error messages are optimized for inline editor tips.
