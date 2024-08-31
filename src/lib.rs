#[macro_use]
pub mod debug; // note, must be first for the macro to work throughout

pub mod ch;
pub mod colour;
pub mod cui;
pub mod element;
pub mod elements;
pub mod errors;
pub mod event;
pub mod keyboard;
pub mod organizer;
pub mod prioritizer;
pub mod scl_location;
pub mod scl_value;
pub mod sorting_hat;
pub mod style;
pub mod widgets;

pub use {
    ch::{DrawCh, DrawChPos, DrawChs2D},
    colour::RgbColour,
    cui::Cui,
    element::{
        Context, CreateWindow, Element, EventResponse, EventResponses, ReceivableEventChanges,
        UpwardPropagator,
    },
    elements::{MenuBar, Pane, ParentPane, RightClickMenu},
    errors::Error,
    event::{CommandEvent, Event, KeyPossibility},
    keyboard::Keyboard,
    organizer::ElementOrganizer,
    prioritizer::Priority,
    scl_location::{Point, SclLocation, SclLocationSet, Size, ZIndex},
    scl_value::SclVal,
    sorting_hat::{ElementID, SortingHat},
    style::{Style, YHAttributes},
    widgets::WidgetPane,
};
