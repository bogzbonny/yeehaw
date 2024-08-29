#[macro_use]
pub mod debug; // note, must be first for the macro to work throughout

pub mod colour;
pub mod cui;
pub mod cui_ch;
pub mod cui_location;
pub mod element;
pub mod elements;
pub mod errors;
pub mod event;
pub mod keyboard;
pub mod organizer;
pub mod prioritizer;
pub mod scaled_value;
pub mod sorting_hat;
pub mod style;
pub mod widgets;

pub use {
    colour::RgbColour,
    cui::Cui,
    cui_ch::{DrawCh, DrawChPos, DrawChs2D},
    cui_location::{Location, LocationSet, Point, Size, ZIndex},
    element::{
        Context, CreateWindow, Element, EventResponse, EventResponses, ReceivableEventChanges,
        UpwardPropagator,
    },
    elements::{MenuBar, ParentPane, RightClickMenu, StandardPane},
    errors::Error,
    event::{CommandEvent, Event, KeyPossibility},
    keyboard::Keyboard,
    organizer::ElementOrganizer,
    prioritizer::Priority,
    scaled_value::{SclLocation, SclVal},
    sorting_hat::{ElementID, SortingHat},
    style::{Style, YHAttributes},
    widgets::WidgetPane,
};
