use {
    crate::{
        element::ReceivableEventChanges, widgets::TextBox, Context, DrawChPos, DynLocationSet,
        DynVal, Element, ElementID, Event, EventResponses, Pane, Priority, SortingHat, Style,
        UpwardPropagator, WidgetPane,
    },
    std::path::Path,
    std::{cell::RefCell, rc::Rc},
};

// TODO mouse functionality

// displays the size
#[derive(Clone)]
pub struct FileViewerPane {
    pub pane: Pane,
    pub styles: Rc<RefCell<FileNavStyle>>,
    pub nav_items: Rc<RefCell<NavItems>>,
    pub highlight_position: Rc<RefCell<usize>>,
    pub offset: Rc<RefCell<usize>>,
    pub indent_size: Rc<RefCell<usize>>,
    pub file_enter_hook: Rc<RefCell<Box<dyn FnMut(String)>>>,
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
            up_dir: Style::default().fg(Color::Green),
            file: Style::default(),
            folder: Style::default().fg(Color::Green),
            top_dir: Style::default().fg(Color::Red),
            background: Style::default(),
            cursor_bg: Color::new(35, 45, 40),
        }
    }
}

impl FileNavStyle {
    pub fn default_receivable_events() -> Vec<Event> {
        vec![
            KB::KEY_ENTER.into(),
            KB::KEY_UP.into(),
            KB::KEY_DOWN.into(),
            KB::KEY_J.into(),
            KB::KEY_K.into(),
        ]
    }

    pub fn new(
        hat: &SortingHat, ctx: &Context, styles: FileNavStyle,
        file_enter_hook: Box<dyn FnMut(String)>,
    ) -> Self {
        let default_ch = DrawCh::new(' ', false, styles.background);
        let default_line = Vec::new();
        let content = vec![default_line.clone()];
        let pane = Pane::new(hat, ctx, content, default_ch, default_line, 0, 0);
        let up_dir = UpDir::new(".. (Up a directory)".to_string(), styles.up_dir, 0);
        let top_dir = TopDir::new(Folder::new());
        let nav_items = NavItems(vec![top_dir, up_dir]);

        pane.self_evs
            .borrow_mut()
            .push_many_at_priority(Self::default_receivable_events(), Priority::FOCUSED);

        Self {
            pane,
            styles,
            nav_items,
            highlight_position: 1,
            offset: 0,
            indent_side: 2,
            file_enter_hook,
        }
    }

    pub fn update_content(&self, ctx: &Context) {
        let content = vec![vec![]];
        for (i, item) in nav_items.iter().enumerate() {
            if i < self.offset {
                continue;
            }
            let mut chs = item.draw(self.default_ch, ctx.s.width);

            // cursor logic
            if i == self.highlight_position {
                for (i, mut ch) in chs {
                    ch.style = ch.style.with_bg(self.styles.cursor_bg);
                }
            }
        }
        self.pane.content.borrow_mut() = content;
    }
}

/*
// UpdateNavItems - basic filenav movements
func (fn *FileNavigator) ReceiveEventCombo(ctx yh.Context, evs []*tcell.EventKey) (
	evResp yh.EventResponse) {

	switch {
	case yh.JLowerEKC.Matches(evs) || yh.DownEKC.Matches(evs):
		if fn.HighlightPosition < len(fn.NIs)-1 {
			fn.HighlightPosition++
			if fn.HighlightPosition > fn.Offset+ctx.S.Height-1 {
				fn.Offset++
			}
		}
	case yh.KLowerEKC.Matches(evs) || yh.UpEKC.Matches(evs):
		if fn.HighlightPosition > 0 {
			fn.HighlightPosition--
			if fn.HighlightPosition < fn.Offset {
				fn.Offset--
			}
		}
	case yh.EnterEKC.Matches(evs):
		fn.NIs[fn.HighlightPosition].Enter(&fn.NIs, fn.FileEnterHook, fn.HighlightPosition)
	}
	return evResp
}
*/

impl Element for FileNavStyle {
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
        self.pane.receive_event(ctx, ev.clone())

        match ev {
            Event::KeyCombo(ke) => {
                match true {
                    _ if ke[0].matches_key(&KB::KEY_J) || ke[0].matches_key(&KB::KEY_DOWN)=> {
                        if self.highlight_position < self.nav_items.len() - 1 {
                            self.highlight_position += 1;


                        }
                    }
                }
            }
            Event::Refresh => {}
            _ => {}
        }
        self.pane.receive_event(ctx, ev)

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

pub enum NavItem {
    File(File),
    Folder(Folder),
    TopDir(TopDir),
    UpDir(UpDir),
}

impl NavItem {
    pub fn sub_items(&self) -> Vec<NavItem> {
        match self {
            NavItem::File(f) => Vec::new(),
            NavItem::Folder(f) => f.sub_items(),
            NavItem::TopDir(f) => Vec::new(),
            NavItem::UpDir(f) => Vec::new(),
        }
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        match self {
            NavItem::File(f) => f.draw(default_ch, width),
            NavItem::Folder(f) => f.draw(default_ch, width, 4),
            NavItem::TopDir(f) => f.draw(default_ch, width),
            NavItem::UpDir(f) => f.draw(default_ch, width),
        }
    }

