use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut tui, ctx) = Tui::new()?;
    let parent = ParentPane::new(&ctx, "main");
    let photobooth = Photobooth::new(&ctx, None, None, None, None)?
        .at(DynVal::FULL, DynVal::FULL);
    parent.add_element(Box::new(photobooth));
    tui.run(Box::new(parent)).await
}
