# Repository Guidelines

## Project Structure & Modules
- `yeehaw/` – core library, contains the element system, context, and UI utilities.
- `yeehaw_derive/` – procedural macros that power the `#[derive]` helpers.
- `vt100_yh/` – thin wrapper around a VT‑100 parser used by the UI.
- `box_drawing_logic/` – small crate that implements Unicode box‑drawing helpers.
- `examples/` – runnable demo applications (`showcase`, `shared/*`).
- `assets/` – static images and Figlet fonts referenced by the README.
- `docs/` – design notes and getting‑started guides.
- `Cargo.toml` (workspace) – aggregates the above crates.

## Build, Test & Development Commands
```bash
# Compile the whole workspace (debug)
cargo build

# Compile with optimisations (release)
cargo build --release

# Run an example (e.g. the showcase demo)
cargo run --release --example showcase

# Run all tests (currently only unit tests in the crates)
cargo test
```
All commands are executed from the repository root.

## Coding Style & Naming
- Follow the Rust 2024 edition guidelines.
- Indentation = 4 spaces (no tabs). See `rustfmt.toml` for the project‑wide formatter.
- Types use `PascalCase`, functions/variables `snake_case`.
- Public items are kept `pub` where useful; internal helpers are prefixed with an underscore.
- Use `use_field_init_shorthand` and `use_try_shorthand` – the formatter will apply them automatically.

## Testing Guidelines
- Tests reside in the crate’s `tests/` directory or as `#[cfg(test)]` modules.
- Name test functions descriptively, e.g. `fn test_button_click_behaviour()`.
- Run tests with `cargo test`; coverage can be collected with `cargo tarpaulin` (optional).

## Commit & Pull‑Request Workflow
- Commit messages follow the conventional style: `type(scope): short description`.
  - Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`.
  - Example: `feat(ui): add dynamic label sizing`.
- Pull requests must:
  1. Reference a GitHub issue.
  2. Include a concise description of the change.
  3. Pass `cargo test` on CI.
  4. (Optional) Provide screenshots or example output for UI changes.

## Additional Tips
- Run `rustfmt` before committing: `cargo fmt --all`.
- Lint with `cargo clippy --all-targets --all-features` to keep the codebase healthy.
- When adding new crates, list them in the workspace `members` array.

---
These guidelines are a living document; feel free to propose improvements via PR.
