# Design Iteration

Design iteration is an important part of the design process. Friction in that process often
causes users to move on to other projects. This section will cover a few methods of reducing
development iteration time friction caused by Stilts.

This friction is unfortunately fundamental to how Stilts operates as an engine. The compile
time guarantees are provided by the rust compiler, meaning that whatever crate contains templates
must recompile when templates change.

The techniques for reducing this friction can be broadly categorized into 3 groups:
Live Reload, Change Watching, and Compilation Speed.

## Live Reload
A method of automatically refreshing changes on the frontend of a design for the designer to view when
changes are made to a code base. When using Stilts this methodology works much better when combined with a change watcher.

- [Tower Livereload](https://github.com/leotaku/tower-livereload) is a library which can be added to any web server that
  makes use of the [tower](https://docs.rs/tower/latest/tower/) ecosystem. It injects code to automatically refresh the
  browser when it detects the server go down and come back.

## Change Watching
A system that watches for file changes inside your project and automatically causes a recompilation
based on that.

- [Bacon](https://github.com/Canop/bacon) is a wonderful tool which watches for rust source code file changes. This requires
  some configuration to use in conjunction with Stilts. Namely, bacon must be told to also watch the `templates` directory,
  and to kill the running process and restart instead of wait and restart.

  You can configure this in a global config or at the project level in a file called `bacon.toml` but here is an example
  config that works for Stilts.
  ```toml
  [jobs.watch]
  command = ["cargo", "run"]
  on_change_strategy = "kill_then_restart"
  watch = ["templates/"]
  ```
  Then all you have to do is run `bacon watch` and code changes will automatically result in a recompilation and rerun.
- [Watchexec](https://github.com/watchexec/watchexec) is a great and fairly simple tool which watches files and runs a command
  when it detects changes. It can be used without configuration with a simple single command.
  ```shell
  watchexec -r -e rs,html,css,js cargo run
  ```
  Will watch for changes in files with the extensions: rs, HTML, CSS, or JS and run the command `cargo run` while
  restarting the existing process that was already running.
- [Cargo Watch](https://github.com/watchexec/cargo-watch) Is not recommended by the project author anymore due to lack of
  time to support the project, however it still works very well. It is the most straightforward to use as a single simple command
  with no special flags works out of the box for Stilts projects.
  ```shell
  cargo watch
  ```

## Compiliation Speed
Stilts requires a full recompilation of your source code anytime a change is made to your templates. This means that reducing
compilation times will increase the speed with which you can iterate on your designs. There are multiple methods of doing this
and many of them can be combined to add on top of eachother.

- **Break you code up into multiple crates.** One simple performance improvement can be to break code into multiple crates. There isn't
  an exact science to this, and it may not be a good idea for very simple projects. The reason this works however is that the rust
  compiler launches multiple threads to perform compilation in parallel, but the unit of compilation is the crate. Meaning that
  splitting code into multiple crates makes for better parallel compilation.
- **Use the [mold](https://github.com/rui314/mold) linker.** This is simply a tool switch from the default linker rust uses to another
  which performs the same task but faster. Linking is the final step in compilation, and any performance increase is welcome.
  The mold readme has a section on how to use it for rust.
- **Use the [rustc cranelift backend](https://github.com/rust-lang/rustc_codegen_cranelift).** This replaces another component in the 
  compilation process. This time it replaces the backend of the compiler which generates the low level objects that get linked together.
  Rust uses LLVM by default but cranelift can sometimes be faster. The downside of this currently is that it requires using nightly rust.
