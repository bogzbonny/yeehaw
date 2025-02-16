use {
    crate::*,
    box_drawing_logic::{BoxDrawingCh, SideAttribute as BoxSideAttr},
};

// TODO coloring for cells (such as alternating colors)
// TODO when the table has lines, those lines should be draggable
// TODO justification within cells
// TODO Equal setting for TableDimension
// TODO optionaly use underlined ansi for the table rows
//       - this will require fixed underlines
// TODO optionize the line style
// TODO left and right padding for the positions

// TODO do something better than using a PaneScrollable internally
//       - the header row should stay fixed
//       - the border should be fixed
//       - the border could have scrollbars
//         - potentially combine the internal border with the Border element
//           once the border element can nicely connect to internal box characters

/// A table container element that can display data in a grid format.
/// Each cell can contain any element
#[derive(Clone)]
pub struct Table {
    pub pane: PaneScrollable,
    pub column_dim: Rc<RefCell<TableDimension>>,
    pub row_dim: Rc<RefCell<TableDimension>>,
    #[allow(clippy::type_complexity)]
    pub cells: Rc<RefCell<Vec<Vec<Option<Box<dyn Element>>>>>>,
    pub style: Rc<RefCell<TableStyle>>,

    pub last_size: Rc<RefCell<Size>>,
    pub is_dirty: Rc<RefCell<bool>>,
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
    pub header_line: Option<BoxSideAttr>,
    /// Draw vertical lines between columns
    pub vertical_lines: Option<BoxSideAttr>,
    /// Draw horizontal lines between content rows
    pub horizontal_lines: Option<BoxSideAttr>,
    /// border around the table
    pub border: Option<BoxSideAttr>,
}

impl Table {
    pub const KIND: &'static str = "table";

    pub fn new(ctx: &Context) -> Self {
        // size of 1 is arbitrary for now it will be updated when the table is drawn
        let pane = PaneScrollable::new_expanding_with_kind(ctx, Self::KIND, 1, 1);
        Self {
            pane,
            column_dim: Rc::new(RefCell::new(TableDimension::Auto)),
            row_dim: Rc::new(RefCell::new(TableDimension::Auto)),
            cells: Rc::new(RefCell::new(Vec::new())),
            style: Rc::new(RefCell::new(TableStyle::default())),
            last_size: Rc::new(RefCell::new(Size::new(0, 0))),
            is_dirty: Rc::new(RefCell::new(true)),
        }
    }

    pub fn with_fixed_row_height(self, height: usize) -> Self {
        self.row_dim.replace(TableDimension::Fixed(height));
        self.is_dirty.replace(true);
        self
    }

    pub fn with_fixed_column_width(self, width: usize) -> Self {
        self.column_dim.replace(TableDimension::Fixed(width));
        self.is_dirty.replace(true);
        self
    }

    pub fn with_auto_column_width(self) -> Self {
        self.column_dim.replace(TableDimension::Auto);
        self.is_dirty.replace(true);
        self
    }

    pub fn with_auto_row_height(self) -> Self {
        self.row_dim.replace(TableDimension::Auto);
        self.is_dirty.replace(true);
        self
    }

    pub fn with_border(self, line_attr: Option<BoxSideAttr>) -> Self {
        self.style.borrow_mut().border = line_attr;
        self.is_dirty.replace(true);
        self
    }

    pub fn with_header_line(self, line_attr: Option<BoxSideAttr>) -> Self {
        self.style.borrow_mut().header_line = line_attr;
        self.is_dirty.replace(true);
        self
    }

    pub fn with_horizontal_lines(self, line_attr: Option<BoxSideAttr>) -> Self {
        self.style.borrow_mut().horizontal_lines = line_attr;
        self.is_dirty.replace(true);
        self
    }

