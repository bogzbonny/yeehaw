use {
    super::element::{ElementIDPath, ZIndexPath},
    crate::{ChPlus, ColorStore, Context, DrawAction, DrawChPos, DrawUpdate, Size},
    crossterm::style::{ContentStyle, StyledContent},
    std::time::Duration,
};

/// cached position on the screen
#[derive(Clone, Debug)]
pub struct CachedPos {
    /// does one of the layers have a time gradient
    /// if so it will recalculate the time gradient at
    /// each request to get the content at this position
    pub time_grad_count: usize,

    /// are the layers dirty at this position
    pub dirty: bool,

    /// the last information sent to the screen for this position
    pub last_draw_ch: Option<StyledContent<ChPlus>>,

    /// layers at this position
    pub layers: Vec<(ElementIDPath, ZIndexPath, DrawChPos)>,
}

impl Default for CachedPos {
    fn default() -> Self {
        Self {
            time_grad_count: 0,
            dirty: true,
            last_draw_ch: None,
            layers: Vec::new(),
        }
    }
}

impl CachedPos {
    /// gets the draw content at this position if an update is necessary
    pub fn get_update(
        &mut self, cs: &ColorStore, dsl: &Duration, draw_size: &Size,
    ) -> Option<StyledContent<ChPlus>> {
        // first sort the layers by z-index if dirty
        if self.dirty {
            self.layers.sort_by(|a, b| a.1.cmp(&b.1));
        }
        if self.dirty || self.time_grad_count > 0 {
            self.dirty = false;

            // start with the default
            let mut draw_ch = StyledContent::new(ContentStyle::default(), ChPlus::Char(' '));

            // iterate the layers from back to front creating the output ch
            for (_, _, dcp) in self.layers.iter() {
                draw_ch = dcp.get_content_style(cs, dsl, draw_size, &draw_ch);
            }
            if let Some(ref last_draw_ch) = self.last_draw_ch {
                if last_draw_ch == &draw_ch {
                    return None;
                }
            }
            self.last_draw_ch = Some(draw_ch.clone());
            return Some(draw_ch);
        }
        None
    }

    /// gets the draw content at this position independent of the dirty state
    pub fn must_get_draw_ch(
        &mut self, cs: &ColorStore, dsl: &Duration, draw_size: &Size,
    ) -> StyledContent<ChPlus> {
        if let Some(ch) = self.get_update(cs, dsl, draw_size) {
            ch
        } else if let Some(ref ch) = self.last_draw_ch {
            ch.clone()
        } else {
            StyledContent::new(ContentStyle::default(), ChPlus::Char(' '))
        }
    }

    // returns the number of time gradient count decreases
    pub fn remove(&mut self, ctx: &Context, ids: &ElementIDPath) -> usize {
        let mut out = 0;
        // NOTE there may be more than one element to remove with this id in this layer
        self.layers.retain(|(layer_ids, _, layer_dcp)| {
            if layer_ids == ids {
                if layer_dcp.ch.style.is_time_effected(ctx) {
                    if cfg!(debug_assertions) {
                        self.time_grad_count -= 1;
                    } else {
                        self.time_grad_count = self.time_grad_count.saturating_sub(1);
                    }
                    out += 1;
                }
                false
            } else {
                true
            }
        });
        self.dirty = true;
        out
    }

    // returns the number of time gradient count increases
    pub fn add(
        &mut self, ctx: &Context, ids: &ElementIDPath, zs: &ZIndexPath, dcp: DrawChPos,
    ) -> usize {
        let out = if dcp.ch.style.is_time_effected(ctx) {
            self.time_grad_count += 1;
            1
        } else {
            0
        };
        self.layers.push((ids.clone(), zs.clone(), dcp));
        self.dirty = true;
        out
    }
}

#[derive(Default, Clone)]
pub struct DrawingCache {
    pub cached_upd: Vec<(ElementIDPath, ZIndexPath, Vec<DrawChPos>)>,

    /// The total number of cached time gradients
    /// This is used to keep track of the number of time gradients,
    /// if there are no time gradients then we don't need to send
    /// any updates to the screen if there have been no changes.
    pub time_grad_count: usize,

    /// 2d array of the screen
    //            rows(y)
    //             │  col(x)
    //             │   │
    pub cache_2d: Vec<Vec<CachedPos>>,
}

impl DrawingCache {
    pub fn clear_screen(&mut self) {
        for row in self.cache_2d.iter_mut() {
            for cell in row.iter_mut() {
                cell.last_draw_ch = None;
                cell.dirty = true;
            }
        }
    }

