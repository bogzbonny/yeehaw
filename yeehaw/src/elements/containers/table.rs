use crate::*;

// TODO coloring for cells
// TODO when the table has lines, those lines should be draggable
// TODO justification within cells
// TODO Equal setting for TableDimension

/// A table container element that can display data in a grid format.
/// Each cell can contain any element
#[derive(Clone)]
pub struct Table {
    pub pane: ParentPane,
    pub column_dim: Rc<RefCell<TableDimension>>,
    pub row_dim: Rc<RefCell<TableDimension>>,
    pub cells: Rc<RefCell<Vec<Vec<Option<Box<dyn Element>>>>>>,
    pub style: Rc<RefCell<TableStyle>>,
    pub last_size: Rc<RefCell<Size>>,
    pub is_dirty: Rc<RefCell<bool>>,
}

#[derive(Clone, Debug)]
pub enum TableLineKind {
    Thin,
    Thick,
    Double,
}

pub enum TableDimension {
    /// Fixed number of chs
    Fixed(usize),

    /// manual sizes for each column/row if there are more cols/rows than sizes in this
    /// vector, the last size will be used for the remaining cols/rows
    Manual(Vec<DynVal>),

    /// Automatically determined by the size of the content within each cell
    Auto,
}

/// Style configuration for table appearance.
/// All style options are optional and can be enabled/disabled independently.
#[derive(Clone, Debug, Default)]
pub struct TableStyle {
    /// Draw a horizontal line under the header row
    pub header_line: Option<TableLineKind>,
    /// Draw vertical lines between columns
    pub vertical_lines: Option<TableLineKind>,
    /// Draw horizontal lines between content rows
    pub horizontal_lines: Option<TableLineKind>,
    /// border around the table
    pub border: Option<TableLineKind>,
}

impl Table {
    pub fn new(ctx: &Context) -> Self {
        Self {
            pane: ParentPane::new(ctx, "table"),
            column_dim: Rc::new(RefCell::new(TableDimension::Auto)),
            row_dim: Rc::new(RefCell::new(TableDimension::Auto)),
            cells: Rc::new(RefCell::new(Vec::new())),
            style: Rc::new(RefCell::new(TableStyle::default())),
            last_size: Rc::new(RefCell::new(Size::new(0, 0))),
            is_dirty: Rc::new(RefCell::new(true)),
        }
    }

    pub fn set_cell(&self, ctx: &Context, row: usize, col: usize, text: &str) {
        self.set_element(row, col, Box::new(Label::new(ctx, text)));
        self.is_dirty.replace(true);
    }

    /// set the header row
    pub fn set_header(&self, ctx: &Context, data: Vec<&str>) {
        self.set_row(ctx, 0, data);
        self.is_dirty.replace(true);
    }

    /// set a row of cells
    pub fn set_row(&self, ctx: &Context, row: usize, data: Vec<&str>) {
        let mut cells = self.cells.borrow_mut();
        if row >= cells.len() {
            cells.resize(row + 1, Vec::new());
        }
        let col_count = data.len();
        if cells[row].len() < col_count {
            cells[row].resize(col_count, None);
        }
        for (col, s) in data.into_iter().enumerate() {
            cells[row][col] = Some(Box::new(Label::new(ctx, s)));
        }
        self.is_dirty.replace(true);
    }

    pub fn set_column(&self, ctx: &Context, col: usize, data: Vec<&str>) {
        let mut cells = self.cells.borrow_mut();
        for (row, s) in data.into_iter().enumerate() {
            if row >= cells.len() {
                cells.resize(row + 1, Vec::new());
            }
            if col >= cells[row].len() {
                cells[row].resize(col + 1, None);
            }
            cells[row][col] = Some(Box::new(Label::new(ctx, s)));
        }
        self.is_dirty.replace(true);
    }

