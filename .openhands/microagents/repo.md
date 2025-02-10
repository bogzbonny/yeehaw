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
- ONLY EVER make changes to files inside the `yeehaw/` folder, DO NOT add a root
  level `src/` folder or root level `examples/` folder, instead work within
  `yeehaw/src/` and `yeehaw/examples`. 
- Never import new libraries unless explicitly asked too.
- This is a new TUI framework which is not ratatui, do not use ratatui types unless explicitly asked too
- When writing yeehaw examples, look at other examples stored in `yeehaw/yeehaw/examples/` first 
- When writing new yeehaw elements, look at other elements are programmed in `yeehaw/yeehaw/src/elements/`, use these as examples. Also study the Element Trait in `yeehaw/yeehaw/src/element.rs`.
- Always code in Rust.
- When building new elements and widgets always use other elements and widget as
  a template to work from.
- Write tests for all new features.
- After writing code always run `cargo test` to make sure the code builds and
  runs. fix any errors that occur


