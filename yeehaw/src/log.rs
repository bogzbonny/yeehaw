/// The Logger is a utility intended to be used to write TUI debug information
/// to a seperate file such that debug output may be read while the a TUI is
/// running.
///
/// DebugFilepath is the path to the debug file. If this is empty, no debug
/// information will be written.
/// The debug filepath is specified at the top of the main file of the package
/// being debugged
use {once_cell::sync::Lazy, parking_lot::RwLock, std::fs::OpenOptions, std::io::prelude::*};

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::log::log(s);
    }}
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::log::log_or_panic(s);
    }}
}

#[derive(Clone)]
pub struct Logger {
    pub log_file: Option<String>, // if some then output to this file
    pub enabled: bool,
    pub max_lines: usize,
    pub lines: Vec<String>,
}

static GLOBAL_LOGGER: Lazy<RwLock<Logger>> = Lazy::new(|| {
    RwLock::new(Logger {
        log_file: None,
        enabled: true,
        max_lines: 300,
        lines: Vec::new(),
    })
});

pub fn get_content() -> String {
    GLOBAL_LOGGER.read().lines.join("\n")
}
pub fn get_max_lines() -> usize {
    (GLOBAL_LOGGER.read()).max_lines
}
pub fn is_enabled() -> bool {
    (GLOBAL_LOGGER.write()).enabled
}

pub fn enable() {
    (GLOBAL_LOGGER.write()).enabled = true;
}

pub fn disable() {
    (GLOBAL_LOGGER.write()).enabled = false;
}

pub fn set_log_file(file: String) {
    (GLOBAL_LOGGER.write()).log_file = Some(file);
}

pub fn reset_log_file(file: String) {
    (GLOBAL_LOGGER.write()).log_file = Some(file);
    clear();
}

/// log or panic either logs the content or panics if the build mode is non-release
pub fn log_or_panic(content: String) {
    log(content.clone());
    #[cfg(debug_assertions)]
    panic!("{}", content);
}

pub fn log(content: String) {
    if !GLOBAL_LOGGER.read().enabled {
        return;
    }

    let mut lines = GLOBAL_LOGGER.read().lines.clone();
    let max_lines = GLOBAL_LOGGER.read().max_lines;
    lines.push(content.clone());
    if lines.len() > max_lines {
        lines.remove(0);
    }
    (GLOBAL_LOGGER.write()).lines = lines;

    // push to file
    if let Some(file) = GLOBAL_LOGGER.read().log_file.clone() {
        // append content to new line at end of file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .expect("could not open log file");
        writeln!(file, "{content}").expect("could not write to log file");
    }
}

pub fn clear() {
    (GLOBAL_LOGGER.write()).lines.clear();

    // clear file
    if let Some(file) = &(GLOBAL_LOGGER.write()).log_file {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .expect("could not open log file");
        file.set_len(0).expect("could not truncate log file");
    }
}
