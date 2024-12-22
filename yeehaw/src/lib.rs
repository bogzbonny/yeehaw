#![doc = include_str!("../../docs/01_getting_started.md")]

#[macro_use]
pub mod log; // note, must be first for the macro to work throughout

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
pub mod sorting_hat;
pub mod style;
pub mod tui;

pub use {
    ch::{ChPlus, DrawCh, DrawChPos, DrawChPosVec, DrawChs2D},
    color::{Color, Gradient, RadialGradient, Rgba, TimeGradient},
    context::Context,
    dyn_location::{DynLocation, DynLocationSet, Loc, Point, Size, ZIndex},
    dyn_value::DynVal,
    element::{DrawAction, DrawUpdate, DrawingCache, Element, HookFn as ElementHookFn, Parent},
    elements::{border::Corner as BorderCorner, border::PropertyCnr as BorderPropertyCnr, *},
    errors::Error,
    event::{
        CommandEvent, Event, EventResponse, EventResponses, KeyPossibility, MoveResponse,
        ReceivableEvent, RelMouseEvent, ResizeResponse, SelfReceivableEvents,
    },
    keyboard::Keyboard,
    organizer::ElementOrganizer,
    sorting_hat::{ElementID, SortingHat},
    style::{Attributes, BgTranspSrc, FgTranspSrc, Style, UlTranspSrc},
    tui::Tui,
};

// we re-export these so that the user doesn't have to import them from std
// if they are using yeehaw-derive
pub use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};
