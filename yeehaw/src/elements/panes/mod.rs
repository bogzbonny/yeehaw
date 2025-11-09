pub mod bat_viewer;
pub mod file_navigator;

pub mod pane_limiter;
pub mod pane_scrollable;
pub mod pane_selectable;

#[cfg(feature = "textbox")]
pub mod file_viewer;

#[cfg(feature = "terminal")]
pub mod terminal;
#[cfg(feature = "terminal_editor")]
pub mod terminal_editor;

pub use {
    file_navigator::FileNavPane,
    pane_limiter::PaneLimiter,
    pane_scrollable::PaneScrollable,
    pane_selectable::{ParentPaneOfSelectable, SelStyles, Selectability, SelectablePane},
};

#[cfg(feature = "terminal")]
pub use terminal::TerminalPane;

#[cfg(feature = "terminal_editor")]
pub use terminal_editor::TermEditorPane;

#[cfg(feature = "textbox")]
pub use file_viewer::FileViewerPane;
