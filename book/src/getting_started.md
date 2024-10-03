# Getting Started

This section will cover instructions for using Stilts as your templating engine.
These instructions are intended to provide more of an introduction into usage of Stilts,
rather than a strict set of rules which must be followed. When followed these steps will lead
to working code, but it is also important to play around and discover. One of the best learning
tools is discovery.

## How to Create Templates
---

### Requirements:
- [Rust Installed](https://www.rust-lang.org/tools/install)
  This includes access to the following commands:
  - cargo
- A text editor
  - (Optional) One that can be specialized for [coding rust](https://areweideyet.com/)
- Access to a command prompt or [terminal emulator](https://en.wikipedia.org/wiki/Terminal_emulator)
  - On Windows the default command prompt or powershell will work fine
  - On Linux most distributions provide a default terminal emulator
  - On macOS the terminal app will work

### Instructions:
1. **Create a new rust project.**
   Depending on what kinds of tools you have installed there are a few ways to create a new rust project,
   the most common is by [using cargo](https://doc.rust-lang.org/cargo/guide/creating-a-new-project.html).
   
   To create a project with cargo open your terminal emulator 
   Using the `cargo` tool create a new project for these instructions it will be called `hellostilts`.
   Run the following commands to create the project and enter the project directory
   ```shell
   cargo new hellostilts
   cd hellostilts
   ```
   This will create a new directory named `hellostilts` with contents that look like this:
   ```text,tree
   📁 hellostilts/
   ├── 📁 src/
   │   └── main.rs
   └── Cargo.toml
   ```

2. **Add Stilts as a Dependency.**
   In order to make use out of _Stilts_ you'll need to add it to your project dependencies.
   The simplest method is to once again use `cargo`
   ```shell
   cargo add stilts
   ```

3. **Create a Template Directory.**
   Inside the new project create a directory named `templates`, this is where
   our future template code will be created. You can do this via a file explorer
   or using the command:
   ```shell
   mkdir templates
   ```
   After creating the folder your project folder should look like this:
   ```text,tree
   📁 hellostilts/
   ├── 📁 src/
   │   └── main.rs
   ├── 📁 templates/
   └── Cargo.toml
   ```

4. **Write the Template Code.**
   Inside the newly created `templates` folder create and edit your first template file.
   It can be named anything but for these instructions it will be called `index.html`
   Write something like this into the file:
   ```stilts
   <ul>
   {% for name in names %}
       <li>Hello {% name %}!</li>
   {% end %}
   </ul>
   ```
   Finally your project directory should look like this which is all the required files:
   ```text,tree
   📁 hellostilts/
   ├── 📁 src/
   │   └── main.rs
   ├── 📁 templates/
   │   └── index.html
   └── Cargo.toml
   ```

5. **Write the Rust Code.**
   Now you have created a template that can be understood by the _Stilts_ engine.
   Next it just has to be used in code. In the `src/main.rs` file that was
   made when your project was created, write the following:
   ```rust
   use stilts::Template;

   #[derive(Template)]
   #[stilts(path = "index.html")]
   struct Index<'s> {
       names: Vec<&'s str>
   }

   fn main() {
       let template = Index {
           names: vec![
               "Jack",
               "Grant",
               "Amber",
               "Alex"
           ],
       };
       println!("{}", template.render().unwrap());
   }
   ``` 

6. **Run The Program.**
   You have almost rendered a template! The final step is to compile and run the
   program. Thanks to `cargo` it is a simple single step!
   ```shell
   cargo run
   ```
   Now the output of that program should look a little something like:
   ```html
   <ul>

       <li>Hello Jack!</li>

       <li>Hello Grant!</li>

       <li>Hello Amber!</li>

       <li>Hello Alex!</li>

   </ul>
   ```
