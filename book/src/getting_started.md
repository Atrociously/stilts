# Getting Started

To get started with Stilts you will need to add the dependency to
your project either with `cargo add stilts` or editing your `Cargo.toml`.

## Your first template

By default Stilts looks for template files in a directory named `templates`
relative to your project root. This can be [configured](./configuration.md)
if that is not your desired behavior.

1. Create a file in the `templates` directory named `example.txt`.
2. Write the following into `templates/example.txt`
   ```html
   Hello from {% name %}!
   ```
   The usage of the `{%` and `%}` delimiters are how we can interact with the
   Stilts engine. There are many more uses for them but in this instance we are
   telling Stilts to *render* the variable `name` into the spot where this is invoked.
3. In your rust project `main.rs` write the following
   ```rust
   use stilts::Template;

   // In Stilts templates are defined on structs
   // the fields on said structs are then the parameters
   // that are available to the template when rendering
   #[derive(Template)]
   #[stilts(path = "example.txt")] // notice I don't specify templates/example.txt
   struct MyFirstTemplate {
       // This is the name variable referenced in example.txt
       name: &'static str,
   }

   fn main() {
       // construct an instance of the template with specified arguments
       let template = MyFirstTemplate { name: "Stilts" };
       // render the template into a string
       let output = template.render().unwrap();
       assert_eq!(output, "Hello from Stilts!".to_string());
   }
   ```

   The comments in the code highlights a few important details, but it is worthwhile to
   go into a little more depth. The struct field `name` is the variable referenced by
   the template itself. This is the mechanism by which you inject data into the template
   at runtime. You can add as many fields of any type you wish to the struct.
