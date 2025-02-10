use {
    crate::*,
    std::{collections::HashMap, rc::Rc},
};

/// A table container element that can display data in a grid format.
/// Each cell can contain any element, and strings are automatically converted to Label elements.
pub struct Table {
    pane: ParentPane,
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

/// Style configuration for table appearance.
/// All style options are optional and can be enabled/disabled independently.
#[derive(Clone, Debug, Default)]
pub struct TableStyle {
    /// Draw a horizontal line under the header row
    pub header_line: bool,
    /// Draw vertical lines between columns
    pub vertical_lines: bool,
    /// Draw horizontal lines between content rows
    pub horizontal_lines: bool,
    /// header style
    pub header_fg_color: Option<Color>,
    /// Foreground color for content cells
    pub content_fg_color: Option<Color>,
    /// Background color for content cells
    pub content_bg_color: Option<Color>,
    /// Alternating background colors for rows with a cycle length
    pub row_alternation: Option<(Color, usize)>,
    /// Alternating background colors for columns with a cycle length
    pub column_alternation: Option<(Color, usize)>,
}

impl Table {
    /// Create a new table with the given style
    pub fn new(ctx: &Context, style: TableStyle) -> Self {
        Self {
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

    pub fn set_elements<T>(&mut self, data: Vec<Vec<T>>)
    where
        T: Into<Box<dyn Element>>,
    {
        self.cells = data
            .into_iter()
            .map(|row| row.into_iter().map(Into::into).collect())
            .collect();
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
                }
                ColumnWidth::Percent(val) => {
                    let width = val.get_val(remaining_width as u16) as usize;
                    col_widths.push(width);
                    remaining_width = remaining_width.saturating_sub(width);
                }
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
                        max_content_width =
                            max_content_width.max(self.calculate_cell_width(cell, dr));
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

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Table {
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
                        y.try_into().unwrap(),
                    ));
                    x += 1;

                    // Fill the rest of the column with horizontal lines
                    let style = self.style_with_colors(
                        BoxDrawChars::HORIZONTAL.style.clone(),
                        row_idx,
                        col_idx,
                    );
                    for _ in 1..*width {
                        draw_chars.push(DrawChPos::new(
                            DrawCh::new(BoxDrawChars::HORIZONTAL.ch, style.clone()),
                            x.try_into().unwrap(),
                            y.try_into().unwrap(),
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
                    let style = self.style_with_colors(
                        BoxDrawChars::VERTICAL.style.clone(),
                        row_idx,
                        col_idx,
                    );
                    draw_chars.push(DrawChPos::new(
                        DrawCh::new(BoxDrawChars::VERTICAL.ch, style),
                        (x - 1).try_into().unwrap(),
                        y.try_into().unwrap(),
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
                            draw_chars.push(DrawChPos::new(updated_ch, ch_pos.x, ch_pos.y));
                        }
                    }
                }

                x += width;
            }

            // Draw horizontal line below cells if enabled
            if (row_idx > 0 || self.style.header_line)
                && (row_idx < self.cells.len() - 1)
                && self.style.horizontal_lines
            {
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
                        y.try_into().unwrap(),
                    ));
                    x += 1;

                    let style = self.style_with_colors(
                        BoxDrawChars::HORIZONTAL.style.clone(),
                        row_idx,
                        col_idx,
                    );
                    for _ in 1..*width {
                        draw_chars.push(DrawChPos::new(
                            DrawCh::new(BoxDrawChars::HORIZONTAL.ch, style.clone()),
                            x.try_into().unwrap(),
                            y.try_into().unwrap(),
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
}
