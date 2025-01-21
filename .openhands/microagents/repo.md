---
name: repo
type: repo
agent: CodeActAgent
---

Repository: yeehaw
Description: A TUI framework
Language: Rust

Directory Structure:
- yeehaw/src/: Main application code
- yeehaw/examples/: Examples of yeehaw
- yeehaw/src/elements/: Location of all the yeehaw tui elements
- yeehaw/src/elements/widgets/: Location of all the yeehaw tui widgets

Setup:
- At startup always install cargo using: `curl https://sh.rustup.rs -sSf | sh`
- Run `cargo build` for ensuring code builds
- Run `cargo test -p yeehaw --no-fail-fast` for testing

Guidelines:
- Always code in Rust.
- When building new elements and widgets always use other elements and widget as
  a template to work from.
- Write tests for all new features.
- After writing code always run `cargo test` to make sure the code builds and
  runs. fix any errors that occur


