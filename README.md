# Lox Tree-Walker Interpreter in Rust

My solution to ["Build your own interpreter" challenge on CodeCrafters](https://app.codecrafters.io/courses/interpreter/overview) in Rust that is based on [Crafting Interpreters](https://craftinginterpreters.com/) book by Robert Nystrom.

Features and highlights:

- Zero LLM-generated code.
- Pratt parser to parse expressions.
- Wrote additional tests more or less in TDD approach before commiting to CodeCrafters to confirm the stage completion.
- All 84 stages from CodeCrafters completed, including optional sections.
- Slow when using heavy recursive code, as expected.
- Instead of optimizing this version, I've started to work on bytecode version of the interpreter (second part of the book - clox) without using CodeCrafters - [lox-bytecode-vm-rs](https://github.com/artemudovyk/lox-bytecode-vm-rs).

The challenge is based on the first part of the book dedicated to jlox - Java implementation of the interpreter with all the baggage that comes with it. Following this solution using Rust could be problematic when linked lists and lifetimes involved, but it's more or less follows the same structure. The Pratt parser I've used here was introduced in the second part of the book (clox implementation), but I've researched it earlier and decided to use it for this implementation instead of using Visitor pattern as proposed in the book. Other than that, almost everything else follows the book, including the Environment linked list and Resolver for local bindings (the latter was frustrating to work with if I'm being honest, as I've felt it wasn't idiomatic to Rust to begin with along with the tree-walking approach, but I've decided to finish this initial implementation as the book suggests).

## Sources and inspirations

- [Crafting Interpreters](https://craftinginterpreters.com/) - the book, of course.
- [Jon Gjengset](https://github.com/codecrafters-io/build-your-own-interpreter) - used as an inspiration when I've started this challenge, but was very careful to not "spoil" the actual solution to the problem - it finishes pretty early in the challenge though, around parsing expressions and statements. It was the source that got me to research Pratt Parsing alghorithm. There is his video on YouTube [Implementing a Lox interpreter in Rust](https://www.youtube.com/watch?v=mNOLaw-_Buc), but I did not watch it for the reasons mentioned earlier.
- [Simple but Powerful Pratt Parsing](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) - main resource to understand and implement Pratt Parsing alghorithm in Rust.
- [Pratt Parsing](https://www.youtube.com/watch?v=2l1Si4gSb9A) YouTube video by Colin James - was very helpful to visualize it.
