use {
    crate::{
        element::ReceivableEventChanges, Color, Context, DrawCh, DrawChPos, DynLocationSet, DynVal,
        Element, ElementID, Event, EventResponses, Keyboard as KB, Pane, Priority, SortingHat,
        Style, UpwardPropagator,
    },
    std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        path::PathBuf,
        rc::Rc,
    },
};

// TODO mouse functionality

// displays the size
#[derive(Clone)]
pub struct FileNavPane {
    pub pane: Pane,
    pub styles: Rc<RefCell<FileNavStyle>>,
    pub nav_items: Rc<RefCell<NavItems>>,
    pub highlight_position: Rc<RefCell<usize>>,
    pub offset: Rc<RefCell<usize>>,
    #[allow(clippy::type_complexity)]
    pub file_enter_hook: Rc<RefCell<Box<dyn FnMut(PathBuf)>>>,
}

#[derive(Clone)]
pub struct FileNavStyle {
    pub up_dir: Style,
    pub file: Style,
    pub folder: Style,
    pub top_dir: Style,
    pub background: Style,
    pub cursor_bg: Color,
}

// FileNavigator is a pane that displays a file navigator
impl Default for FileNavStyle {
    fn default() -> Self {
        FileNavStyle {
            up_dir: Style::default().with_fg(Color::GREEN),
            file: Style::default(),
            folder: Style::default().with_fg(Color::GREEN),
            top_dir: Style::default().with_fg(Color::RED),
            background: Style::default(),
            cursor_bg: Color::new(35, 45, 40),
        }
    }
}

impl FileNavPane {
    const INDENT_SIZE: usize = 2;

    pub fn default_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_ENTER.into(),
            KB::KEY_UP.into(),
            KB::KEY_DOWN.into(),
            KB::KEY_J.into(),
            KB::KEY_K.into(),
        ]
    }

    pub fn new(hat: &SortingHat, ctx: &Context, dir: PathBuf) -> Self {
        let styles = FileNavStyle::default();
        let pane = Pane::new(hat, "file_nav_pane");
        let up_dir = UpDir::new(".. (up a dir)".to_string(), styles.up_dir, 0);
        let top_dir = TopDir::new(
            Folder::new(dir, styles.folder, styles.file, 0, true),
            styles.top_dir,
        );

        let sub_items = top_dir.folder.sub_items(Self::INDENT_SIZE);
        let mut nav_items = NavItems(vec![NavItem::UpDir(up_dir), NavItem::TopDir(top_dir)]);
        nav_items.extend(sub_items);

        pane.self_evs
            .borrow_mut()
            .push_many_at_priority(Self::default_receivable_events(), Priority::FOCUSED);

        pane.set_dyn_height(DynVal::new_flex(1.));
        pane.set_dyn_width(DynVal::new_flex(1.));

        let out = Self {
            pane,
            styles: Rc::new(RefCell::new(styles)),
            nav_items: Rc::new(RefCell::new(nav_items)),
            highlight_position: Rc::new(RefCell::new(1)),
            offset: Rc::new(RefCell::new(0)),
            file_enter_hook: Rc::new(RefCell::new(Box::new(|_path| {}))),
        };
        out.update_content(ctx);
        out
    }

    pub fn set_open_fn(&self, file_enter_hook: Box<dyn FnMut(PathBuf)>) {
        *self.file_enter_hook.borrow_mut() = file_enter_hook;
    }

    pub fn update_content(&self, ctx: &Context) {
        let mut content = vec![vec![]];
        for (i, item) in self.nav_items.borrow().0.iter().enumerate() {
            if i < *self.offset.borrow() {
                continue;
            }
            let mut chs = item.draw(self.pane.default_ch.borrow().clone(), ctx.s.width.into());

            // cursor logic
            if i == *self.highlight_position.borrow() {
                for ch in chs.iter_mut() {
                    ch.style = ch.style.with_bg(self.styles.borrow().cursor_bg);
                }
            }
            content.push(chs);
        }
        *self.pane.content.borrow_mut() = content.into();
    }
}

