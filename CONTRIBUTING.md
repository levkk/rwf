# Contribution guidelines

All contributions are welcome. No minimal level of knowledge is required to participate in the project, including knowledge of the Rust programming language or web development experience.

If you are not sure about building a feature or fixing a bug, file a GitHub issue to discuss. Otherwise, unsolicited PRs are welcome.

## Coding guidelines

Before submitting a PR, please:

1. Format the code with `cargo fmt`
2. Write a test if your feature is not trivial
3. Test your feature/bug fix by compiling your code and running code which uses your bug fix/feature

Thank you! Happy coding.

## Philosophy

Rwf prioritizes user ergonomics over _bleeding_ edge runtime performance. Rust already runs at native speed (think x86/ARM instructions, not an interpreter with a slow GC), and most code will fetch bytes from the network, so a few allocations and clones aren't going to make a difference. When designing APIs, please prioritize making them easy to use. Concretely, if lifetimes are giving you grief, just clone the struct.

Functions should accept the most inputs possible, for example:

```rust
fn do_stuff(s: impl ToString) -> String;
```

is much better than

```rust
fn do_stuff(s: &'a str) -> String;
```

because the first version can accept `String`, `&String`, `&str` and any other data type that implements the `ToString` (or `Display`) trait, which is most of them.