    pub fn enter(&self, NIs: &mut NavItems, file_enter_hook: Box<dyn FnMut(String)>) {
        match self {
            NavItem::File(f) => f.enter(file_enter_hook),
            NavItem::Folder(f) => f.Enter(NIs, file_enter_hook, 0),
            NavItem::TopDir(f) => {}
            NavItem::UpDir(f) => {}
        }
    }

    pub fn indentation(&self) -> usize {
        match self {
            NavItem::File(f) => f.indentation(),
            NavItem::Folder(f) => f.indentation(),
            NavItem::TopDir(f) => 0,
            NavItem::UpDir(f) => f.indentation(),
        }
    }
}

pub struct NavItems(Vec<NavItem>);

impl NavItems {
    pub fn new() -> NavItems {
        NavItems(Vec::new())
    }

    /// adds the items at the insert position
    pub fn add_items(&mut self, insert_pos: usize, items: Vec<NavItem>) {
        if self.0.len() == insert_pos {
            self.0.extend(items);
        } else {
            let end_half = items;
            self.0.splice(insert_pos.., end_half);
        }
    }

    /// iterates the navItems beginning inclusively at startPos removing items until
    /// the indentation is less than the provided folderIndentation
    pub fn remove_items_at_indentation(&mut self, start_pos: usize, folder_indentation: usize) {
        // don't increment i because we are removing the elements
        for i in start_pos..self.0.len() {
            if self.0[i].indentation() <= folder_indentation {
                break; // hit something at the same folder level or higher, stop deleting
            }
            // remove the item (at i)
            self.0.remove(i);
        }
    }
}

pub struct File {
    pub path: Path,
    pub style: Style,
    pub indentation: usize,
}

impl File {
    pub fn new(path: String, style: Style, indentations: usize) -> File {
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
    pub fn enter(&self, file_enter_hook: Box<dyn FnMut(String)>) {
        file_enter_hook(self.path.to_str().unwrap().to_string());
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        let mut out = Vec::new();
        let mut x = 0;
        while x < self.indentation {
            out.push(default_ch);
            x += 1;
        }
        out.extend(
            self.name()
                .chars()
                .map(|c| DrawCh::new(c, false, self.style)),
        );
        while x < width {
            out.push(default_ch);
            x += 1;
        }
        out
    }
}

pub struct Folder {
    pub path: Path,
    pub folder_style: Style,
    pub file_style: Style,
    pub indentation: usize,
    pub is_expanded: bool,
}

impl Folder {
    pub fn new(
        path: Path, folder_style: Style, file_style: Style, indentation: usize, is_expanded: bool,
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

    pub fn sub_items(&self) -> Vec<NavItem> {
        let mut sub_items = Vec::new();
        let files = std::fs::read_dir(&self.path).unwrap();
        for file in files {
            let file = file.unwrap();
            if file.file_type().unwrap().is_dir() {
                let new_folder = Folder::new(
                    file.path(),
                    self.folder_style.clone(),
                    self.file_style.clone(),
                    self.indentation + 4,
                    false,
                );
                sub_items.push(NavItem::Folder(new_folder));
            } else {
                let new_file = File::new(
                    file.path().to_str().unwrap().to_string(),
                    self.file_style.clone(),
                    self.indentation + 4,
                );
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
        });
        sub_items
    }

    pub fn Enter(
        &mut self, NIs: &mut NavItems, file_enter_hook: Box<dyn FnMut(String)>,
        highlight_position: usize,
    ) {
        let sub_items = self.sub_items();
        if !self.is_expanded {
            NIs.add_items(highlight_position + 1, sub_items);
        } else {
            let folder_indentation = self.indentation;
            NIs.remove_items_at_indentation(highlight_position + 1, folder_indentation);
        }
        self.is_expanded = !self.is_expanded;
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize, indent_size: usize) -> Vec<DrawCh> {
        let mut out = Vec::new();
        let mut x = 0;
        while x < self.indentation {
            if x == self.indentation - indent_size {
                if self.is_expanded {
                    out.push(DrawCh::new('▾', false, self.folder_style));
                } else {
                    out.push(DrawCh::new('▸', false, self.folder_style));
                }
            } else {
                out.push(default_ch);
            }
            x += 1;
        }
        out.extend(
            self.name()
                .chars()
                .map(|c| DrawCh::new(c, false, self.folder_style)),
        );
        while x < width {
            out.push(default_ch);
            x += 1;
        }
        out
    }
}

pub struct TopDir {
    pub folder: Folder,
    pub top_dir_style: Style,
}

impl TopDir {
    pub fn new(folder: Folder, style: Style) -> TopDir {
        TopDir { folder, s: style }
    }
    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        let mut out = DrawCh::str_to_draw_chs(self.folder.name(), self.s);
        for i in out.len()..width {
            out.push(default_ch);
        }
        out
    }
}

pub struct UpDir {
    pub text: String,
    pub style: Style,
    pub indentation: usize,
}

impl UpDir {
    pub fn new(text: String, style: Style, indentation: usize) -> UpDir {
        UpDir {
            text,
            style,
            indentation,
        }
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn draw(&self, default_ch: DrawCh, width: usize) -> Vec<DrawCh> {
        let mut out = DrawCh::str_to_draw_chs(self.text, self.style);
        for i in out.len()..width {
            out.push(default_ch);
        }
        out
    }
}
