# Getting Started

Make sure to add `stilts` to your dependencies

```toml
[dependencies]
stilts = "0.1.0"
```

By default your project templates will be searched
for in a directory named `templates` in your crate root.


### Your first template
In the `templates` directory create a file and name it anything you like
but I will use `index.html`

```html
Hello {% name %}!
```

Then in your crate add the following rust code

```rust
use stilts::Template;

#[derive(Template)]
#[template(path = "index.html")] // replace with the name of your file if you changed it
struct MyFirstTemplate<'a> { // The struct name does not matter and can have generics
    name: &'a str, // fields on this struct can be used in your template
}

fn main() {
    let template = MyFirstTemplate {
        name: "World"
    }; // create a new instance of your template
    println!("{}", template.render().unwrap()); // render your template
}
```

This code should compile and run
