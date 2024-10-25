#[macro_use]
pub mod debug; // note, must be first for the macro to work throughout

pub mod ch;
pub mod color;
pub mod cui;
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
pub mod widgets;

pub use {
    ch::{ChPlus, DrawCh, DrawChPos, DrawChPosVec, DrawChs2D},
    color::{Color, Gradient, RadialGradient, Rgba, TimeGradient},
    cui::Cui,
    dyn_location::{DynLocation, DynLocationSet, Loc, Point, Size, ZIndex},
    dyn_value::DynVal,
    element::{
        Context, Element, EventResponse, EventResponses, ReceivableEventChanges, UpwardPropagator,
    },
    elements::{
        DebugSizePane, FileNavPane, FileViewerPane, HorizontalStack, ImageViewer, MenuBar, Pane,
        PaneScrollable, PaneWithScrollbars, ParentPane, RightClickMenu, Tabs, VerticalStack,
    },
    errors::Error,
    event::{CommandEvent, Event, KeyPossibility},
    keyboard::Keyboard,
    organizer::ElementOrganizer,
    prioritizer::Priority,
    sorting_hat::{ElementID, SortingHat},
    style::{Attributes, Style},
    widgets::WidgetPane,
};
