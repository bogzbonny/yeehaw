use {crate::*, rayon::prelude::*};

/// displays a shadow around the element
#[derive(Clone)]
pub struct Shadowed {
    pub inner: Box<dyn Element>,
    pub sh_sty: Rc<RefCell<ShadowSty>>,
    pub overflow: Rc<RefCell<bool>>,
}

#[derive(Clone)]
pub struct ShadowSty {
    pub bottom_left: DrawCh,
    pub bottom_middle: DrawCh,
    pub bottom_right: DrawCh,
    pub right: DrawCh,
    pub top_right: DrawCh,
}

impl ShadowSty {
    pub fn new_thin(shadow_color: Color) -> Self {
        let sty = Style::default()
            .with_bg(Color::TRANSPARENT)
            .with_fg(shadow_color)
            .with_fg_transp_src(FgTranspSrc::LowerBg);
        Self {
            bottom_left: DrawCh::new('▝', sty.clone()),
            bottom_middle: DrawCh::new('▀', sty.clone()),
            bottom_right: DrawCh::new('▘', sty.clone()),
            right: DrawCh::new('▌', sty.clone()),
            top_right: DrawCh::new('▖', sty.clone()),
        }
    }

    pub fn new_thick(shadow_color: Color) -> Self {
        let sty = Style::default()
            .with_bg(shadow_color.clone())
            .with_fg(shadow_color);
        let full = DrawCh::new(ChPlus::Transparent, sty);
        let empty = DrawCh::transparent();
        Self {
            bottom_left: empty.clone(),
            bottom_middle: full.clone(),
            bottom_right: full.clone(),
            right: full,
            top_right: empty,
        }
    }
}

impl Shadowed {
    pub const KIND: &'static str = "shadowed";

    pub fn thin(ctx: &Context, inner: Box<dyn Element>) -> Shadowed {
        let shadow_color = Color::new_with_alpha(100, 100, 100, 100);
        let out = Shadowed {
            inner,
            sh_sty: Rc::new(RefCell::new(ShadowSty::new_thin(shadow_color))),
            overflow: Rc::new(RefCell::new(true)),
        };
        out.set_shadow_content(ctx);
        out
    }

    pub fn thin_with_color(
        ctx: &Context, inner: Box<dyn Element>, shadow_color: Color,
    ) -> Shadowed {
        let out = Shadowed {
            inner,
            sh_sty: Rc::new(RefCell::new(ShadowSty::new_thin(shadow_color))),
            overflow: Rc::new(RefCell::new(true)),
        };
        out.set_shadow_content(ctx);
        out
    }

    pub fn thick(ctx: &Context, inner: Box<dyn Element>) -> Shadowed {
        let shadow_color = Color::new_with_alpha(100, 100, 100, 100);
        let out = Shadowed {
            inner,
            sh_sty: Rc::new(RefCell::new(ShadowSty::new_thick(shadow_color))),
            overflow: Rc::new(RefCell::new(true)),
        };
        out.set_shadow_content(ctx);
        out
    }

    pub fn thick_with_color(
        ctx: &Context, inner: Box<dyn Element>, shadow_color: Color,
    ) -> Shadowed {
        let out = Shadowed {
            inner,
            sh_sty: Rc::new(RefCell::new(ShadowSty::new_thick(shadow_color))),
            overflow: Rc::new(RefCell::new(true)),
        };
        out.set_shadow_content(ctx);
        out
    }

    /// TODO could cache this
    pub fn set_shadow_content(&self, ctx: &Context) -> Vec<DrawChPos> {
        let size = ctx.size;
        let sh_sty = self.sh_sty.borrow();

        let mut out = vec![];

        out.push(DrawChPos::new(sh_sty.bottom_left.clone(), 0, size.height));
        out.push(DrawChPos::new(sh_sty.top_right.clone(), size.width, 0));
        out.push(DrawChPos::new(
            sh_sty.bottom_right.clone(),
            size.width,
            size.height,
        ));
        for x in 1..size.width {
            out.push(DrawChPos::new(sh_sty.bottom_middle.clone(), x, size.height));
        }
        for y in 1..size.height {
            out.push(DrawChPos::new(sh_sty.right.clone(), size.width, y));
        }

        out
    }
}

#[yeehaw_derive::impl_element_from(inner)]
impl Element for Shadowed {
    fn drawing(&self, ctx: &Context, force_update: bool) -> Vec<DrawUpdate> {
        let mut upds = self.inner.drawing(ctx, force_update);

        // because the shadow allows overflow, we must deal with any overflows here
        for upd in &mut upds {
            match upd.action {
                DrawAction::Update(_, ref mut dcps) | DrawAction::Extend(_, ref mut dcps) => {
                    if *self.inner.get_ref_cell_overflow().borrow() {
                        let width = ctx.size.width;
                        let height = ctx.size.height;
                        dcps.par_iter_mut().for_each(move |dcp| {
                            if dcp.x >= width || dcp.y >= height {
                                // it'd be better to delete, but we can't delete from a parallel iterator
                                // also using a filter here its slower that this
                                dcp.ch = DrawCh::transparent();
                                (dcp.x, dcp.y) = (0, 0);
                            }
                        })
                    }
                }
                DrawAction::Remove | DrawAction::ClearAll => {}
            }
        }

        // add the shadow content
        for upd in &mut upds {
            if upd.sub_id.is_empty() {
                if let DrawAction::Update(_, ref mut dcps) = upd.action {
                    dcps.extend(self.set_shadow_content(ctx));
                    break;
                }
            }
        }
        upds
    }

    fn get_ref_cell_overflow(&self) -> Rc<RefCell<bool>> {
        self.overflow.clone()
    }
}
