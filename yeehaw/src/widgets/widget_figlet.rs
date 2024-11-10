use {
    super::{Selectability, WBStyles, Widget, WidgetBase, Widgets},
    crate::{
        Context, DrawChPos, DrawChs2D, DynLocationSet, DynVal, Element, ElementID, Event,
        EventResponses, Parent, Priority, ReceivableEventChanges, SelfReceivableEvents, Style,
    },
    figlet_rs::FIGfont,
};

/// TODO click function

#[derive(Clone)]
pub struct FigletText {
    pub base: WidgetBase,
}

impl FigletText {
    const KIND: &'static str = "widget_megatext";

    pub fn new(ctx: &Context, text: &str, font: FIGfont) -> Self {
        let Some(fig_text) = font.convert(text) else {
            return FigletText {
                base: WidgetBase::new(
                    ctx,
                    Self::KIND,
                    DynVal::new_fixed(0),
                    DynVal::new_fixed(0),
                    WBStyles::default(),
                    vec![],
                ),
            };
        };

        let text = format!("{}", fig_text);
        let text = text.trim_end_matches('\n'); // remove the last newline
        let content = DrawChs2D::from_string(text.to_string(), Style::default());
        let size = content.size();

        let wb = WidgetBase::new(
            ctx,
            Self::KIND,
            DynVal::new_fixed(size.width as i32),
            DynVal::new_fixed(size.height as i32),
            WBStyles::default(),
            vec![],
        );
        wb.set_content(content);
        _ = wb.set_selectability(ctx, Selectability::Unselectable);
        FigletText { base: wb }
    }

    // ----------------------------------------------
    /// decorators

    pub fn with_styles(self, styles: WBStyles) -> Self {
        self.base
            .set_content_style(styles.unselectable_style.clone());
        self.base.set_styles(styles);
        self
    }

    pub fn at(self, loc_x: DynVal, loc_y: DynVal) -> Self {
        self.base.at(loc_x, loc_y);
        self
    }

    pub fn to_widgets(self) -> Widgets {
        Widgets(vec![Box::new(self)])
    }
}

impl Widget for FigletText {}

#[yeehaw_derive::impl_element_from(base)]
impl Element for FigletText {}