    pub fn set_element(&self, row: usize, col: usize, element: Box<dyn Element>) {
        let mut cells = self.cells.borrow_mut();
        if row >= cells.len() {
            cells.resize(row + 1, Vec::new());
        }
        if col >= cells[row].len() {
            cells[row].resize(col + 1, None);
        }
        cells[row][col] = Some(element);
        self.is_dirty.replace(true);
    }

    pub fn max_width_for_column(&self, col: usize) -> usize {
        let cells = self.cells.borrow();
        let dummy_dr = DrawRegion::default().with_size(Size::new(1, 1));
        cells
            .iter()
            .filter_map(|row| row.get(col))
            .map(|cell| {
                cell.as_ref()
                    .map(|cell| cell.get_dyn_location_set().get_width_val(&dummy_dr))
                    .unwrap_or(1)
            })
            .max()
            .unwrap_or(1)
    }

    pub fn max_height_for_row(&self, row: usize) -> usize {
        let cells = self.cells.borrow();
        let dummy_dr = DrawRegion::default().with_size(Size::new(1, 1));
        cells
            .get(row)
            .map(|row| {
                row.iter()
                    .map(|cell| {
                        cell.as_ref()
                            .map(|cell| cell.get_dyn_location_set().get_height_val(&dummy_dr))
                            .unwrap_or(1)
                    })
                    .max()
                    .unwrap_or(1)
            })
            .unwrap_or(1)
    }