impl Element for FileNavPane {
    fn kind(&self) -> &'static str {
        self.pane.kind()
    }
    fn id(&self) -> ElementID {
        self.pane.id()
    }
    fn receivable(&self) -> Vec<(Event, Priority)> {
        self.pane.receivable()
    }
    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.pane.receive_event(ctx, ev.clone());

        match ev {
            Event::KeyCombo(ke) => match true {
                _ if ke[0].matches_key(&KB::KEY_J) || ke[0].matches_key(&KB::KEY_DOWN) => {
                    if *self.highlight_position.borrow() < (*self.nav_items.borrow()).len() - 1 {
                        *self.highlight_position.borrow_mut() += 1;
                    }

                    // correct offsets
                    if *self.highlight_position.borrow()
                        >= *self.offset.borrow() + ctx.s.height as usize - 1
                    {
                        *self.offset.borrow_mut() += 1;
                    }
                }
                _ if ke[0].matches_key(&KB::KEY_K) || ke[0].matches_key(&KB::KEY_UP) => {
                    if *self.highlight_position.borrow() > 0 {
                        *self.highlight_position.borrow_mut() -= 1;
                    }

                    // correct offsets
                    if *self.highlight_position.borrow() < *self.offset.borrow() {
                        *self.offset.borrow_mut() -= 1;
                    }
                }
                _ if ke[0].matches_key(&KB::KEY_ENTER) => {
                    let ni = {
                        let nav_items = self.nav_items.borrow().clone();
                        self.nav_items.borrow_mut()[*self.highlight_position.borrow()].enter(
                            &nav_items,
                            &mut self.file_enter_hook.borrow_mut(),
                            *self.highlight_position.borrow(),
                        )
                    };
                    if let Some(ni) = ni {
                        *self.nav_items.borrow_mut() = ni;
                    }
                }
                _ => {}
            },
            Event::Refresh => {}
            _ => {}
        }
        (true, EventResponses::default())
    }
    fn change_priority(&self, ctx: &Context, p: Priority) -> ReceivableEventChanges {
        self.pane.change_priority(ctx, p)
    }
    fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
        self.update_content(ctx);
        self.pane.drawing(ctx)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }
    fn set_attribute(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute(key, value)
    }
    fn set_upward_propagator(&self, up: Box<dyn UpwardPropagator>) {
        self.pane.set_upward_propagator(up)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.pane.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.pane.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.pane.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.pane.call_hooks_of_kind(kind)
    }
    fn get_dyn_location_set(&self) -> Rc<RefCell<DynLocationSet>> {
        self.pane.get_dyn_location_set()
    }
    fn get_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_visible()
    }
}

#[derive(Clone)]
pub enum NavItem {
    File(File),
    Folder(Folder),
    TopDir(TopDir),
    UpDir(UpDir),
}

impl NavItem {
    pub fn sub_items(&self, indent_size: usize) -> Vec<NavItem> {
        match self {
            NavItem::File(_) => Vec::new(),
            NavItem::Folder(f) => f.sub_items(indent_size),
            NavItem::TopDir(_) => Vec::new(),
            NavItem::UpDir(_) => Vec::new(),
        }
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        match self {
            NavItem::File(f) => f.draw(default_ch, width),
            NavItem::Folder(f) => f.draw(default_ch, width, FileNavPane::INDENT_SIZE),
            NavItem::TopDir(f) => f.draw(default_ch, width),
            NavItem::UpDir(f) => f.draw(default_ch, width),
        }
    }

    pub fn enter(
        &mut self, nis: &NavItems, file_enter_hook: &mut Box<dyn FnMut(PathBuf)>,
        highlight_position: usize,
    ) -> Option<NavItems> {
        match self {
            NavItem::File(f) => f.enter(file_enter_hook),
            NavItem::Folder(f) => return Some(f.enter(nis, highlight_position)),
            NavItem::TopDir(_f) => {}
            NavItem::UpDir(_f) => {}
        }
        None
    }

    pub fn indentation(&self) -> usize {
        match self {
            NavItem::File(f) => f.indentation(),
            NavItem::Folder(f) => f.indentation(),
            NavItem::TopDir(_) => 0,
            NavItem::UpDir(f) => f.indentation(),
        }
    }
}

#[derive(Clone, Default)]
pub struct NavItems(Vec<NavItem>);

