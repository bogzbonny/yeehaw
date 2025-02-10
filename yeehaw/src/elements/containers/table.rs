use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crossterm::style::Color;

use crate::{
    Context, DrawAction, DrawCh, DrawChPos, DrawRegion, DrawUpdate, DynLocation, DynLocationSet,
    DynVal, Element, ElementID, Event, EventResponses, HookFn, Label, Parent, ReceivableEvents,
};

/// Box drawing characters for table borders
const BOX_HORIZONTAL: char = '─';
const BOX_VERTICAL: char = '│';
const BOX_CROSS: char = '┼';
const BOX_DOWN_RIGHT: char = '┌';
const BOX_DOWN_LEFT: char = '┐';
const BOX_UP_RIGHT: char = '└';
const BOX_UP_LEFT: char = '┘';
const BOX_VERTICAL_RIGHT: char = '├';
const BOX_VERTICAL_LEFT: char = '┤';
const BOX_HORIZONTAL_DOWN: char = '┬';
const BOX_HORIZONTAL_UP: char = '┴';

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
            id: ctx.new_context().get_element_id(),
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

    /// Calculate column widths based on content and provided DynVal settings
    fn calculate_grid_layout(&self, dr: &DrawRegion) -> (Vec<usize>, Vec<usize>) {
        let mut col_widths = Vec::new();
        let mut row_heights = Vec::new();
        
        let total_width = dr.get_width();
        let total_height = dr.get_height();

        // Calculate column widths
        let num_cols = self.cells.first().map_or(0, |r| r.len());
        for i in 0..num_cols {
            let width = if i < self.column_widths.len() {
                self.column_widths[i].get_val(total_width as u16) as usize
            } else {
                // Default to equal distribution
                total_width / num_cols
            };
            col_widths.push(width);
        }

        // Calculate row heights (default to 1 for now)
        for _ in 0..self.cells.len() {
            row_heights.push(1);
        }

        (col_widths, row_heights)
    }

    /// Get the background color for a cell based on row/column alternation
    fn get_cell_bg_color(&self, row: usize, col: usize) -> Option<Color> {
        if row == 0 {
            return self.style.header_bg_color;
        }

        // Row alternation takes precedence over column alternation
        if let Some((color, length)) = self.style.row_alternation {
            if (row - 1) % length == 0 {
                return Some(color);
            }
        }

        if let Some((color, length)) = self.style.column_alternation {
            if col % length == 0 {
                return Some(color);
            }
        }

        self.style.content_bg_color
    }

    /// Get the foreground color for a cell
    fn get_cell_fg_color(&self, row: usize) -> Option<Color> {
        if row == 0 {
            self.style.header_fg_color
        } else {
            self.style.content_fg_color
        }
    }
}

impl Clone for Table {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
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
        self.id.clone()
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
        // For now, tables don't handle events
        (false, EventResponses::default())
    }

    fn set_focused(&self, focused: bool) {
        *self.focused.borrow_mut() = focused;
    }

    fn get_focused(&self) -> bool {
        *self.focused.borrow()
    }

    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        if !self.get_visible() {
            return vec![DrawUpdate::clear_all()];
        }

        let mut updates = Vec::new();
        let mut draw_chars = Vec::new();
        let location = self.get_dyn_location_set();
        let base_x = location.l.get_end_x(dr) as usize;
        let base_y = location.l.get_end_y(dr) as usize;
        
        let (col_widths, row_heights) = self.calculate_grid_layout(dr);

        // Draw cell contents and borders
        let mut y = base_y;
        for (row_idx, row) in self.cells.iter().enumerate() {
            let mut x = base_x;

            // Draw horizontal line above header if it's the first row
            if row_idx == 0 && self.style.header_line {
                for (col_idx, width) in col_widths.iter().enumerate() {
                    let mut line_char = BOX_HORIZONTAL;
                    if col_idx == 0 {
                        line_char = BOX_DOWN_RIGHT;
                    } else if col_idx == col_widths.len() - 1 {
                        line_char = BOX_DOWN_LEFT;
                    } else if self.style.vertical_lines {
                        line_char = BOX_HORIZONTAL_DOWN;
                    }

                    draw_chars.push(DrawChPos::new(
                        x,
                        y,
                        line_char as u8,
                        None,
                        None
                    ));
                    x += 1;

                    // Fill the rest of the column with horizontal lines
                    for _ in 1..*width {
                        draw_chars.push(DrawChPos::new(
                            x,
                            y,
                            BOX_HORIZONTAL as u8,
                            None,
                            None
                        ));
                        x += 1;
                    }
                }
                y += 1;
            }

            x = base_x;
            // Draw cell contents
            for (col_idx, cell) in row.iter().enumerate() {
                let width = col_widths[col_idx];
                let bg_color = self.get_cell_bg_color(row_idx, col_idx);
                let fg_color = self.get_cell_fg_color(row_idx);

                // Draw vertical line before cell if enabled
                if self.style.vertical_lines && col_idx > 0 {
                    draw_chars.push(DrawChPos::new(
                        x - 1,
                        y,
                        BOX_VERTICAL as u8,
                        None,
                        None
                    ));
                }

                // Draw cell content
                for dx in 0..width {
                    draw_chars.push(DrawChPos::new(
                        x + dx,
                        y,
                        b' ',
                        fg_color,
                        bg_color
                    ));
                }

                x += width;
            }

            // Draw horizontal line below cells if enabled
            if (row_idx > 0 || self.style.header_line) && 
               (row_idx < self.cells.len() - 1) && 
               self.style.horizontal_lines {
                y += 1;
                let mut x = base_x;
                for (col_idx, width) in col_widths.iter().enumerate() {
                    let mut line_char = BOX_HORIZONTAL;
                    if col_idx == 0 {
                        line_char = BOX_VERTICAL_RIGHT;
                    } else if col_idx == col_widths.len() - 1 {
                        line_char = BOX_VERTICAL_LEFT;
                    } else if self.style.vertical_lines {
                        line_char = BOX_CROSS;
                    }

                    draw_chars.push(DrawChPos::new(
                        x,
                        y,
                        line_char as u8,
                        None,
                        None
                    ));
                    x += 1;

                    for _ in 1..*width {
                        draw_chars.push(DrawChPos::new(
                            x,
                            y,
                            BOX_HORIZONTAL as u8,
                            None,
                            None
                        ));
                        x += 1;
                    }
                }
                y += 1;
            } else {
                y += row_heights[row_idx];
            }
        }

        // Create final update
        updates.push(DrawUpdate {
            sub_id: vec![self.id()],
            z_indicies: vec![location.z],
            action: DrawAction::Update(draw_chars),
        });

        updates
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        None
    }

    fn set_attribute_inner(&self, _key: &str, _value: Vec<u8>) {}

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

    fn get_content_width(&self, dr: Option<&DrawRegion>) -> usize {
        if let Some(dr) = dr {
            let (col_widths, _) = self.calculate_grid_layout(dr);
            col_widths.iter().sum()
        } else {
            0
        }
    }

    fn get_content_height(&self, dr: Option<&DrawRegion>) -> usize {
        if let Some(dr) = dr {
            let (_, row_heights) = self.calculate_grid_layout(dr);
            row_heights.iter().sum()
        } else {
            0
        }
    }
}

// Implement string to element conversion using Label
impl From<String> for Box<dyn Element> {
    fn from(s: String) -> Self {
        Box::new(Label::new(&Context::new_context(), &s))
    }
}

impl From<&str> for Box<dyn Element> {
    fn from(s: &str) -> Self {
        Box::new(Label::new(&Context::new_context(), s))
    }
}
