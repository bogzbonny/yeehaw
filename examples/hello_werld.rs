use yeehaw::{Cui, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let el = Element::new();
    Cui::new(el)?.run().await?;
    Ok(())
}