    pub fn update(&mut self, ctx: &Context, mut updates: Vec<DrawUpdate>) {
        if updates.is_empty() {
            return;
        }

        let mut upd_2d_rm = Vec::<(ElementIDPath, Vec<DrawChPos>)>::new(); // removals for the 2d array
        let mut upd_2d_add = Vec::<(ElementIDPath, ZIndexPath, Vec<DrawChPos>)>::new(); // additions for the 2d array

        for update in updates.drain(..) {
            match update.action {
                DrawAction::ClearAll => {
                    //debug!("clearing all at sub_id: {:?}", update.sub_id);

                    // take all entries with the prefix and remove them
                    let mut cached = Vec::with_capacity(self.cached_upd.len());
                    for (ids, zs, dcps) in self.cached_upd.drain(..) {
                        if ids.starts_with(&update.sub_id) {
                            upd_2d_rm.push((ids, dcps));
                        } else {
                            cached.push((ids, zs, dcps));
                        }
                    }
                    self.cached_upd = cached;
                }
                DrawAction::Remove => {
                    //debug!("removing at sub_id: {:?}", update.sub_id);

                    // take all entries with the prefix and remove them
                    let mut cached = Vec::with_capacity(self.cached_upd.len());
                    for (ids, zs, dcps) in self.cached_upd.drain(..) {
                        if ids == update.sub_id {
                            upd_2d_rm.push((ids, dcps));
                        } else {
                            cached.push((ids, zs, dcps));
                        }
                    }
                    self.cached_upd = cached;
                }
                DrawAction::Update(upd_dcps) => {
                    //debug!("updating at sub_id: {:?}", update.sub_id);

                    // take all entries with the prefix and remove them, then add in the update
                    let mut cached = Vec::with_capacity(self.cached_upd.len());
                    for (ids, zs, dcps) in self.cached_upd.drain(..) {
                        if ids == update.sub_id {
                            upd_2d_rm.push((ids.clone(), dcps));
                        } else {
                            cached.push((ids, zs, dcps));
                        }
                    }
                    cached.push((
                        update.sub_id.clone(),
                        update.z_indicies.clone(),
                        upd_dcps.clone(),
                    ));
                    upd_2d_add.push((update.sub_id, update.z_indicies, upd_dcps));
                    self.cached_upd = cached;
                }
                DrawAction::Extend(upd_dcps) => {
                    //debug!("extending at sub_id: {:?}", update.sub_id);
                    if let Some((_, z, draw)) = self
                        .cached_upd
                        .iter_mut()
                        .find(|(ids, _, _)| ids == &update.sub_id)
                    {
                        draw.extend(upd_dcps.clone());
                        *z = update.z_indicies.clone();
                    } else {
                        self.cached_upd.push((
                            update.sub_id.clone(),
                            update.z_indicies.clone(),
                            upd_dcps.clone(),
                        ));
                    }
                    upd_2d_add.push((update.sub_id, update.z_indicies, upd_dcps));
                }
            }
        }

        // now that the the updated are added to the cached_upd, we need to update
        // the cache_2d based on the removals and additions
        for (ids, mut dcps) in upd_2d_rm.drain(..) {
            for dcp in dcps.drain(..) {
                let (x, y) = (dcp.x, dcp.y);
                let Some(row) = self.cache_2d.get_mut(y as usize) else {
                    // NOTE this can happen when an element passes multiple updates
                    // notibly the tabs top element triggers this at startup
                    continue;
                };
                let Some(cell) = row.get_mut(x as usize) else {
                    // same as above note
                    continue;
                };
                let time_grad_count_decr = cell.remove(ctx, &ids);
                if time_grad_count_decr > 0 {
                    if cfg!(debug_assertions) {
                        self.time_grad_count -= time_grad_count_decr;
                    } else {
                        self.time_grad_count =
                            self.time_grad_count.saturating_sub(time_grad_count_decr);
                    }
                }
            }
        }

        for (ids, zs, mut dcps) in upd_2d_add.drain(..) {
            for dcp in dcps.drain(..) {
                let (x, y) = (dcp.x, dcp.y);
                if self.cache_2d.len() <= y as usize {
                    self.cache_2d.resize(y as usize + 1, Vec::new());
                }
                let row = self.cache_2d.get_mut(y as usize).expect("impossible");
                if row.len() <= x as usize {
                    row.resize(x as usize + 1, CachedPos::default());
                }
                let cell = row.get_mut(x as usize).expect("impossible");
                let time_grad_count_incr = cell.add(ctx, &ids, &zs, dcp);
                if time_grad_count_incr > 0 {
                    self.time_grad_count += time_grad_count_incr;
                }
            }
        }
    }

    pub fn update_and_get(
        &mut self, ctx: &Context, draw_size: &Size, updates: Vec<DrawUpdate>,
    ) -> Vec<(usize, usize, StyledContent<ChPlus>)> {
        let upd_len = updates.len();
        self.update(ctx, updates);

        // no updates, no time gradients, no need to do anything
        if upd_len == 0 && self.time_grad_count == 0 {
            return Vec::new();
        }

        let cs = &ctx.color_store;
        let dsl = &ctx.dur_since_launch;

        // NOTE I tried refactoring this with rayon but it was MUCH slower
        let mut out = Vec::new();
        for (y, row) in self.cache_2d.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                if let Some(upd) = cell.get_update(cs, dsl, draw_size) {
                    out.push((x, y, upd));
                }
            }
        }
        out
    }
}
