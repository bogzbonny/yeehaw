use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crossterm::style::Color;

use crate::{
    Context, DrawChPos, DrawRegion, DynLocation, DynLocationSet, DynVal, Element, ElementID,
    Event, EventResponses, HookFn, Label, Parent, ReceivableEvents,
};

/// Style configuration for table appearance
#[derive(Clone, Debug)]
pub struct TableStyle {
    /// Draw a horizontal line under the header row
    pub header_line: bool,
    /// Draw vertical lines between columns
    pub vertical_lines: bool,
    /// Draw horizontal lines between content rows
    pub horizontal_lines: bool,
    /// Foreground color for header cells
    pub header_fg_color: Option<Color>,
    /// Background color for header cells
    pub header_bg_color: Option<Color>,
    /// Foreground color for content cells
    pub content_fg_color: Option<Color>,
    /// Background color for content cells
    pub content_bg_color: Option<Color>,
    /// Alternating background colors for rows with a cycle length
    pub row_alternation: Option<(Color, usize)>,
    /// Alternating background colors for columns with a cycle length
    pub column_alternation: Option<(Color, usize)>,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            header_line: true,
            vertical_lines: true,
            horizontal_lines: false,
            header_fg_color: None,
            header_bg_color: None,
            content_fg_color: None,
            content_bg_color: None,
            row_alternation: None,
            column_alternation: None,
        }
    }
}

/// A table container element that can display data in a grid format
pub struct Table {                    
    id: ElementID,
    cells: Vec<Vec<Box<dyn Element>>>,
    column_widths: Vec<DynVal>,
    row_heights: Vec<DynVal>,
    style: TableStyle,
    dyn_location_set: Rc<RefCell<DynLocationSet>>,
    visible: Rc<RefCell<bool>>,
    overflow: Rc<RefCell<bool>>,
    content_x_offset: Rc<RefCell<usize>>,
    content_y_offset: Rc<RefCell<usize>>,
    hooks: Rc<RefCell<HashMap<String, HashMap<ElementID, HookFn>>>>,
    parent: Rc<RefCell<Option<Box<dyn Parent>>>>,
    focused: Rc<RefCell<bool>>,
    receivable: Vec<Rc<RefCell<ReceivableEvents>>>,
}

impl Table {
    /// Create a new table with the given dimensions and style
    pub fn new(ctx: &Context, style: TableStyle) -> Self {
        Self {
            id: ctx.get_element_id(),
            cells: Vec::new(),
            column_widths: Vec::new(),
            row_heights: Vec::new(),
            style,
            dyn_location_set: Rc::new(RefCell::new(DynLocationSet::default())),
            visible: Rc::new(RefCell::new(true)),
            overflow: Rc::new(RefCell::new(false)),
            content_x_offset: Rc::new(RefCell::new(0)),
            content_y_offset: Rc::new(RefCell::new(0)),
            hooks: Rc::new(RefCell::new(HashMap::new())),
            parent: Rc::new(RefCell::new(None)),
            focused: Rc::new(RefCell::new(false)),
            receivable: Vec::new(),
        }
    }

    /// Set the data for the table, converting strings to labels automatically
    pub fn set_data<T>(&mut self, data: Vec<Vec<T>>) 
    where 
        T: Into<Box<dyn Element>>,
    {
        self.cells = data.into_iter()
            .map(|row| row.into_iter()
                .map(|cell| cell.into())
                .collect())
            .collect();
    }

    /// Set column widths using DynVal
    pub fn set_column_widths(&mut self, widths: Vec<DynVal>) {
        self.column_widths = widths;
    }

    /// Set row heights using DynVal
    pub fn set_row_heights(&mut self, heights: Vec<DynVal>) {
        self.row_heights = heights;
    }
}

