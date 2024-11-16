use {
    crate::{Keyboard as KB, *},
    crossterm::event::MouseEventKind,
    std::{cell::RefCell, rc::Rc},
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Selectability {
    /// currently selected
    Selected,
    /// not selected but able to be selected
    Ready,
    /// unselectable
    Unselectable,
}

impl std::fmt::Display for Selectability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selectability::Selected => write!(f, "Selected"),
            Selectability::Ready => write!(f, "Ready"),
            Selectability::Unselectable => write!(f, "Unselectable"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct SelectabilityResp {
    sel: Selectability,
    id: ElementID,
}

pub const ATTR_SELECTABILITY: &str = "selectability";

/// SelectablePane is an extension of the ParentPane which allows for the selection.
#[derive(Clone)]
pub struct SelectablePane {
    pub pane: ParentPane,
    pub styles: Rc<RefCell<SelStyles>>,
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl SelectablePane {
    pub const RESP_SELECTABILITY_WAS_SET: &'static str = "selectability_was_set";

    pub fn new(ctx: &Context, kind: &'static str) -> SelectablePane {
        let out = SelectablePane {
            pane: ParentPane::new(ctx, kind),
            styles: Rc::new(RefCell::new(SelStyles::default())),
        };
        out.enable();
        out
    }

    #[allow(clippy::type_complexity)]
    pub fn set_pre_hook_for_set_selectability(&self, hook: ElementHookFn) {
        let pre_hook_name = format!(
            "{}{}",
            element::PRE_ATTR_CHANGE_HOOK_NAME_PREFIX,
            ATTR_SELECTABILITY
        );
        self.pane.pane.set_hook(&pre_hook_name, self.id(), hook);
    }

    pub fn set_post_hook_for_set_selectability(&self, hook: ElementHookFn) {
        let pre_hook_name = format!(
            "{}{}",
            element::POST_ATTR_CHANGE_HOOK_NAME_PREFIX,
            ATTR_SELECTABILITY
        );
        self.pane.pane.set_hook(&pre_hook_name, self.id(), hook);
    }

    pub fn set_styles(&self, styles: SelStyles) {
        *self.styles.borrow_mut() = styles;
        self.set_style(self.get_current_style());
    }

    pub fn with_styles(self, styles: SelStyles) -> Self {
        self.set_styles(styles);
        self
    }

    pub fn disable(&self) -> EventResponses {
        self.set_selectability(Selectability::Unselectable)
    }

    pub fn enable(&self) -> EventResponses {
        self.set_selectability(Selectability::Ready)
    }

    pub fn deselect(&self) -> EventResponses {
        self.set_selectability(Selectability::Ready)
    }

    pub fn select(&self) -> EventResponses {
        self.set_selectability(Selectability::Selected)
    }

    pub fn get_current_style(&self) -> Style {
        match self.get_selectability() {
            Selectability::Selected => self.styles.borrow().selected_style.clone(),
            Selectability::Ready => self.styles.borrow().ready_style.clone(),
            Selectability::Unselectable => self.styles.borrow().unselectable_style.clone(),
        }
    }

    pub fn get_selectability(&self) -> Selectability {
        let Some(bz) = self.get_attribute(ATTR_SELECTABILITY) else {
            return Selectability::Ready;
        };
        match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                Selectability::Ready
            }
        }
    }

    fn set_attr_selectability(&self, s: Selectability) {
        let bz = match serde_json::to_vec(&s) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return;
            }
        };
        self.set_attribute(ATTR_SELECTABILITY, bz)
    }

    pub fn set_selectability(&self, s: Selectability) -> EventResponses {
        let mut prev_sel: Option<Selectability> = None;
        if let Some(bz) = self.get_attribute(ATTR_SELECTABILITY) {
            if let Ok(v) = serde_json::from_slice(&bz) {
                prev_sel = Some(v);
            }
        };
        if let Some(prev_sel) = prev_sel {
            if s == prev_sel {
                return EventResponses::default();
            }
        }

        let mut resps = EventResponses::default();
        match s {
            Selectability::Selected => {
                // NOTE needs to happen before the next line or else receivable will return the
                // wrong value
                self.set_attr_selectability(s);
                let rec = ReceivableEventChanges::default().with_add_evs(self.receivable().0);
                resps.push(EventResponse::ReceivableEventChanges(rec));
                resps.push(EventResponse::BringToFront);
                resps.push(EventResponse::Metadata(
                    Self::RESP_SELECTABILITY_WAS_SET.to_string(),
                    serde_json::to_vec(&SelectabilityResp {
                        sel: s,
                        id: self.id().clone(),
                    })
                    .unwrap(),
                ));
            }
            Selectability::Ready | Selectability::Unselectable => {
                if let Some(Selectability::Selected) = prev_sel {
                    let rec = ReceivableEventChanges::default().with_remove_evs(
                        self.receivable().iter().map(|(ev, _)| ev.clone()).collect(),
                    );
                    resps.push(EventResponse::ReceivableEventChanges(rec));

                    resps.push(EventResponse::Metadata(
                        Self::RESP_SELECTABILITY_WAS_SET.to_string(),
                        serde_json::to_vec(&SelectabilityResp {
                            sel: s,
                            id: self.id().clone(),
                        })
                        .unwrap(),
                    ));
                }
                // NOTE this needs to after the prev line or else receivable will return the
                // wrong value
                self.set_attr_selectability(s);
            }
        }

        resps
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for SelectablePane {
    /// default implementation of Receivable, only receive when widget is active
    fn receivable(&self) -> SelfReceivableEvents {
        let attr_sel = self.get_selectability();
        if let Selectability::Selected = attr_sel {
            self.pane.receivable()
        } else {
            SelfReceivableEvents::default()
        }
    }

    // XXX delete
    ///// default implementation of drawing
    //fn drawing(&self, ctx: &Context) -> Vec<DrawChPos> {
    //    let mut chs = self.pane.drawing(ctx);

    //    let sty = self.get_current_style();
    //    let h = self.pane.pane.get_height(ctx);
    //    let w = self.pane.pane.get_width(ctx);
    //    let view_offset_y = *self.pane.pane.content_view_offset_y.borrow();
    //    let view_offset_x = *self.pane.pane.content_view_offset_x.borrow();
    //    let content_height = self.pane.pane.content.borrow().height();
    //    let content_width = self.pane.pane.content.borrow().width();

    //    for y in view_offset_y..view_offset_y + h {
    //        for x in view_offset_x..view_offset_x + w {
    //            let ch = if y < content_height && x < content_width {
    //                self.pane.pane.content.borrow().0[y][x].clone()
    //            } else {
    //                DrawCh::new(' ', sty.clone())
    //            };
    //            chs.push(DrawChPos::new(
    //                ch,
    //                (x - view_offset_x) as u16,
    //                (y - view_offset_y) as u16,
    //            ));
    //        }
    //    }
    //    chs
    //}

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = match ev {
            Event::Mouse(me) => {
                if matches!(me.kind, MouseEventKind::Up(_)) {
                    (false, self.select()) // XXX
                } else {
                    (false, EventResponses::default())
                }
            }
            Event::ExternalMouse(ref me) => {
                if matches!(
                    me.kind,
                    MouseEventKind::Down(_) | MouseEventKind::Drag(_) | MouseEventKind::Up(_)
                ) {
                    (false, self.deselect())
                } else {
                    (false, EventResponses::default())
                }
            }
            Event::Custom(ref ev_name, ref bz) => {
                if ev_name == ParentPaneOfSelectable::EV_SET_SELECTABILITY {
                    match serde_json::from_slice(bz) {
                        Ok(v) => (true, self.set_selectability(v)),
                        Err(_e) => {
                            // TODO log error
                            (true, EventResponses::default())
                        }
                    }
                } else {
                    (false, EventResponses::default())
                }
            }
            _ => (false, EventResponses::default()),
        };
        if captured {
            return (true, resps);
        }
        let (captured, resps_) = self.pane.receive_event(ctx, ev);
        resps.extend(resps_);
        (captured, resps)
    }
}

