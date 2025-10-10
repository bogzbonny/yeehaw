## Overview
The Theme Manager provides a centralized, extensible system for defining and retrieving colors (and related style information) for every UI element. By storing a theme in the application’s `Context` and exposing a trait that maps an element **kind** to a `Color`/`Style`, UI components can automatically obtain their visual appearance. Themes can be swapped globally (e.g., day/night) or locally for a subset of elements, and all colors update without requiring explicit per‑component changes.

## Goals
- Offer a single source of truth for element colors and styles.
- Enable easy theme injection via the existing `Context` (`Rc<RefCell<Theme>>`).
- Allow sub‑themes for specific groups of elements without affecting the rest of the UI.
- Support fast day/night switching by swapping or resetting the theme.
- Keep the system fully extensible by using string‑based keys for theme entries.

## Scope
**In scope**
- Design and implementation of a `Theme` struct that holds a `HashMap<String, Color>` (and optional style data).
- Integration of the theme into the application `Context` using `Rc<RefCell<Theme>>`.
- Creation of a trait (e.g., `Themed`) analogous to `Into<Color>` that takes an element kind and returns the appropriate color/style from the current theme.
- Default constructor functions for UI elements that pull colors from the theme.
- Decorator helpers (e.g., `.with_color(...)`) to apply special overrides.
- Mechanism for creating a new `Context` with an overridden theme for a subtree of elements.
- Night/day theme switching by swapping the entire `Theme` object or resetting its color map.

**Out of scope**
- Persistence of user‑selected themes to disk.
- Actual rendering logic of UI components (the manager only supplies colors).
- Designing a complete design‑system beyond color mapping (e.g., typography, spacing).

## Requirements
1. **Themed Trait**: Implement a trait that, given an element kind (as a string or enum), returns a `Color`/`Style` by looking it up in the current theme, mirroring the `Into<Color>` pattern.
2. **Theme Storage**: Store a `Theme` instance inside the global `Context` as `Rc<RefCell<Theme>>`.
3. **Default Construction**: UI element constructors (`new`) should automatically fetch their default colors from the theme stored in `Context`.
4. **Sub‑theme Override**: Allow creation of a child `Context` that replaces the theme for a specific group of elements, without affecting the parent context.
5. **Extensible Map**: `Theme` must use a `HashMap<String, Color>` (or `HashMap<String, Style>` as needed) so any new element kind can be added without code changes.
6. **Day/Night Switching**: Provide an API to replace the entire `Theme` object or reset its color map, causing all elements that query the theme to reflect the new colors.
7. **Decorators**: Supply helper methods (e.g., `with_color`, `with_style`) that apply explicit overrides on top of the theme‑derived defaults.
8. **No New Color Kind**: Do **not** introduce a new `Color` variant such as “ColorFromTheme”; instead rely on the existing `Color` type combined with theme lookup.
9. **Automatic Refresh**: When the theme is swapped, any UI element that queries its color lazily (or on next render) should see the updated value without needing manual updates.

## Acceptance Criteria
- The `Themed` trait returns the correct `Color` for a given element kind based on the theme present in the current `Context`.
- Changing the theme in the global `Context` (e.g., swapping to a night theme) results in all UI elements reflecting the new colors on the next render.
- Creating a child `Context` with an overridden `Theme` only changes colors for elements constructed within that child context.
- The theme map accepts arbitrary string keys; unknown keys fall back to a sensible default color.
- Decorator functions (`with_color`, `with_style`) correctly override the theme‑derived values for a specific element.
- No additional `Color` enum variants are introduced; all theme interactions use the existing `Color` type.
- Night/day switching can be performed by either resetting the theme’s internal map or swapping the entire `Theme` object, and both approaches produce identical visual results.
- All requirements above are covered by unit tests that verify lookup, overrides, sub‑theme scoping, and theme swapping behavior.