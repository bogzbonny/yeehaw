pub mod debug_pane;
pub mod menu;
pub mod menu_right_click;
pub mod pane;
pub mod pane_parent;
pub mod tabs;

pub use {
    debug_pane::DebugSizePane, menu::MenuBar, menu_right_click::RightClickMenu, pane::Pane,
    pane_parent::ParentPane,
};
