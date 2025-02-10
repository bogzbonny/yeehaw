use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crossterm::style::Color as CrosstermColor;

use crate::{
    color::Color, Context, DrawAction, DrawCh, DrawChPos, DrawRegion, DrawUpdate, DynLocation, DynLocationSet,
    DynVal, Element, ElementID, Event, EventResponses, element::HookFn, Label, Parent, 
    ReceivableEvents, Style,
};

#[derive(Clone, Copy, Debug)]
struct BoxDrawChars;

impl BoxDrawChars {
    const STYLE: Style = Style::transparent();

    const HORIZONTAL: DrawCh = DrawCh::const_new('─', Self::STYLE);
    const VERTICAL: DrawCh = DrawCh::const_new('│', Self::STYLE);
    const CROSS: DrawCh = DrawCh::const_new('┼', Self::STYLE);
    const DOWN_RIGHT: DrawCh = DrawCh::const_new('┌', Self::STYLE);
    const DOWN_LEFT: DrawCh = DrawCh::const_new('┐', Self::STYLE);
    const UP_RIGHT: DrawCh = DrawCh::const_new('└', Self::STYLE);
    const UP_LEFT: DrawCh = DrawCh::const_new('┘', Self::STYLE);
    const VERTICAL_RIGHT: DrawCh = DrawCh::const_new('├', Self::STYLE);
    const VERTICAL_LEFT: DrawCh = DrawCh::const_new('┤', Self::STYLE);
    const HORIZONTAL_DOWN: DrawCh = DrawCh::const_new('┬', Self::STYLE);
    const HORIZONTAL_UP: DrawCh = DrawCh::const_new('┴', Self::STYLE);
}

/// Style configuration for table appearance.
/// All style options are optional and can be enabled/disabled independently.
#[derive(Clone, Debug)]
pub struct TableStyle {
    /// Draw a horizontal line under the header row
    pub header_line: bool,
    /// Draw vertical lines between columns
    pub vertical_lines: bool,
    /// Draw horizontal lines between content rows
    pub horizontal_lines: bool,
    /// Foreground color for header cells
    pub header_fg_color: Option<CrosstermColor>,
    /// Background color for header cells
    pub header_bg_color: Option<CrosstermColor>,
    /// Foreground color for content cells
    pub content_fg_color: Option<CrosstermColor>,
    /// Background color for content cells
    pub content_bg_color: Option<CrosstermColor>,
    /// Alternating background colors for rows with a cycle length
    pub row_alternation: Option<(CrosstermColor, usize)>,
    /// Alternating background colors for columns with a cycle length
    pub column_alternation: Option<(CrosstermColor, usize)>,
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

/// Width configuration for table columns
#[derive(Clone, Debug)]
pub enum ColumnWidth {
    /// Fixed width in characters
    Fixed(DynVal),
    /// Percentage of available width
    Percent(DynVal),
    /// Automatically size based on content
    Auto,
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self::Auto
    }
}

