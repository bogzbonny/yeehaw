use {std::path::PathBuf, yeehaw::elements::widgets::TextBox, yeehaw::*};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element");

    // TextBox for entering audio file path
    let textbox = TextBox::new(&ctx, "")
        .with_text_when_empty("enter text here")
        .with_dyn_width(50.into())
        .with_dyn_height(2.into())
        .editable(&ctx)
        .at(1, 1);

    // AudioPlayer (starts with empty sources)
    let audio_player = AudioPlayer::new(&ctx, vec![]);
    audio_player
        .pane
        .set_at(DynVal::new_fixed(1), DynVal::new_fixed(4));
    audio_player.pane.set_dyn_width(DynVal::new_fixed(50));
    audio_player.pane.set_dyn_height(DynVal::new_fixed(2));

    // Button to load audio file
    let textbox_ = textbox.clone();
    let audio_player_ = audio_player.clone();
    let load_btn = Button::new(&ctx, "Load")
        .with_fn(Box::new(move |_, _| {
            let path_str = textbox_.get_text();
            if !path_str.is_empty() {
                let path = PathBuf::from(path_str);
                audio_player_.set_sources(vec![path]);
            }
            EventResponses::default()
        }))
        .at(DynVal::new_fixed(52), DynVal::new_fixed(1));

    main_el.add_element(Box::new(textbox));
    main_el.add_element(Box::new(audio_player));
    main_el.add_element(Box::new(load_btn));
    tui.run(Box::new(main_el)).await
}
