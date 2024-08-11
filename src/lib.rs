#[macro_use]
pub mod debug; // note, must be first for the macro to work throughout

pub mod colour;
pub mod cui;
pub mod cui_ch;
pub mod cui_location;
pub mod element;
pub mod errors;
pub mod event;
pub mod keyboard;
pub mod organizer;
pub mod prioritizer;
pub mod style;

pub use {
    colour::RgbColour,
    cui_ch::{DrawCh, DrawChPos},
    cui_location::{Location, LocationSet, Size, ZIndex},
    element::{Context, CreateWindow, Element, ElementID, EventResponse, UpwardPropagator},
    errors::Error,
    event::{CommandEvent, Event, KeyPossibility},
    keyboard::Keyboard,
    organizer::ElementOrganizer,
    prioritizer::Priority,
    style::Style,
};
