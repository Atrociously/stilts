# Stilts

Stilts the rust centric compile time type safe templating engine.

Stilts is a templating language inspired by [Askama](https://github.com/djc/askama).
It is still loosly related to [Jinja](https://jinja.palletsprojects.com), however it
has departed significantly in some ways due to the focus on rust as a language.

This project is very early on, I just started it because I liked askama but I felt that
it could be improved by allowing a more rust specific syntax in the template expressions.
If you have suggestions, features, questions, or anything else please feel free to open
an issue. I could especially use extra help writing tests for stuff, and am open to working
with other people to get that done.

I also got some of the project organization structure from looking at the Askama git.
Seriously go check out Askama, it is an amazing project and more mature than Stilts.

While the primary use case for Stilts is in html templates for websites, there are no rules
stating that you can't use it for general templating purposes. However, because html is it's
primary target use case it includes some html specific features. For instance stilts includes
an opt-out html escaping mechanism that works on files with a detected mime type of html. This
can be configured via a [config](./configuration.md) option, or directly within the macro attributes.

One more thing, while reading through the documentation please consider giving me feedback on ways to
improve it. This is the first time I have written serious documentation for library code that is
more than an internal or personal system.
