use crossterm::style::Color;
use yeehaw::{
    elements::{
        containers::{Table, TableStyle, ColumnWidth},
        Button, Label,
    },
    Context, DynVal, TuiBuilder,
};

fn main() -> std::io::Result<()> {
    let mut tui = TuiBuilder::default()
        .description("Table Container Demo")
        .build()?;

    let ctx = &tui.get_context();

    // Create a table with various style options
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

    let mut table = Table::new(ctx, style);
    
    // Create header using strings (auto-converted to Labels)
    let header = vec!["ID", "Name", "Status", "Actions"];

    // Create content rows with mixed element types
    let data = vec![
        (
            "1",
            "Alice Thompson",
            Label::new(ctx, "✓ Active").with_fg(Color::Green),
            Button::new(ctx, "Edit"),
        ),
        (
            "2",
            "Bob Wilson",
            Label::new(ctx, "⚠ Pending").with_fg(Color::Yellow),
            Button::new(ctx, "Edit"),
        ),
        (
            "3",
            "Charlie Brown",
            Label::new(ctx, "✗ Inactive").with_fg(Color::Red),
            Button::new(ctx, "Edit"),
        ),
    ];

    // Convert rows to boxed elements
    let mut rows = vec![];
    rows.push(header.into_iter().map(|s| Box::new(Label::new(ctx, s)) as Box<dyn Element>).collect());

    for (id, name, status, action) in data {
        rows.push(vec![
            Box::new(Label::new(ctx, id)),
            Box::new(Label::new(ctx, name)),
            Box::new(status),
            Box::new(action),
        ]);
    }

    table.set_data_elements(rows);

    // Set column widths with different strategies
    let widths = vec![
        ColumnWidth::Fixed(DynVal::new_fixed(4)),      // ID: fixed 4 chars
        ColumnWidth::Percent(DynVal::new_flex(0.4)),   // Name: 40% of remaining
        ColumnWidth::Auto,                             // Status: auto-fit content
        ColumnWidth::Fixed(DynVal::new_fixed(8)),      // Actions: fixed 8 chars
    ];

    table.set_column_widths(widths);

    // Position the table in the center of the screen
    table.set_dyn_location_set(DynLocationSet {
        l: DynLocation::new(
            DynVal::new_flex(0.1),  // 10% from left
            DynVal::new_flex(0.1),  // 10% from top
            DynVal::new_flex(0.8),  // 80% width
            DynVal::new_flex(0.8),  // 80% height
        ),
        z: 0,
    });

    // Add table to TUI
    tui.get_main_pane_mut().add_element(Box::new(table));

    tui.run()
}
