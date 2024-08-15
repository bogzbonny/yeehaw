use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{Cui, DrawChs2D, Error, SortingHat, StandardPane, Style},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let hat = SortingHat::default();
    let text = DrawChs2D::from_string("Hello, Werld!".to_string(), Style::default());
    let el = StandardPane::new(&hat, StandardPane::KIND).with_content(text);
    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
