# Stilts

Stilts is a templating language inspired by [Askama](https://github.com/djc/askama).
That means it features all the type safety the rust compiler will give you.
It is still loosly related to [Jinja](https://jinja.palletsprojects.com), however it
has adopted a more rust-like expression syntax.

This project is very early on, I just started it because I liked askama but I felt that
it could be improved by allowing a more rust specific syntax in the template expressions.
If you have suggestions, features, questions, or anything else please feel free to open
an issue. I could especially use extra help writing unit tests for stuff, and would be
open to working with others to get that done.

I also got some of the project organization structure from looking at the Askama git.
Seriously go check out Askama, it is a seriously cool project and much more mature
than Stilts at the moment.

While the primary use case for Stilts is in html templates for websites, there are no rules
stating that you can't use it for general templating purposes.
