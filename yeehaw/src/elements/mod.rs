pub mod debug_pane;
pub mod file_navigator;
pub mod file_viewer;
pub mod image_viewer;
pub mod menu;
pub mod menu_right_click;
pub mod pane;
pub mod pane_parent;
pub mod pane_scrollable;
pub mod stack;
pub mod tabs;
pub mod terminal;
pub mod terminal_editor;
pub mod window;

pub use {
    debug_pane::DebugSizePane,
    file_navigator::FileNavPane,
    file_viewer::FileViewerPane,
    image_viewer::ImageViewer,
    menu::MenuBar,
    menu_right_click::RightClickMenu,
    pane::Pane,
    pane_parent::ParentPane,
    pane_scrollable::{PaneScrollable, PaneWithScrollbars},
    stack::{HorizontalStack, VerticalStack},
    tabs::Tabs,
    terminal::TerminalPane,
    terminal_editor::TermEditorPane,
    window::WindowPane,
};
