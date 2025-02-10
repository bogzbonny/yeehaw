use {crate::*, std::rc::Rc};

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
    pub fn reposition_all_elements(&self, dr: &DrawRegion) {}

    /// repositions all elements within the table
    pub fn reposition_all_elements(&self, dr: &DrawRegion) {}

    pub fn ensure_correct_positions(&self, dr: &DrawRegion) {
        if *self.last_size.borrow() != dr.size || self.is_dirty.replace(false) {
            self.reposition_all_elements(dr);

            // XXX redraw the content for the lines

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