// ---------------------------------------
#[derive(Clone, Default)]
pub struct SelStyles {
    pub selected_style: Style,
    pub ready_style: Style,
    pub unselectable_style: Style,
}

impl SelStyles {
    pub fn new(selected_style: Style, ready_style: Style, unselectable_style: Style) -> SelStyles {
        SelStyles {
            selected_style,
            ready_style,
            unselectable_style,
        }
    }

    pub fn transparent() -> SelStyles {
        SelStyles {
            selected_style: Style::transparent(),
            ready_style: Style::transparent(),
            unselectable_style: Style::transparent(),
        }
    }
}

// ---------------------------------------

/// parent pane but with selectable pane logic
/// use tabs to
#[derive(Clone)]
pub struct ParentPaneOfSelectable {
    pub pane: ParentPane,
    pub selected: Rc<RefCell<Option<ElementID>>>,
    pub selectables: Rc<RefCell<Vec<ElementID>>>,
}

impl ParentPaneOfSelectable {
    pub const KIND: &'static str = "parent_pane_of_selectable";
    pub const EV_SET_SELECTABILITY: &'static str = "set_selectability";

    pub fn default_receivable_events() -> SelfReceivableEvents {
        SelfReceivableEvents(vec![
            (KB::KEY_ESC.into(), Priority::Focused),
            (KB::KEY_TAB.into(), Priority::Focused),
            (KB::KEY_BACKTAB.into(), Priority::Focused),
        ])
    }

