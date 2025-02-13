#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README_DOCS.md"))]

#[macro_use]
pub mod log; // note, must be first for the macro to work throughout

pub mod ch;
pub mod color;
pub mod context;
pub mod draw_cache;
pub mod draw_region;
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
    color::{Color, ColorStore, Gradient, Pattern, RadialGradient, Rgba, TimeGradient},
    context::Context,
    draw_cache::{CachedPos, DrawingCache},
    draw_region::DrawRegion,
    dyn_location::{DynLocation, DynLocationSet, Loc, Point, Size, ZIndex},
    dyn_value::DynVal,
    element::{DrawAction, DrawUpdate, Element, HookFn as ElementHookFn, Parent},
    elements::{border::Corner as BorderCorner, border::PropertyCnr as BorderPropertyCnr, *},
    errors::Error,
    event::{
        CommandEvent, Event, EventResponse, EventResponses, KeyPossibility, MouseEvent,
        MoveResponse, ReceivableEvent, ReceivableEvents, ResizeResponse,
    },
    keyboard::Keyboard,
    organizer::ElementOrganizer,
    sorting_hat::{ElementID, SortingHat},
    style::{Attributes, BgTranspSrc, FgTranspSrc, Style, UlTranspSrc},
    tui::Tui,
};

pub use {
    box_drawing_logic::{BoxDrawingCh, SideAttribute as BoxSideAttr},

    // we re-export these so that the user doesn't have to import them from std
    // while using yeehaw-derive
    std::{
        cell::{Ref, RefCell, RefMut},
        rc::Rc,
    },
};