impl Clone for Table {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            cells: self.cells.clone(),
            column_widths: self.column_widths.clone(),
            row_heights: self.row_heights.clone(),
            style: self.style.clone(),
            dyn_location_set: self.dyn_location_set.clone(),
            visible: self.visible.clone(),
            overflow: self.overflow.clone(),
            content_x_offset: self.content_x_offset.clone(),
            content_y_offset: self.content_y_offset.clone(),
            hooks: self.hooks.clone(),
            parent: self.parent.clone(),
            focused: self.focused.clone(),
            receivable: self.receivable.clone(),
        }
    }
}

impl Element for Table {
    fn kind(&self) -> &'static str {
        "table"
    }

    fn id(&self) -> ElementID {
        self.id
    }

    fn can_receive(&self, ev: &Event) -> bool {
        for r in &self.receivable {
            if r.borrow().can_receive(ev) {
                return true;
            }
        }
        false
    }

    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        self.receivable.clone()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        // Implement event handling for table interaction
        (false, EventResponses::default())
    }

    fn set_focused(&self, focused: bool) {
        *self.focused.borrow_mut() = focused;
    }

    fn get_focused(&self) -> bool {
        *self.focused.borrow()
    }

    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        let mut updates = Vec::new();
        // TODO: Implement table drawing with borders and styles
        updates
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        None
    }

    fn set_attribute_inner(&self, key: &str, value: Vec<u8>) {
        // Not needed for basic table implementation
    }

    fn set_hook(&self, kind: &str, el_id: ElementID, hook: HookFn) {
        self.hooks.borrow_mut()
            .entry(kind.to_string())
            .or_default()
            .insert(el_id, hook);
    }

    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        if let Some(hooks) = self.hooks.borrow_mut().get_mut(kind) {
            hooks.remove(&el_id);
        }
    }

    fn clear_hooks_by_id(&self, el_id: ElementID) {
        for hooks in self.hooks.borrow_mut().values_mut() {
            hooks.remove(&el_id);
        }
    }

    fn call_hooks_of_kind(&self, kind: &str) {
        if let Some(hooks) = self.hooks.borrow_mut().get_mut(kind) {
            for hook in hooks.values_mut() {
                hook(kind, Box::new(self.clone()));
            }
        }
    }

    fn set_parent(&self, up: Box<dyn Parent>) {
        *self.parent.borrow_mut() = Some(up);
    }

    fn get_dyn_location_set(&self) -> Ref<DynLocationSet> {
        self.dyn_location_set.borrow()
    }

    fn get_visible(&self) -> bool {
        *self.visible.borrow()
    }

    fn get_ref_cell_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.dyn_location_set.clone()
    }

    fn get_ref_cell_visible(&self) -> Rc<RefCell<bool>> {
        self.visible.clone()
    }

    fn get_ref_cell_overflow(&self) -> Rc<RefCell<bool>> {
        self.overflow.clone()
    }

    fn set_content_x_offset(&self, _dr: Option<&DrawRegion>, x: usize) {
        *self.content_x_offset.borrow_mut() = x;
    }

    fn set_content_y_offset(&self, _dr: Option<&DrawRegion>, y: usize) {
        *self.content_y_offset.borrow_mut() = y;
    }

    fn get_content_x_offset(&self) -> usize {
        *self.content_x_offset.borrow()
    }

    fn get_content_y_offset(&self) -> usize {
        *self.content_y_offset.borrow()
    }

    fn get_content_width(&self, _dr: Option<&DrawRegion>) -> usize {
        0 // TODO: Calculate total table width
    }

    fn get_content_height(&self, _dr: Option<&DrawRegion>) -> usize {
        0 // TODO: Calculate total table height
    }
}

// Implement string to element conversion
impl From<String> for Box<dyn Element> {
    fn from(s: String) -> Self {
        Box::new(Label::new(&Context::mock(), DynLocation::default(), &s))
    }
}

impl From<&str> for Box<dyn Element> {
    fn from(s: &str) -> Self {
        Box::new(Label::new(&Context::mock(), DynLocation::default(), s))
    }
}