    pub fn new(ctx: &Context) -> ParentPaneOfSelectable {
        let pane = ParentPane::new(ctx, Self::KIND)
            .with_self_receivable_events(Self::default_receivable_events());
        ParentPaneOfSelectable {
            pane,
            selected: Rc::new(RefCell::new(None)),
            selectables: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn add_element(&self, el: Box<dyn Element>) {
        // check if it is selectable
        if el.get_attribute(ATTR_SELECTABILITY).is_some() {
            self.selectables.borrow_mut().push(el.id());
        }
        self.pane.add_element(el);
    }

    fn get_selectability_for_el(&self, el_id: &ElementID) -> Option<Selectability> {
        let bz = self.pane.get_element_attribute(el_id, ATTR_SELECTABILITY)?;
        let sel = match serde_json::from_slice(&bz) {
            Ok(v) => v,
            Err(_e) => {
                // TODO log error
                return None;
            }
        };
        Some(sel)
    }

    fn set_selectability_for_el(
        &self, ctx: &Context, el_id: &ElementID, s: Selectability,
    ) -> EventResponses {
        let ev_bz = serde_json::to_vec(&s).unwrap();
        let ev = Event::Custom(Self::EV_SET_SELECTABILITY.to_string(), ev_bz);
        let mut resps = self.pane.send_event_to_el(ctx, el_id, ev);
        self.partially_process_sel_resps(ctx, &mut resps);
        resps
    }

    pub fn remove_element(&self, el_id: &ElementID) {
        self.pane.remove_element(el_id);
        self.selectables.borrow_mut().retain(|id| id != el_id);
    }

    pub fn clear_elements(&self) {
        self.pane.clear_elements();
        self.selectables.borrow_mut().clear();
    }

    pub fn unselect_selected(&self, ctx: &Context) -> EventResponses {
        let sel_el_id = self.selected.borrow().clone();
        let resps = if let Some(ref sel_el_id) = sel_el_id {
            self.set_selectability_for_el(ctx, sel_el_id, Selectability::Ready)
        } else {
            EventResponses::default()
        };
        *self.selected.borrow_mut() = None;
        resps
    }

    /// processes the custom response for setting selectability
    pub fn partially_process_sel_resps(&self, ctx: &Context, resps: &mut EventResponses) {
        let mut extend_resps = vec![];
        for resp in resps.0.iter_mut() {
            let mut modified_resp = None;

            if let EventResponse::Metadata(k, v_bz) = resp {
                if k == SelectablePane::RESP_SELECTABILITY_WAS_SET {
                    let s_resp: SelectabilityResp = match serde_json::from_slice(v_bz) {
                        Ok(v) => v,
                        Err(_e) => {
                            // TODO log error
                            continue;
                        }
                    };
                    match s_resp.sel {
                        Selectability::Selected => {
                            let old_sel_el_id = self.selected.borrow().clone();
                            if let Some(old_sel_el_id) = old_sel_el_id {
                                if old_sel_el_id != s_resp.id {
                                    *self.selected.borrow_mut() = Some(s_resp.id.clone());

                                    // deselect the old selected element
                                    let resps_ = self.set_selectability_for_el(
                                        ctx,
                                        &old_sel_el_id,
                                        Selectability::Ready,
                                    );
                                    extend_resps.extend(resps_.0);
                                }
                            }
                        }
                        Selectability::Ready => {
                            let old_sel_el_id = self.selected.borrow().clone();
                            if let Some(sel_el_id) = old_sel_el_id {
                                if sel_el_id == s_resp.id {
                                    *self.selected.borrow_mut() = None;
                                }
                            }
                        }
                        Selectability::Unselectable => {
                            let old_sel_el_id = self.selected.borrow().clone();
                            if let Some(sel_el_id) = old_sel_el_id {
                                if sel_el_id == s_resp.id {
                                    *self.selected.borrow_mut() = None;
                                }
                            }
                        }
                    }
                    modified_resp = Some(EventResponse::None);
                }
            }

            if let Some(mr) = modified_resp {
                *resp = mr;
            }
        }
        resps.0.extend(extend_resps);
    }

    /// this function will not set the new element to selected if it is unselectable or if it does
    /// not already have selectability (aka a normal element)
    pub fn switch_between_els(
        &self, ctx: &Context, old_el_id: Option<ElementID>, new_el_id: Option<ElementID>,
    ) -> EventResponses {
        if old_el_id == new_el_id {
            return EventResponses::default();
        }

        // first remove the selectability from the old element
        let mut resps = EventResponses::default();
        if let Some(ref old_el_id) = old_el_id {
            let resps_ = self.set_selectability_for_el(ctx, old_el_id, Selectability::Ready);
            resps.extend(resps_);
        }

        if let Some(ref new_el_id) = new_el_id {
            let new_sel = self.get_selectability_for_el(new_el_id);
            if let Some(Selectability::Unselectable) = new_sel {
                return EventResponses::default();
            }
            if new_sel.is_none() {
                return EventResponses::default();
            }
        }

        if let Some(ref new_el_id) = new_el_id {
            let resps_ = self.set_selectability_for_el(ctx, new_el_id, Selectability::Selected);
            resps.extend(resps_);
        }
        *self.selected.borrow_mut() = new_el_id;
        resps
    }

    fn get_selectables_index_for_el(&self, el_id: &ElementID) -> Option<usize> {
        self.selectables.borrow().iter().position(|id| id == el_id)
    }

    /// gets the next ready widget index starting from the startingIndex provided
    fn next_ready_el_id(&self, starting_el_id: Option<ElementID>) -> Option<ElementID> {
        if self.selectables.borrow().is_empty() {
            return None;
        }
        let mut working_index = match starting_el_id {
            Some(ref starting_el_id) => self
                .get_selectables_index_for_el(starting_el_id)
                .unwrap_or(0),
            None => return self.selectables.borrow().first().cloned(),
        };
        let starting_index = working_index;

        for _ in 0..self.selectables.borrow().len() + 1 {
            working_index = (working_index + 1) % self.selectables.borrow().len();
            let Some(working_el_id) = self.selectables.borrow().get(working_index).cloned() else {
                continue;
            };
            let Some(working_sel) = self.get_selectability_for_el(&working_el_id) else {
                continue;
            };
            if working_sel != Selectability::Unselectable {
                return Some(working_el_id.to_string());
            }
            if working_index == starting_index {
                // we've come full circle just return the same el
                return starting_el_id;
            }
        }
        None
    }

    /// gets the previous ready widget index starting from the startingIndex provided
    fn prev_ready_el_id(&self, starting_el_id: Option<ElementID>) -> Option<ElementID> {
        if self.selectables.borrow().is_empty() {
            return None;
        }
        let sel_len = self.selectables.borrow().len();
        let mut working_index = match starting_el_id {
            Some(ref starting_el_id) => self
                .get_selectables_index_for_el(starting_el_id)
                .unwrap_or(sel_len - 1),
            None => return self.selectables.borrow().last().cloned(),
        };
        let starting_index = working_index;

        for _ in 0..self.selectables.borrow().len() + 1 {
            working_index = (working_index + sel_len - 1) % sel_len;
            let Some(working_el_id) = self.selectables.borrow().get(working_index).cloned() else {
                continue;
            };
            let Some(working_sel) = self.get_selectability_for_el(&working_el_id) else {
                continue;
            };
            if working_sel != Selectability::Unselectable {
                return Some(working_el_id.to_string());
            }
            if working_index == starting_index {
                // we've come full circle just return the same el
                return starting_el_id;
            }
        }
        None
    }

    pub fn switch_to_next_widget(&self, ctx: &Context) -> EventResponses {
        let el_id = self.selected.borrow().clone();
        self.switch_between_els(ctx, el_id.clone(), self.next_ready_el_id(el_id))
    }

    pub fn switch_to_prev_widget(&self, ctx: &Context) -> EventResponses {
        let el_id = self.selected.borrow().clone();
        self.switch_between_els(ctx, el_id.clone(), self.prev_ready_el_id(el_id))
    }
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for ParentPaneOfSelectable {
    // XXX delete post widget recall
    //fn receivable(&self) -> SelfReceivableEvents {
    //    // all of the events returned by the widget organizer are set to
    //    // focused because WO.Receivable only returns the events associated with
    //    // widget that is currently active.

    //    let wpes = match *self.active_widget_index.borrow() {
    //        Some(i) => self.widgets.borrow()[i].0.receivable(),
    //        None => SelfReceivableEvents::default(),
    //    };

    //    // Add the widget pane's self events. These are default receivable events of the widget
    //    // organizer
    //    let mut rec = self.pane.receivable();
    //    rec.extend(wpes.0);
    //    rec
    //}

    fn receive_event_inner(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = match &ev {
            Event::Mouse(me) => {
                let mut resps = EventResponses::default();
                if let MouseEventKind::Down(_) = me.kind {
                    let eoz = self.pane.eo.get_el_id_z_order_under_mouse(ctx, me);
                    let new_el_id = eoz.first().map(|(el_id, _)| el_id.clone());
                    let old_selected = self.selected.borrow().clone();
                    // NOTE if new_el is not selectable, then this function will only
                    // unselect the old_selected
                    let resps_ = self.switch_between_els(ctx, old_selected, new_el_id);
                    resps.extend(resps_);
                }
                let (captured, resps_) = self.pane.receive_event(ctx, ev);
                resps.extend(resps_);
                (captured, resps)
            }
            Event::KeyCombo(ref ke) => {
                let ke = ke.clone();
                let (mut captured, mut resps) = self.pane.receive_event(ctx, ev);

                if !captured && !ke.is_empty() {
                    match true {
                        _ if ke[0] == KB::KEY_ESC => {
                            let resps_ = self.unselect_selected(ctx);
                            resps.extend(resps_);
                            captured = true;
                        }
                        _ if ke[0] == KB::KEY_TAB => {
                            let resps_ = self.switch_to_next_widget(ctx);
                            resps.extend(resps_);
                            captured = true;
                        }
                        _ if ke[0] == KB::KEY_BACKTAB => {
                            let resps_ = self.switch_to_prev_widget(ctx);
                            resps.extend(resps_);
                            captured = true;
                        }
                        _ => {}
                    }
                }
                (captured, resps)
            }
            _ => self.pane.receive_event(ctx, ev),
        };
        self.partially_process_sel_resps(ctx, &mut resps);
        (captured, resps)
    }
}
