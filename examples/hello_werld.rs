use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{Cui, DrawChs2D, Error, SortingHat, StandardPane, Style},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let hat = SortingHat::default();
    let mut el = StandardPane::new_empty(&hat, StandardPane::KIND);
    let text = DrawChs2D::from_string("Hello, World!".to_string(), Style::default());
    el.content = text.0;
    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