    /// repositions all elements within the table
    pub fn reposition_all_elements_redraw_lines(&self, dr: &DrawRegion) {
        // determine the positions of each cell and draw the lines at the same time
        let mut col_widths = Vec::new();
        let mut row_heights = Vec::new();
        let mut vertical_line_positions: Vec<usize> = Vec::new();
        let mut horizontal_line_positions: Vec<usize> = Vec::new();
        let cells = self.cells.borrow();

        let max_widths = Vec::new();
        let max_heights = Vec::new();
        let end_row = cells.len();
        let end_col = cells.iter().map(|row| row.len()).max().unwrap_or(0);

        // compile the absolute col widths and height for each row and col
        // given the table TableDimension strategy
        match &*self.column_dim.borrow() {
            TableDimension::Fixed(size) => {
                for col in 0..end_col {
                    col_widths.push((*size).into());
                }
            }
            TableDimension::Manual(sizes) => {
                col_widths = sizes.clone();
            }
            TableDimension::Auto => {
                // first compile the max widths and heights for each row and col
                for col in 0..end_col {
                    col_widths.push(self.max_width_for_column(col).into());
                }
            }
        }
        match &*self.row_dim.borrow() {
            TableDimension::Fixed(size) => {
                for row in 0..end_row {
                    row_heights.push((*size).into());
                }
            }
            TableDimension::Manual(sizes) => {
                row_heights = sizes.clone();
            }
            TableDimension::Auto => {
                for row in 0..end_row {
                    row_heights.push(self.max_height_for_row(row).into());
                }
            }
        }

        let content = DrawChs2D::new_empty_of_size(
            dr.size.width.into(),
            dr.size.height.into(),
            self.pane.pane.get_style(),
        );

        // iterate through all the cells and set the position el.set_dyn_location(l) for each of them
        // set the content for all the lines, given the TableLineKind
        // and the position for each of the lines

        let mut x = 0;
        let mut y = 0;

        // first consider if there is a border

        for row in 0..end_row {
            let mut x = 0;
            let row_height = row_heights
                .get(row)
                .cloned()
                .unwrap_or_else(|| row_heights.last().cloned().unwrap_or(1.into()));
            let row_height = row_height.get_val(dr.size.height);

            for col in 0..end_col {
                let col_width = col_widths
                    .get(col)
                    .cloned()
                    .unwrap_or_else(|| col_widths.last().cloned().unwrap_or(1.into()));
                let col_width = col_width.get_val(dr.size.width);

                if let Some(Some(cell)) = cells.get(row).and_then(|r| r.get(col)) {
                    let loc = DynLocation::new(
                        x.into(),
                        x.into() + col_width,
                        y.into(),
                        y.into() + row_height,
                    );
                    cell.set_dyn_location(loc);
                }

                x += col_width;
                vertical_line_positions.push(x as usize);
            }

            y += row_height;
            horizontal_line_positions.push(y as usize);
        }

        // Draw table lines based on style
        let style = self.style.borrow();

        // Helper function to get line characters based on style
        let get_line_chars = |line_kind: &TableLineKind| -> (char, char, char, char, char, char) {
            match line_kind {
                TableLineKind::Thin => ('─', '│', '┌', '┐', '└', '┘'),
                TableLineKind::Thick => ('━', '┃', '┏', '┓', '┗', '┛'),
                TableLineKind::Double => ('═', '║', '╔', '╗', '╚', '╝'),
            }
        };

        // Draw border if specified
        if let Some(border_style) = &style.border {
            let (h, v, tl, tr, bl, br) = get_line_chars(border_style);
            // Top border
            for x in 0..dr.size.width {
                content.set_ch(x.into(), 0, DrawCh::new(h, self.pane.pane.get_style()));
            }
            // Bottom border
            for x in 0..dr.size.width {
                content.set_ch(
                    x.into(),
                    dr.size.height.into() - 1,
                    DrawCh::new(h, self.pane.pane.get_style()),
                );
            }
            // Left border
            for y in 0..dr.size.height {
                content.set_ch(0, y.into(), DrawCh::new(v, self.pane.pane.get_style()));
            }
            // Right border
            for y in 0..dr.size.height {
                content.set_ch(
                    dr.size.width.into() - 1,
                    y.into(),
                    DrawCh::new(v, self.pane.pane.get_style()),
                );
            }
            // Corners
            content.set_ch(0, 0, DrawCh::new(tl, self.pane.pane.get_style()));
            content.set_ch(
                dr.size.width.into() - 1,
                0,
                DrawCh::new(tr, self.pane.pane.get_style()),
            );
            content.set_ch(
                0,
                dr.size.height.into() - 1,
                DrawCh::new(bl, self.pane.pane.get_style()),
            );
            content.set_ch(
                dr.size.width.into() - 1,
                dr.size.height.into() - 1,
                DrawCh::new(br, self.pane.pane.get_style()),
            );
        }

        // Draw vertical lines
        if let Some(line_style) = &style.vertical_lines {
            let (_, v, _, _, _, _) = get_line_chars(line_style);
            for x in &vertical_line_positions {
                for y in 0..dr.size.height {
                    content.set_ch(*x, y.into(), DrawCh::new(v, self.pane.pane.get_style()));
                }
            }
        }

        // Draw horizontal lines
        if let Some(line_style) = &style.horizontal_lines {
            let (h, _, _, _, _, _) = get_line_chars(line_style);
            for y in &horizontal_line_positions {
                for x in 0..dr.size.width {
                    content.set_ch(x.into(), *y, DrawCh::new(h, self.pane.pane.get_style()));
                }
            }
        }

        // Draw header line if specified
        if let Some(header_style) = &style.header_line {
            let (h, _, _, _, _, _) = get_line_chars(header_style);
            if let Some(first_row_height) = row_heights.first() {
                let header_y = first_row_height.get_val(&dr);
                for x in 0..dr.size.width {
                    content.set_ch(
                        x.into(),
                        header_y as usize,
                        DrawCh::new(h, self.pane.pane.get_style()),
                    );
                }
            }
        }

        // Update the pane's content
        self.pane.pane.set_content(content);
    }

    pub fn ensure_correct_positions(&self, dr: &DrawRegion) {
        if *self.last_size.borrow() != dr.size || self.is_dirty.replace(false) {
            self.reposition_all_elements_redraw_lines(dr);
            *self.last_size.borrow_mut() = dr.size;
        }
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for Table {
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        self.ensure_correct_positions(dr);
        self.pane.drawing(ctx, dr, force_update)
    }
}
