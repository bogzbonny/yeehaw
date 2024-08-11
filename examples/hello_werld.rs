use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{Cui, DrawChs2D, Error, StandardPane, Style},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut el = StandardPane::default();
    let text = DrawChs2D::from_string("Hello, World!".to_string(), Style::default());
    el.content = text.0;
    Cui::new(Rc::new(RefCell::new(el)))?.run().await
}
