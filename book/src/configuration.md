# Configuration

Stilts can be configured to meet many potential requirements. Stilts is
configured via your projects existing `Cargo.toml` file which already is
your rust config. All configuration is done using [`package.metadata`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table)
which is a cargo feature that allows Stilts to define a custom configuration
within your existing cargo project. To modify the configuration set values
in the `package.metadata.stilts` field in your project config, e.g.

```toml
# Cargo.toml
[package.metadata.stilts]
trim = true
```


Here is a list of configuration options, what they do, and their defaults:
- **template_dir**: Sets the root directory that Stilts looks in to find your templates.
  > Default: "$CARGO_MANIFEST_DIR/templates"
- **trim**: Trims whitespace from the beginning and end of each piece of template content
  in between expressions.
  > Default: false
- **delimiters**: Sets what delimiters Stilts uses when parsing templates.
  > Default: ["{%", "%}"]
- **writer_name**: Sets the name of the variable used when generating the template rendering code.
  > Default: "_w"
- **escape**: A table of paths to types that implement [`Escaper`](https://docs.rs/stilts/latest/stilts/escaping/trait.Escaper.html),
  and the list of file extensions which that implementation will be applied to.
  > Default: { "::stilts::escaping::Html" = ["html", "htm"] }


So the default configuration would look like this in the context of a full `Cargo.toml` file.
```toml
[package.metadata.stilts]
template_dir = "$CARGO_MANIFEST_DIR/templates"
trim = false
delimiters = ["{%", "%}"]
writer_name = "_w"

[package.metadata.stilts.escape]
"::stilts::escaping::Html" = ["html", "htm"]
```

## Escaping
---

Stilts implements an opt-out escaping scheme for templates. By default the only escaping mechanism
is for html files, which is the major use case for Stilts. Custom schemes can be added to the configuration
as seen above. Stilts also provides a method of excluding whole templates and single display expressions from
being escaped if so desired.

The html escaping follows OWASP standards of replacing the following characters with safe versions: `&`, `<`, `>`, `"`, `'`, `/`

The above configuration section shows how users can add escapers to the opt-out system of stilts
but it does not describe how to actually implement an escaper. Below is a custom implementation that
replaces a curse word with stars. This is meant only as an example of how to create a custom escaper.

```rust,numbered
use std::fmt::{self, Display};
use stilts::escaping::Escaper;

struct HorrificSwear;
impl Escaper for HorrificSwear {
    fn fmt<T: Display + ?Sized>(
        &self,
        value: &T,
        f: &mut fmt::Formatter<'_>
    ) -> fmt::Result {
        let s = value.to_string();
        let safe = s.replace("heck", "**ck") // clearly much better
        f.write_str(&safe)
    }
}
```

Once you have that done simply add it to your config as follows to 
make it operate on all of the file extensions listed for its entry.

```toml
[package.metadata.stilts.escape]
"::my_crate::HorrificSwear" = ["txt", "md"]
```
