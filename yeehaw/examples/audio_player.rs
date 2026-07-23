use std::path::PathBuf;

use yeehaw::*;
use yeehaw::elements::widgets::SingleLineTextBox;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element");

    // TextBox for entering audio file path
    let mut textbox = SingleLineTextBox::new(&ctx);
    textbox.tb = textbox.tb.editable(&ctx);
    textbox.tb.pane.set_at(DynVal::new_fixed(1), DynVal::new_fixed(1));
    textbox.tb.pane.set_dyn_width(DynVal::new_fixed(50));

    // AudioPlayer (starts with empty sources)
    let audio_player = AudioPlayer::new(&ctx, vec![]);
    audio_player.pane.set_at(DynVal::new_fixed(0), DynVal::new_fixed(3));
    audio_player.pane.set_dyn_width(DynVal::FULL);
    audio_player.pane.set_dyn_height(DynVal::new_fixed(2));

    // Clone for closure
    let textbox_ = textbox.clone();
    let audio_player_ = audio_player.clone();

    // Button to load audio file
    let load_btn = Button::new(&ctx, "Load")
        .with_fn(Box::new(move |_, _| {
            let path_str = textbox_.tb.get_text();
            if !path_str.is_empty() {
                let path = PathBuf::from(path_str);
                audio_player_.add_source(path);
            }
            EventResponses::default()
        }))
        .at(DynVal::new_fixed(52), DynVal::new_fixed(1));

    main_el.add_element(Box::new(textbox));
    main_el.add_element(Box::new(audio_player));
    main_el.add_element(Box::new(load_btn));
    tui.run(Box::new(main_el)).await
}
