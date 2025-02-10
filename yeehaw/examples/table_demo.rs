use crossterm::style::Color;
use yeehaw::{
    elements::containers::{Table, TableStyle},
    Context, DynLocation, DynVal, TuiBuilder,
};

fn main() -> std::io::Result<()> {
    let mut tui = TuiBuilder::default()
        .description("Table Container Demo")
        .build()?;

    let ctx = Context::new_context();

    let style = TableStyle {
        header_line: true,
        vertical_lines: true,
        horizontal_lines: true,
        header_fg_color: Some(Color::White),
        header_bg_color: Some(Color::Blue),
        content_fg_color: None,
        content_bg_color: None,
        row_alternation: Some((Color::DarkGrey, 2)),
        column_alternation: None,
    };

    // Create table data mixing strings and custom elements
    let mut table = Table::new(&ctx, style);
    
    // Header row
    let header = vec![
        "ID", 
        "Name",
        "Role",
        "Status",
    ];

    // Content rows
    let data = vec![
        vec!["1", "Alice", "Developer", "Active"],
        vec!["2", "Bob", "Designer", "Inactive"],
        vec!["3", "Charlie", "Manager", "Active"],
        vec!["4", "David", "Developer", "Active"],
    ];

    // Column widths
    let widths = vec![
        DynVal::new_fixed(4),    // ID
        DynVal::new_flex(0.4),   // Name (40% of remaining)
        DynVal::new_flex(0.4),   // Role (40% of remaining)
        DynVal::new_flex(0.2),   // Status (20% of remaining)
    ];

    // Create rows combining header and data
    let mut rows = Vec::new();
    rows.push(header.into_iter().map(|s| s.into()).collect());

    for row in data {
        rows.push(row.into_iter().map(|s| s.into()).collect());
    }

    table.set_data(rows);
    table.set_column_widths(widths);

    // Add table to TUI
    tui.get_main_pane_mut().push(Box::new(table));

    tui.run()
}
