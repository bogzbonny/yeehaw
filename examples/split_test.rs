use {
    std::{cell::RefCell, rc::Rc},
    yeehaw::{
        Context, Cui, DebugSizePane, DynLocation, DynVal, Element, Error, ParentPane, SortingHat,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //yeehaw::debug::set_log_file("./widget_test.log".to_string());
    //yeehaw::debug::clear();

    //std::env::set_var("RUST_BACKTRACE", "1");

    let hat = SortingHat::default();
    let _ctx = Context::new_context_for_screen();
    let pp = ParentPane::new(&hat, "pp");
    let left = DebugSizePane::new(&hat);
    let right = DebugSizePane::new(&hat);
    let endval = DynVal::new_flex(0.35);
    let left_loc = DynLocation::new(
        DynVal::new_flex(0.0),
        endval.clone(),
        DynVal::new_flex(0.0),
        DynVal::new_flex(1.0),
    );
    let right_loc = DynLocation::new(
        endval,
        DynVal::new_flex(1.0),
        DynVal::new_flex(0.0),
        DynVal::new_flex(1.0),
    );
    left.get_dyn_location_set().borrow_mut().l = left_loc;
    right.get_dyn_location_set().borrow_mut().l = right_loc;
    pp.add_element(Rc::new(RefCell::new(left)));
    pp.add_element(Rc::new(RefCell::new(right)));

    Cui::new(Rc::new(RefCell::new(pp)))?.run().await
}
