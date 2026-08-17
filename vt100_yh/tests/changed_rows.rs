use vt100_yh::Parser;

#[test]
fn new_screen_reports_all_changed() {
    let mut parser = Parser::new(4, 10, 0);
    assert_eq!(parser.screen_mut().take_changed_rows(), None);
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![]));
}

#[test]
fn text_reports_only_changed_rows() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"abc");
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![0]));

    parser.process(b"\r\nxyz");
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![1]));
}

#[test]
fn cursor_movement_reports_no_rows() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"\x1b[2;5H");
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![]));
    // the coarse dirty flag is still set
    assert!(parser.screen().is_dirty());
}

#[test]
fn sgr_reports_no_rows() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"\x1b[31m");
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![]));
}

#[test]
fn full_erase_reports_all() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"abc\r\n\x1b[2J");
    assert_eq!(parser.screen_mut().take_changed_rows(), None);
}

#[test]
fn el_reports_only_current_row() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"aaa\r\nbbb\r\nccc");
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![0, 1, 2]));
    parser.process(b"\x1b[2;1H\x1b[K");
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![1]));
}

#[test]
fn scroll_reports_all() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"a\r\nb\r\nc\r\nd\r\ne");
    // the second to last \r\n scrolled the screen
    assert_eq!(parser.screen_mut().take_changed_rows(), None);
}

#[test]
fn resize_reports_all() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"abc");
    parser.screen_mut().set_size(6, 12);
    assert_eq!(parser.screen_mut().take_changed_rows(), None);
}

#[test]
fn alternate_screen_reports_all() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"\x1b[?1049h");
    assert_eq!(parser.screen_mut().take_changed_rows(), None);
    parser.process(b"\x1b[?1049l");
    assert_eq!(parser.screen_mut().take_changed_rows(), None);
}

#[test]
fn take_clears_tracking() {
    let mut parser = Parser::new(4, 10, 0);
    let _ = parser.screen_mut().take_changed_rows();
    parser.process(b"abc");
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![0]));
    assert_eq!(parser.screen_mut().take_changed_rows(), Some(vec![]));
}
