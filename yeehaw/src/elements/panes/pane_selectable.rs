use {
    crate::{Keyboard as KB, *},
    crossterm::event::{MouseButton, MouseEventKind},
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

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
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
        self.set_selectability(Selectability::Unselectable, true)
    }

    pub fn enable(&self) -> EventResponses {
        self.set_selectability(Selectability::Ready, true)
    }

    pub fn deselect(&self) -> EventResponses {
        self.set_selectability(Selectability::Ready, false)
    }

    pub fn select(&self) -> EventResponses {
        self.set_selectability(Selectability::Selected, false)
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
            Err(e) => {
                log_err!("error deserializing selectability: {}", e);
                Selectability::Ready
            }
        }
    }

    fn set_attr_selectability(&self, s: Selectability) {
        let bz = match serde_json::to_vec(&s) {
            Ok(v) => v,
            Err(e) => {
                log_err!("error serializing selectability: {}", e);
                return;
            }
        };
        self.set_attribute(ATTR_SELECTABILITY, bz)
    }

    /// sets the selectability of the widget, if the widget is unselectable then the selectability
    /// will not be set unless force_set is true.
    pub fn set_selectability(&self, s: Selectability, force_set: bool) -> EventResponses {
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

        if !force_set && prev_sel == Some(Selectability::Unselectable) {
            return EventResponses::default();
        }

        let mut resps = EventResponses::default();
        match s {
            Selectability::Selected => {
                // NOTE needs to happen before the next line or else receivable will return the
                // wrong value
                self.set_attr_selectability(s);
                resps.push(EventResponse::BringToFront);

                let bz = serde_json::to_vec(&SelectabilityResp {
                    sel: s,
                    id: self.id().clone(),
                });
                match bz {
                    Ok(bz) => {
                        resps.push(EventResponse::Custom(
                            Self::RESP_SELECTABILITY_WAS_SET.to_string(),
                            bz,
                        ));
                    }
                    Err(e) => {
                        log_err!("error serializing selectability resp: {}", e);
                    }
                }
            }
            Selectability::Ready | Selectability::Unselectable => {
                if let Some(Selectability::Selected) = prev_sel {
                    let bz = serde_json::to_vec(&SelectabilityResp {
                        sel: s,
                        id: self.id().clone(),
                    });
                    match bz {
                        Ok(bz) => {
                            resps.push(EventResponse::Custom(
                                Self::RESP_SELECTABILITY_WAS_SET.to_string(),
                                bz,
                            ));
                        }
                        Err(e) => {
                            log_err!("error serializing selectability resp: {}", e);
                        }
                    }
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
    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        let attr_sel = self.get_selectability();
        if let Selectability::Selected = attr_sel {
            self.pane.receivable()
        } else {
            Vec::with_capacity(0)
        }
    }

    fn can_receive(&self, ev: &Event) -> bool {
        let attr_sel = self.get_selectability();
        if let Selectability::Selected = attr_sel { self.pane.can_receive(ev) } else { false }
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = match ev {
            Event::Mouse(ref me) => {
                if matches!(me.kind, MouseEventKind::Up(MouseButton::Left)) {
                    (false, self.select())
                } else {
                    (false, EventResponses::default())
                }
            }
            Event::ExternalMouse(ref me) => {
                if matches!(
                    me.kind,
                    MouseEventKind::Down(MouseButton::Left)
                        | MouseEventKind::Drag(MouseButton::Left)
                        | MouseEventKind::Up(MouseButton::Left)
                ) {
                    let resps = self.deselect();
                    (false, resps)
                } else {
                    (false, EventResponses::default())
                }
            }
            Event::Custom(ref ev_name, ref bz) => {
                debug!("custom ev_name: {:?}", ev_name);
                if ev_name == ParentPaneOfSelectable::EV_SET_SELECTABILITY {
                    match serde_json::from_slice(bz) {
                        Ok(v) => (true, self.set_selectability(v, false)),
                        Err(e) => {
                            log_err!("error deserializing selectability: {}", e);
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

    pub fn opaque(ctx: &Context) -> SelStyles {
        SelStyles {
            selected_style: Style::opaque(ctx, Color::YELLOW, 40),
            ready_style: Style::transparent(),
            unselectable_style: Style::opaque(ctx, Color::GREY10, 200),
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

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl ParentPaneOfSelectable {
    pub const KIND: &'static str = "parent_pane_of_selectable";
    pub const EV_SET_SELECTABILITY: &'static str = "set_selectability";

    pub fn default_receivable_events() -> ReceivableEvents {
        ReceivableEvents(vec![
            (KB::KEY_ESC.into()),
            (KB::KEY_TAB.into()),
            (KB::KEY_BACKTAB.into()),
        ])
    }

    pub fn new(ctx: &Context) -> ParentPaneOfSelectable {
        let pane = ParentPane::new(ctx, Self::KIND)
            .with_focused_receivable_events(Self::default_receivable_events());
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
            Err(e) => {
                log_err!("error deserializing selectability: {}", e);
                return None;
            }
        };
        Some(sel)
    }

    #[must_use]
    fn set_selectability_for_el(
        &self, ctx: &Context, el_id: &ElementID, s: Selectability,
    ) -> EventResponses {
        let ev_bz = serde_json::to_vec(&s);
        match ev_bz {
            Ok(ev_bz) => {
                let ev = Event::Custom(Self::EV_SET_SELECTABILITY.to_string(), ev_bz);
                let mut resps = self.pane.send_event_to_el(ctx, el_id, ev);
                self.partially_process_sel_resps(ctx, &mut resps);
                resps
            }
            Err(e) => {
                log_err!("error serializing selectability: {}", e);
                EventResponses::default()
            }
        }
    }

    pub fn remove_element(&self, el_id: &ElementID) {
        self.pane.remove_element(el_id);
        self.selectables.borrow_mut().retain(|id| id != el_id);
    }

    pub fn clear_elements(&self) {
        self.pane.clear_elements();
        self.selectables.borrow_mut().clear();
    }

    #[must_use]
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

            if let EventResponse::Custom(k, v_bz) = resp {
                if k == SelectablePane::RESP_SELECTABILITY_WAS_SET {
                    let s_resp: SelectabilityResp = match serde_json::from_slice(v_bz) {
                        Ok(v) => v,
                        Err(e) => {
                            log_err!("error deserializing selectability resp: {}", e);
                            continue;
                        }
                    };
                    //debug!("selectability was set: {:?}", s_resp);
                    match s_resp.sel {
                        Selectability::Selected => {
                            let old_sel_el_id = self.selected.borrow().clone();
                            *self.selected.borrow_mut() = Some(s_resp.id.clone());
                            if let Some(old_sel_el_id) = old_sel_el_id {
                                if old_sel_el_id != s_resp.id {
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
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let (captured, mut resps) = match &ev {
            Event::KeyCombo(ref ke) => {
                let ke = ke.clone();
                let (mut captured, mut resps) = self.pane.receive_event(ctx, ev);

                if !captured && !ke.is_empty() {
                    match true {
                        _ if ke[0] == KB::KEY_ESC => {
                            //debug!("esc, about to unselect: {:?}", self.selected.borrow());
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