/// A table container element that can display data in a grid format.
/// Each cell can contain any element, and strings are automatically converted to Label elements.
pub struct Table {                    
    id: ElementID,
    cells: Vec<Vec<Box<dyn Element>>>,
    column_widths: Vec<ColumnWidth>,
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
    /// Create a new table with the given style
    pub fn new(ctx: &Context, style: TableStyle) -> Self {
        Self {
            id: ctx.hat.create_element_id("table"),
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

    /// Set data in the table from anything that can convert to a string
    pub fn set_data<T>(&mut self, ctx: &Context, data: Vec<Vec<T>>)
    where 
        T: ToString
    {
        self.cells = data.into_iter()
            .map(|row| row.into_iter()
                .map(|cell| Box::new(Label::new(ctx, cell.to_string())) as Box<dyn Element>)
                .collect())
            .collect();
    }

    /// Set data from elements directly
    pub fn set_data_elements<T>(&mut self, data: Vec<Vec<T>>)
    where
        T: Into<Box<dyn Element>>
    {
        self.cells = data.into_iter()
            .map(|row| row.into_iter()
                .map(Into::into)
                .collect())
            .collect();
    }

    /// Set column widths using ColumnWidth configuration
    pub fn set_column_widths(&mut self, widths: Vec<ColumnWidth>) {
        self.column_widths = widths;
    }

    /// Set row heights using DynVal
    pub fn set_row_heights(&mut self, heights: Vec<DynVal>) {
        self.row_heights = heights;
    }

    /// Calculate the width needed for a cell's content
    fn calculate_cell_width(&self, cell: &Box<dyn Element>, dr: &DrawRegion) -> usize {
        cell.get_content_width(Some(dr))
    }

    /// Calculate the height needed for a cell's content
    fn calculate_cell_height(&self, cell: &Box<dyn Element>, dr: &DrawRegion) -> usize {
        cell.get_content_height(Some(dr))
    }

    /// Calculate column widths and row heights based on content and settings
    fn calculate_grid_layout(&self, dr: &DrawRegion) -> (Vec<usize>, Vec<usize>) {
        let mut col_widths = Vec::new();
        let mut row_heights = Vec::new();
        
        let total_width = dr.get_width() as usize;
        let total_height = dr.get_height() as usize;

        // First pass: calculate auto widths and fixed widths
        let num_cols = self.cells.first().map_or(0, |r| r.len());
        let mut remaining_width = total_width;
        let mut auto_cols = Vec::new();

        for col_idx in 0..num_cols {
            let width_setting = self.column_widths.get(col_idx).cloned().unwrap_or_default();
            
            match width_setting {
                ColumnWidth::Fixed(val) => {
                    let width = val.get_val(total_width as u16) as usize;
                    col_widths.push(width);
                    remaining_width = remaining_width.saturating_sub(width);
                },
                ColumnWidth::Percent(val) => {
                    let width = val.get_val(remaining_width as u16) as usize;
                    col_widths.push(width);
                    remaining_width = remaining_width.saturating_sub(width);
                },
                ColumnWidth::Auto => {
                    auto_cols.push(col_idx);
                    col_widths.push(0); // Placeholder
                }
            }
        }

        // Second pass: distribute remaining width among auto columns
        if !auto_cols.is_empty() {
            let width_per_auto = remaining_width / auto_cols.len();
            for &col_idx in &auto_cols {
                let mut max_content_width = 0;
                for row in &self.cells {
                    if let Some(cell) = row.get(col_idx) {
                        max_content_width = max_content_width.max(
                            self.calculate_cell_width(cell, dr)
                        );
                    }
                }
                col_widths[col_idx] = max_content_width.min(width_per_auto);
            }
        }

        // Calculate row heights
        for (row_idx, row) in self.cells.iter().enumerate() {
            let height = if row_idx < self.row_heights.len() {
                self.row_heights[row_idx].get_val(total_height as u16) as usize
            } else {
                let mut max_height = 1;
                for cell in row {
                    max_height = max_height.max(self.calculate_cell_height(cell, dr));
                }
                max_height
            };
            row_heights.push(height);
        }

        (col_widths, row_heights)
    }

    /// Get the background color for a cell based on row/column alternation
    fn get_cell_bg_color(&self, row: usize, col: usize) -> Option<CrosstermColor> {
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
    fn get_cell_fg_color(&self, row: usize) -> Option<CrosstermColor> {
        if row == 0 {
            self.style.header_fg_color
        } else {
            self.style.content_fg_color
        }
    }

    fn style_with_colors(&self, style: Style, row: usize, col: usize) -> Style {
        let mut style = style;
        if let Some(color) = self.get_cell_bg_color(row, col) {
            style = style.with_bg(Color::ANSI(color));
        }
        if let Some(color) = self.get_cell_fg_color(row) {
            style = style.with_fg(Color::ANSI(color));
        }
        style
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
        // Check if any cell can receive the event
        for row in &self.cells {
            for cell in row {
                if cell.can_receive(ev) {
                    return true;
                }
            }
        }

        // Also check table's own receivable events
        for rec in &self.receivable {
            if rec.borrow().can_receive(ev) {
                return true;
            }
        }
        false
    }

    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        self.receivable.clone()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        // Delegate event to focused cell if any
        for row in &self.cells {
            for cell in row {
                if cell.get_focused() {
                    return cell.receive_event(ctx, ev);
                }
            }
        }
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
                    let mut line_char = BoxDrawChars::HORIZONTAL;
                    if col_idx == 0 {
                        line_char = BoxDrawChars::DOWN_RIGHT;
                    } else if col_idx == col_widths.len() - 1 {
                        line_char = BoxDrawChars::DOWN_LEFT;
                    } else if self.style.vertical_lines {
                        line_char = BoxDrawChars::HORIZONTAL_DOWN;
                    }

                    let style = self.style_with_colors(line_char.style.clone(), row_idx, col_idx);
                    draw_chars.push(DrawChPos::new(
                        DrawCh::new(line_char.ch, style),
                        x.try_into().unwrap(),
                        y.try_into().unwrap()
                    ));
                    x += 1;

                    // Fill the rest of the column with horizontal lines
                    let style = self.style_with_colors(BoxDrawChars::HORIZONTAL.style.clone(), row_idx, col_idx);
                    for _ in 1..*width {
                        draw_chars.push(DrawChPos::new(
                            DrawCh::new(BoxDrawChars::HORIZONTAL.ch, style.clone()),
                            x.try_into().unwrap(),
                            y.try_into().unwrap()
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

                // Draw vertical line before cell if enabled
                if self.style.vertical_lines && col_idx > 0 {
                    let style = self.style_with_colors(BoxDrawChars::VERTICAL.style.clone(), row_idx, col_idx);
                    draw_chars.push(DrawChPos::new(
                        DrawCh::new(BoxDrawChars::VERTICAL.ch, style),
                        (x - 1).try_into().unwrap(),
                        y.try_into().unwrap()
                    ));
                }

                // Create child region for cell content
                let child_dr = dr.child_region(&location.l);
                let cell_updates = cell.drawing(ctx, &child_dr, force_update);
                
                // Apply cell updates with colors
                for update in cell_updates {
                    if let DrawAction::Update(chars) = update.action {
                        let style = self.style_with_colors(Style::transparent(), row_idx, col_idx);
                        for ch_pos in chars {
                            let mut updated_ch = ch_pos.ch;
                            updated_ch.style = style.clone();
                            draw_chars.push(DrawChPos::new(
                                updated_ch,
                                ch_pos.x,
                                ch_pos.y
                            ));
                        }
                    }
                }

                x += width;
            }

            // Draw horizontal line below cells if enabled
            if (row_idx > 0 || self.style.header_line) && 
               (row_idx < self.cells.len() - 1) && 
               self.style.horizontal_lines {
                y += row_heights[row_idx];
                let mut x = base_x;
                for (col_idx, width) in col_widths.iter().enumerate() {
                    let mut line_char = BoxDrawChars::HORIZONTAL;
                    if col_idx == 0 {
                        line_char = BoxDrawChars::VERTICAL_RIGHT;
                    } else if col_idx == col_widths.len() - 1 {
                        line_char = BoxDrawChars::VERTICAL_LEFT;
                    } else if self.style.vertical_lines {
                        line_char = BoxDrawChars::CROSS;
                    }

                    let style = self.style_with_colors(line_char.style.clone(), row_idx, col_idx);
                    draw_chars.push(DrawChPos::new(
                        DrawCh::new(line_char.ch, style),
                        x.try_into().unwrap(),
                        y.try_into().unwrap()
                    ));
                    x += 1;

                    let style = self.style_with_colors(BoxDrawChars::HORIZONTAL.style.clone(), row_idx, col_idx);
                    for _ in 1..*width {
                        draw_chars.push(DrawChPos::new(
                            DrawCh::new(BoxDrawChars::HORIZONTAL.ch, style.clone()),
                            x.try_into().unwrap(),
                            y.try_into().unwrap()
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

    fn get_attribute(&self, _key: &str) -> Option<Vec<u8>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color as CrosstermColor;
    use tokio::sync::mpsc::channel;

    fn create_test_context() -> Context {
        let sorting_hat = SortingHat::default();
        let (tx, _) = channel::<Event>(1);
        let color_store = ColorStore::default();
        Context::new_context_no_dur(&sorting_hat, tx, &color_store)
    }

    #[test]
    fn test_table_creation() {
        let ctx = create_test_context();
        let style = TableStyle::default();
        let table = Table::new(&ctx, style);

        assert_eq!(table.kind(), "table");
        assert!(table.get_visible());
    }

    #[test]
    fn test_table_data_setup() {
        let ctx = create_test_context();
        let style = TableStyle::default();
        let mut table = Table::new(&ctx, style);

        let header = vec!["ID", "Name", "Status"];
        let row1 = vec!["1", "Alice", "Active"];
        let row2 = vec!["2", "Bob", "Inactive"];

        let data = vec![header, row1, row2];
        table.set_data(&ctx, data);

        // Verify content width/height calculation
        assert!(table.get_content_width(None) == 0); // Without DrawRegion
        assert!(table.get_content_height(None) == 0); // Without DrawRegion
    }

    #[test]
    fn test_table_styling() {
        let ctx = create_test_context();
        let style = TableStyle {
            header_line: true,
            vertical_lines: true,
            horizontal_lines: true,
            header_fg_color: Some(CrosstermColor::White),
            header_bg_color: Some(CrosstermColor::Blue),
            content_fg_color: None,
            content_bg_color: None,
            row_alternation: Some((CrosstermColor::DarkGrey, 2)),
            column_alternation: None,
        };
        let mut table = Table::new(&ctx, style);

        // Set column widths with different strategies
        let widths = vec![
            ColumnWidth::Fixed(DynVal::new_fixed(4)),
            ColumnWidth::Percent(DynVal::new_flex(0.4)),
            ColumnWidth::Auto,
        ];
        table.set_column_widths(widths);

        // Set some row heights
        let heights = vec![
            DynVal::new_fixed(1), // Header row
            DynVal::new_fixed(2), // Content row
        ];
        table.set_row_heights(heights);
    }

    #[test]
    fn test_event_handling() {
        let ctx = create_test_context();
        let style = TableStyle::default();
        let table = Table::new(&ctx, style);

        // Test focusing
        assert!(!table.get_focused());
        table.set_focused(true);
        assert!(table.get_focused());

        // Test receivable events list
        assert!(table.receivable().is_empty());
    }
}