impl Deref for NavItems {
    type Target = Vec<NavItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NavItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl NavItems {
    /// adds the items at the insert position
    pub fn add_items(&mut self, insert_pos: usize, items: Vec<NavItem>) {
        if self.0.len() == insert_pos {
            self.0.extend(items);
        } else {
            self.0.splice(insert_pos..insert_pos, items);
        }
    }

    /// iterates the navItems beginning inclusively at startPos removing items until
    /// the indentation is less than the provided folderIndentation
    pub fn remove_items_at_indentation(&mut self, start_pos: usize, folder_indentation: usize) {
        // don't increment i because we are removing the elements
        for _ in start_pos..self.0.len() {
            if self.0[start_pos].indentation() <= folder_indentation {
                break; // hit something at the same folder level or higher, stop deleting
            }
            self.0.remove(start_pos);
        }
    }
}

#[derive(Clone)]
pub struct File {
    pub path: PathBuf,
    pub style: Style,
    pub indentation: usize,
}

impl File {
    pub fn new(path: PathBuf, style: Style, indentations: usize) -> File {
        File {
            path,
            style,
            indentation: indentations,
        }
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    // Execute the file enter hook
    pub fn enter(&self, file_enter_hook: &mut Box<dyn FnMut(PathBuf)>) {
        file_enter_hook(self.path.clone());
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        let mut out = Vec::new();
        let mut x = 0;
        while x < self.indentation {
            out.push(default_ch.clone());
            x += 1;
        }
        out.extend(self.name().chars().map(|c| DrawCh::new(c, self.style)));
        while x < width {
            out.push(default_ch.clone());
            x += 1;
        }
        out
    }
}

#[derive(Clone)]
pub struct Folder {
    pub path: PathBuf,
    pub folder_style: Style,
    pub file_style: Style,
    pub indentation: usize,
    pub is_expanded: bool,
}

impl Folder {
    pub fn new(
        path: PathBuf, folder_style: Style, file_style: Style, indentation: usize,
        is_expanded: bool,
    ) -> Folder {
        Folder {
            path,
            folder_style,
            file_style,
            indentation,
            is_expanded,
        }
    }

    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn sub_items(&self, indent_size: usize) -> Vec<NavItem> {
        let mut sub_items = Vec::new();
        let files = std::fs::read_dir(&self.path).unwrap();
        for file in files {
            let file = file.unwrap();
            if file.file_type().unwrap().is_dir() {
                let new_folder = Folder::new(
                    file.path(),
                    self.folder_style,
                    self.file_style,
                    self.indentation + indent_size,
                    false,
                );
                sub_items.push(NavItem::Folder(new_folder));
            } else {
                let new_file =
                    File::new(file.path(), self.file_style, self.indentation + indent_size);
                sub_items.push(NavItem::File(new_file));
            }
        }
        sub_items.sort_by(|a, b| match (a, b) {
            (NavItem::Folder(_), NavItem::File(_)) => std::cmp::Ordering::Less,
            (NavItem::File(_), NavItem::Folder(_)) => std::cmp::Ordering::Greater,
            (NavItem::File(a), NavItem::File(b)) => {
                a.name().to_lowercase().cmp(&b.name().to_lowercase())
            }
            (NavItem::Folder(a), NavItem::Folder(b)) => {
                a.name().to_lowercase().cmp(&b.name().to_lowercase())
            }
            (_, _) => std::cmp::Ordering::Equal,
        });
        sub_items
    }

    pub fn enter(&self, nis: &NavItems, highlight_position: usize) -> NavItems {
        let mut nis = nis.clone();
        if !self.is_expanded {
            let sub_items = self.sub_items(FileNavPane::INDENT_SIZE);
            nis.add_items(highlight_position + 1, sub_items);
        } else {
            nis.remove_items_at_indentation(highlight_position + 1, self.indentation);
        }
        if let Some(NavItem::Folder(f)) = nis.get_mut(highlight_position) {
            f.is_expanded = !f.is_expanded;
        }
        nis
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize, indent_size: usize) -> Vec<DrawCh> {
        let mut out = Vec::new();
        let mut x = 0;
        while x < self.indentation {
            if x == self.indentation - indent_size {
                if self.is_expanded {
                    out.push(DrawCh::new('▾', self.folder_style));
                } else {
                    out.push(DrawCh::new('▸', self.folder_style));
                }
            } else {
                out.push(default_ch.clone());
            }
            x += 1;
        }
        out.extend(
            self.name()
                .chars()
                .map(|c| DrawCh::new(c, self.folder_style)),
        );
        while x < width {
            out.push(default_ch.clone());
            x += 1;
        }
        out
    }
}

#[derive(Clone)]
pub struct TopDir {
    pub folder: Folder,
    pub sty: Style,
}

impl TopDir {
    pub fn new(folder: Folder, sty: Style) -> TopDir {
        TopDir { folder, sty }
    }
    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        let mut out = DrawCh::str_to_draw_chs(&self.folder.name(), self.sty);
        for _ in out.len()..width {
            out.push(default_ch.clone());
        }
        out
    }
}

#[derive(Clone)]
pub struct UpDir {
    pub text: String,
    pub sty: Style,
    pub indentation: usize,
}

impl UpDir {
    pub fn new(text: String, sty: Style, indentation: usize) -> UpDir {
        UpDir {
            text,
            sty,
            indentation,
        }
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        let mut out = DrawCh::str_to_draw_chs(&self.text, self.sty);
        for _ in out.len()..width {
            out.push(default_ch.clone());
        }
        out
    }
}