    pub fn with_vertical_lines(self, line_attr: Option<BoxSideAttr>) -> Self {
        self.style.borrow_mut().vertical_lines = line_attr;
        self.is_dirty.replace(true);
        self
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

    /// doesn't clear the header (1st) row
    pub fn clear_data(&self) {
        let len = self.cells.borrow().len();
        for row in 1..len {
            self.remove_row(row);
        }
    }

    pub fn set_data(&self, ctx: &Context, data: Vec<Vec<&str>>) {
        for (row, row_data) in data.into_iter().enumerate() {
            self.set_row(ctx, row + 1, row_data); // row starts at 1 as 0 is reserved for the header row
        }
    }

    pub fn set_data_el(&self, data: Vec<Vec<Box<dyn Element>>>) {
        for (row, row_data) in data.into_iter().enumerate() {
            self.set_row_el(row + 1, row_data); // row starts at 1 as 0 is reserved for the header row
        }
    }

    /// set a row of cells
    pub fn set_row_el(&self, row: usize, data: Vec<Box<dyn Element>>) {
        let mut cells = self.cells.borrow_mut();
        if row >= cells.len() {
            cells.resize(row + 1, Vec::new());
        }
        let col_count = data.len();
        if cells[row].len() < col_count {
            cells[row].resize(col_count, None);
        }
        for (col, el) in data.into_iter().enumerate() {
            self.pane.add_element(el.clone());
            cells[row][col] = Some(el);
        }
        self.is_dirty.replace(true);
    }

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
            let el = Box::new(Label::new(ctx, s));
            self.pane.add_element(el.clone());
            cells[row][col] = Some(el);
        }
        self.is_dirty.replace(true);
    }

    pub fn set_column(&self, ctx: &Context, col: usize, data: Vec<&str>) {
        let mut cells = self.cells.borrow_mut();
        for (row, s) in data.into_iter().enumerate() {
            if row >= cells.len() {
                cells.resize(row, Vec::new());
            }
            if col >= cells[row].len() {
                cells[row].resize(col + 1, None);
            }
            let el = Box::new(Label::new(ctx, s));
            self.pane.add_element(el.clone());
            cells[row][col] = Some(el);
        }
        self.is_dirty.replace(true);
    }

    pub fn set_column_el(&self, col: usize, data: Vec<Box<dyn Element>>) {
        let mut cells = self.cells.borrow_mut();
        for (row, el) in data.into_iter().enumerate() {
            if row >= cells.len() {
                cells.resize(row, Vec::new());
            }
            if col >= cells[row].len() {
                cells[row].resize(col + 1, None);
            }
            self.pane.add_element(el.clone());
            cells[row][col] = Some(el);
        }
        self.is_dirty.replace(true);
    }

    /// push a row of data to the end of the table
    pub fn push_row(&self, ctx: &Context, data: Vec<&str>) {
        let row_count = self.cells.borrow().len();
        self.set_row(ctx, row_count, data);
    }

    pub fn push_row_el(&self, data: Vec<Box<dyn Element>>) {
        let row_count = self.cells.borrow().len();
        self.set_row_el(row_count, data);
    }

    pub fn push_column(&self, ctx: &Context, data: Vec<&str>) {
        let col_count = self.cells.borrow()[0].len();
        self.set_column(ctx, col_count, data);
    }

    pub fn push_column_el(&self, data: Vec<Box<dyn Element>>) {
        let col_count = self.cells.borrow()[0].len();
        self.set_column_el(col_count, data);
    }

    pub fn remove_row(&self, row: usize) {
        let mut cells = self.cells.borrow_mut();
        if row >= cells.len() - 1 {
            return;
        }
        let removed_row = cells.remove(row + 1);
        for cell in removed_row.into_iter().flatten() {
            self.pane.remove_element(&cell.id());
        }
        self.is_dirty.replace(true);
    }

    pub fn remove_column(&self, col: usize) {
        let mut cells = self.cells.borrow_mut();
        if col >= cells[0].len() {
            return;
        }
        for row in cells.iter_mut() {
            let cell = row.remove(col + 1);
            if let Some(cell) = cell {
                self.pane.remove_element(&cell.id());
            }
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
        self.pane.add_element(element.clone());
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
        let cells = self.cells.borrow();

        let end_row = cells.len();
        let end_col = cells.iter().map(|row| row.len()).max().unwrap_or(0);

        // compile the absolute col widths and height for each row and col
        // given the table TableDimension strategy
        match &*self.column_dim.borrow() {
            TableDimension::Fixed(size) => {
                for _ in 0..end_col {
                    col_widths.push(*size);
                }
            }
            TableDimension::Manual(sizes) => {
                for col in 0..end_col {
                    // compute the size
                    let size = sizes
                        .get(col)
                        .or_else(|| sizes.last())
                        .map(|size| size.get_val(dr.size.width));
                    match size {
                        Some(size) => {
                            if size < 1 {
                                col_widths.push(1);
                                continue;
                            }
                            col_widths.push(size as usize);
                        }
                        None => col_widths.push(1),
                    }
                }
            }
            TableDimension::Auto => {
                // first compile the max widths and heights for each row and col
                for col in 0..end_col {
                    col_widths.push(self.max_width_for_column(col));
                }
            }
        }
        match &*self.row_dim.borrow() {
            TableDimension::Fixed(size) => {
                for _ in 0..end_row {
                    row_heights.push(*size);
                }
            }
            TableDimension::Manual(sizes) => {
                for row in 0..end_row {
                    // compute the size
                    let size = sizes
                        .get(row)
                        .or_else(|| sizes.last())
                        .map(|size| size.get_val(dr.size.height));
                    match size {
                        Some(size) => {
                            if size < 1 {
                                row_heights.push(1);
                                continue;
                            }
                            row_heights.push(size as usize);
                        }
                        None => row_heights.push(1),
                    }
                }
            }
            TableDimension::Auto => {
                for row in 0..end_row {
                    row_heights.push(self.max_height_for_row(row));
                }
            }
        }

        let mut x = 0;
        let mut y = 0;
        let has_border = self.style.borrow().border.is_some();

        // first consider if there is a border
        if has_border {
            x += 1;
            y += 1;
        }
        let (mut max_x, mut max_y) = (x, y);

        // iterate through all the cells and set the position el.set_dyn_location(l) considering
        // border and lines positions
        for row in 0..end_row {
            let height = row_heights[row];
            for (col, width) in col_widths.iter().enumerate() {
                let cell = cells[row][col].as_ref().unwrap();
                cell.set_dyn_location(DynLocation::new(
                    x.into(),
                    (x + width).into(),
                    y.into(),
                    (y + height).into(),
                ));
                x += width;
                if max_x < x {
                    max_x = x;
                }

                // consider lines
                if self.style.borrow().vertical_lines.is_some() {
                    x += 1;
                }
            }
            y += height;
            if max_y < y {
                max_y = y;
            }
            x = if has_border { 1 } else { 0 };

            // consider lines
            if row == 0 {
                if self.style.borrow().header_line.is_some() {
                    y += 1;
                }
            } else if self.style.borrow().horizontal_lines.is_some() {
                y += 1;
            }
        }

        if has_border {
            max_x += 1;
            max_y += 1;
        }
        self.pane.set_content_width(max_x);
        self.pane.set_content_height(max_y);
        //debug!("setting max_x: {}, max_y: {}", max_x, max_y);

        let content_width = self.pane.get_content_width(Some(dr));
        let content_height = self.pane.get_content_height(Some(dr));

        // the content layer of the parent pane, will contains background colors a
        // and box drawing characters
        let mut content =
            DrawChs2D::new_empty_of_size(content_width, content_height, self.pane.pane.get_style());

        // TODO optionize the line style
        let line_sty = Style::transparent().with_fg(Color::WHITE);

        // Draw the horizontal header table line
        let mut y = if has_border { 1 } else { 0 };
        if let Some(line_attr) = self.style.borrow().header_line {
            let line = BoxDrawingCh::new_with_side_attr(true, true, false, false, line_attr);
            let ch = line.to_char_permissive().expect("box drawing logic broken");
            let ch = DrawCh::new(ch, line_sty.clone());

            let height = row_heights.first().unwrap_or(&0);
            y += height;
            for x in 0..content_width {
                content.set_ch(x, y, ch.clone());
            }
            y += 1; // account for the line
        }

        // Draw horizontal table lines, combining with the previous box drawing character at
        // intersecting positions
        if let Some(line_attr) = self.style.borrow().horizontal_lines {
            let line = BoxDrawingCh::new_with_side_attr(true, true, false, false, line_attr);
            let ch = line.to_char_permissive().expect("box drawing logic broken");
            let ch = DrawCh::new(ch, line_sty.clone());

            for (i, height) in row_heights.iter().enumerate() {
                if i == 0 || i == row_heights.len() - 1 {
                    continue; // skip header line and last line
                }
                y += height;
                for x in 0..content_width {
                    content.set_ch(x, y, ch.clone());
                }
                y += 1; // account for the line
            }
        }

        // Draw vertical table lines, combining with the previous box drawing character at
        // intersecting positions (aka horizontal lines)
        if let Some(line_attr) = self.style.borrow().vertical_lines {
            let mut x = if has_border { 1 } else { 0 };
            let line = BoxDrawingCh::new_with_side_attr(false, false, true, true, line_attr);
            let ch = line.to_char_permissive().expect("box drawing logic broken");
            let ch = DrawCh::new(ch, line_sty.clone());

            for (i, width) in col_widths.iter().enumerate() {
                x += width;
                if i == col_widths.len() - 1 {
                    continue; // skip the final line
                }
                for y in 0..content_height {
                    //for y in 0..dr.size.height as usize {
                    let mut ch_to_set = ch.clone();
                    'if_: {
                        let prev_ch = content.get_at(x, y);
                        let Some(prev_ch) = prev_ch else {
                            break 'if_;
                        };
                        let ChPlus::Char(prev_ch) = prev_ch.ch else {
                            break 'if_;
                        };
                        let Some(mut prev_box_ch) = BoxDrawingCh::from_char(prev_ch) else {
                            break 'if_;
                        };
                        prev_box_ch.overlay_with(line);
                        let Some(ch) = prev_box_ch.to_char_permissive() else {
                            break 'if_;
                        };
                        ch_to_set = DrawCh::new(ch, line_sty.clone());
                    }
                    content.set_ch(x, y, ch_to_set);
                }
                x += 1; // account for the line
            }
        }

        // Draw the border
        if let Some(line_attr) = self.style.borrow().border {
            // horizontal lines
            let line = BoxDrawingCh::new_with_side_attr(true, true, false, false, line_attr);
            let ch = line.to_char_permissive().expect("box drawing logic broken");
            let ch = DrawCh::new(ch, line_sty.clone());
            for y in [0, content_height - 1].iter() {
                for x in 0..content_width {
                    let mut ch_to_set = ch.clone();
                    'if_: {
                        let prev_ch = content.get_at(x, *y);
                        let Some(prev_ch) = prev_ch else {
                            break 'if_;
                        };
                        let ChPlus::Char(prev_ch) = prev_ch.ch else {
                            break 'if_;
                        };
                        let Some(mut prev_box_ch) = BoxDrawingCh::from_char(prev_ch) else {
                            break 'if_;
                        };
                        prev_box_ch.overlay_with(line);
                        let Some(ch) = prev_box_ch.to_char_permissive() else {
                            break 'if_;
                        };
                        ch_to_set = DrawCh::new(ch, line_sty.clone());
                    }
                    content.set_ch(x, *y, ch_to_set);
                }
            }

            // vertical lines
            let line = BoxDrawingCh::new_with_side_attr(false, false, true, true, line_attr);
            let ch = line.to_char_permissive().expect("box drawing logic broken");
            let ch = DrawCh::new(ch, line_sty.clone());
            for x in [0, content_width - 1].iter() {
                for y in 0..content_height {
                    let mut ch_to_set = ch.clone();
                    'if_: {
                        let prev_ch = content.get_at(*x, y);
                        let Some(prev_ch) = prev_ch else {
                            break 'if_;
                        };
                        let ChPlus::Char(prev_ch) = prev_ch.ch else {
                            break 'if_;
                        };
                        let Some(mut prev_box_ch) = BoxDrawingCh::from_char(prev_ch) else {
                            break 'if_;
                        };
                        prev_box_ch.overlay_with(line);
                        let Some(ch) = prev_box_ch.to_char_permissive() else {
                            break 'if_;
                        };
                        ch_to_set = DrawCh::new(ch, line_sty.clone());
                    }
                    content.set_ch(*x, y, ch_to_set);
                }
            }

            // trim the outermost box-drawing sides of the border
            // top
            let y = 0;
            for x in 0..content_width {
                let ch = content.get_at(x, y);
                let Some(ch) = ch else {
                    continue;
                };
                let ChPlus::Char(ch) = ch.ch else {
                    continue;
                };
                let ch = box_drawing_logic::remove_up(ch);
                content.set_ch(x, y, DrawCh::new(ch, line_sty.clone()));
            }

            // bottom
            let y = content_height - 1;
            for x in 0..content_width {
                let ch = content.get_at(x, y);
                let Some(ch) = ch else {
                    continue;
                };
                let ChPlus::Char(ch) = ch.ch else {
                    continue;
                };
                let ch = box_drawing_logic::remove_down(ch);
                content.set_ch(x, y, DrawCh::new(ch, line_sty.clone()));
            }

            // left
            let x = 0;
            for y in 0..content_height {
                let ch = content.get_at(x, y);
                let Some(ch) = ch else {
                    continue;
                };
                let ChPlus::Char(ch) = ch.ch else {
                    continue;
                };
                let ch = box_drawing_logic::remove_left(ch);
                content.set_ch(x, y, DrawCh::new(ch, line_sty.clone()));
            }

            // right
            let x = content_width - 1;
            for y in 0..content_height {
                let ch = content.get_at(x, y);
                let Some(ch) = ch else {
                    continue;
                };
                let ChPlus::Char(ch) = ch.ch else {
                    continue;
                };
                let ch = box_drawing_logic::remove_right(ch);
                content.set_ch(x, y, DrawCh::new(ch, line_sty.clone()));
            }
        }

        // Update the pane's content
        self.pane.pane.pane.set_content(content);
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
