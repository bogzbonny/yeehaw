/// The debugger is a utility intended to be used to write TUI debug information
/// to a seperate file such that debug output may be read while the a TUI is
/// running.
///
/// DebugFilepath is the path to the debug file. If this is empty, no debug
/// information will be written.
/// The debug filepath is specified at the top of the main file of the package
/// being debugged
use {once_cell::sync::Lazy, parking_lot::RwLock, std::fs::OpenOptions, std::io::prelude::*};

//let s = std::fmt::format(std::alloc::__export::format_args!($($arg)*));
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::debug::log(s);
    }}
}

#[derive(Clone)]
pub struct Debugger {
    pub log_file: Option<String>, // if some then output debug to file
    pub enabled: bool,
    pub max_lines: usize,
    pub lines: Vec<String>,
}

static GLOBAL_DEBUGGER: Lazy<RwLock<Debugger>> = Lazy::new(|| {
    RwLock::new(Debugger {
        log_file: None,
        enabled: true,
        max_lines: 300,
        lines: Vec::new(),
    })
});

pub fn get_content() -> String {
    GLOBAL_DEBUGGER.read().lines.join("\n")
}
pub fn get_max_lines() -> usize {
    (GLOBAL_DEBUGGER.read()).max_lines
}
pub fn is_enabled() -> bool {
    (GLOBAL_DEBUGGER.write()).enabled
}

pub fn enable() {
    (GLOBAL_DEBUGGER.write()).enabled = true;
}

pub fn disable() {
    (GLOBAL_DEBUGGER.write()).enabled = false;
}

pub fn set_log_file(file: String) {
    (GLOBAL_DEBUGGER.write()).log_file = Some(file);
}

pub fn log(content: String) {
    if !GLOBAL_DEBUGGER.read().enabled {
        return;
    }

    let mut lines = GLOBAL_DEBUGGER.read().lines.clone();
    let max_lines = GLOBAL_DEBUGGER.read().max_lines;
    lines.push(content.clone());
    if lines.len() > max_lines {
        lines.remove(0);
    }
    (GLOBAL_DEBUGGER.write()).lines = lines;

    // push to file
    if let Some(file) = GLOBAL_DEBUGGER.read().log_file.clone() {
        // append content to new line at end of file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .unwrap();
        writeln!(file, "{content}").unwrap();
    }
}

pub fn clear() {
    (GLOBAL_DEBUGGER.write()).lines.clear();

    // clear file
    if let Some(file) = &(GLOBAL_DEBUGGER.write()).log_file {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .unwrap();
        file.set_len(0).unwrap();
    }
}
