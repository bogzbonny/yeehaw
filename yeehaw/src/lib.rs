#[macro_use]
pub mod debug; // note, must be first for the macro to work throughout

pub mod ch;
pub mod color;
pub mod context;
pub mod dyn_location;
pub mod dyn_value;
pub mod element;
pub mod elements;
pub mod errors;
pub mod event;
pub mod keyboard;
pub mod organizer;
pub mod prioritizer;
pub mod sorting_hat;
pub mod style;
pub mod tui;

pub use {
    ch::{ChPlus, DrawCh, DrawChPos, DrawChPosVec, DrawChs2D},
    color::{Color, Gradient, RadialGradient, Rgba, TimeGradient},
    context::Context,
    dyn_location::{DynLocation, DynLocationSet, Loc, Point, Size, ZIndex},
    dyn_value::DynVal,
    element::{Element, HookFn as ElementHookFn, Parent},
    elements::{
        border::Corner as BorderCorner, border::CornerPos,
        border::PropertyCnr as BorderPropertyCnr, Bordered, Button, ButtonMicroShadow,
        ButtonShadow, ButtonSides, Checkbox, DebugSizePane, DropdownList, FigletText, FileNavPane,
        FileViewerPane, HorizontalSBPositions, HorizontalScrollbar, HorizontalStack, ImageViewer,
        Label, ListBox, MenuBar, NumbersTextBox, Pane, PaneScrollable, ParentPane,
        ParentPaneOfSelectable, RadioButtons, RightClickMenu, ScrollbarSty, SelStyles,
        Selectability, SelectablePane, Shadowed, Tabs, TermEditorPane, TerminalPane, TextBox,
        Toggle, VerticalSBPositions, VerticalScrollbar, VerticalStack, WindowPane,
    },
    errors::Error,
    event::{
        CommandEvent, Event, EventResponse, EventResponses, KeyPossibility, MoveResponse,
        ReceivableEvent, ReceivableEventChanges, RelMouseEvent, ResizeResponse,
        SelfReceivableEvents,
    },
    keyboard::Keyboard,
    organizer::ElementOrganizer,
    prioritizer::Priority,
    sorting_hat::{ElementID, SortingHat},
    style::{Attributes, BgTranspSrc, FgTranspSrc, Style, UlTranspSrc},
    tui::Tui,
};
