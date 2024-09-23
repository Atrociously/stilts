# TODO: lots more work on the book incoming

The config is set in your project `Cargo.toml` file
below is the default config.

## Cargo.toml
```toml
[package.metadata.stilts]
template_dir = "$CARGO_MANIFEST_DIR/templates"
trim = false
delimiters = ["{%", "%}"]
writer_name = "_w"

# This table defines a relationship between
# escapers and file extensions
# All file extensions can only have one escaper
# that escaper can be overridden
[package.metadata.stilts.escape]
"::stilts::escaping::Html" = ["html", "htm"]
```
