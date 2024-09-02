/*
const (
    newTabBtnLen = 3 // # of cells new cell '+' btn occupies
    tabBreakLen  = 1 // # of cells tab separator char occupies
    tabPadLen    = 2 // # of cells padding on either side of tab name
)

// Tab represents a tab in a tab bar
type Tab struct {
    name string
    id   yh.ElementID
}

// NewTab creates a new Tab object
func NewTab(name string, id yh.ElementID) Tab {
    return Tab{
        name,
        id,
    }
}

// TabsEl displays the Tabs in a row above the pane of information contained
type TabsEl struct {
    ParentPane
    Tabs         []Tab
    openTabIndex int

    // when navigating if the openTab is beyond the maximum viewable tab
    // then this offset must be set to include the open tab in the viewable
    // range
    tabsRowXOffset uint

    unselectedStyle tcell.Style
    selectedStyle   tcell.Style
    newTabStyle     tcell.Style
    emptyStyle      tcell.Style // content beyond the Tabs
}

var (
// TODO: what is the benefit of registering these commands this way? they're
// coming out as {Ctrl+Rune[p]} when tcell is looking for {Ctrl+P}
/*
    ctrlu = ChKey(ctrl, 'u')
    ctrli = ChKey(ctrl, 'i')
    ctrlo = ChKey(ctrl, 'o')
    ctrlp = ChKey(ctrl, 'p')
    ctrlt = ChKey(ctrl, 't')

        ctrlu = "Ctrl+U"
        ctrli = "Ctrl+I"
        tab   = "Tab"
        ctrlo = "Ctrl+O"
        ctrlp = "Ctrl+P"
        ctrlt = "Ctrl+T"
*/
)

// NewTabsEl creates a new TabsEl object
func NewTabsEl(unselectedStyle, selectedStyle, newTabStyle,
    emptyStyle tcell.Style) *TabsEl {

    tabsEl := TabsEl{
        ParentPane:      NewEmptyParentPane(),
        Tabs:            []Tab{},
        openTabIndex:    0, // open the first tab by default
        unselectedStyle: unselectedStyle,
        selectedStyle:   selectedStyle,
        newTabStyle:     newTabStyle,
        emptyStyle:      emptyStyle,
    }

    ecs := []yh.PrioritizableEv{
        yh.NewEC(yh.TabKey),
        yh.NewEC("Ctrl+U"),
        yh.NewEC("Ctrl+I"),
        yh.NewEC("Ctrl+O"),
        yh.NewEC("Ctrl+P"),
        yh.NewEC("Ctrl+T"),
        yh.NewCommandStr("tab"),
    }
    tabsEl.SelfEvs = yh.NewPriorityEvs(yh.Focused, ecs)
    return &tabsEl
}

var _ yh.Element = &TabsEl{}

func (t *TabsEl) ReceiveEvent(ctx yh.Context, ev interface{}) (
    captured bool, resp yh.EventResponse) {

    switch ev := ev.(type) {
    case yh.KeysEvent:
        resp = t.ReceiveEventCombo(ctx, ev)
        return true, resp
    case yh.MouseEvent:
        resp = t.ReceiveMouseEvent(ctx, ev)
        return true, resp
    case yh.CommandEvent:
        return t.ReceiveCommandEvent(ctx, ev)
    case yh.ResizeEvent:
        t.ResizeEvent(ctx)
        return false, resp // resize events aren't captured
    default:
        return t.ParentPane.ReceiveEvent(ctx, ev)
    }
}

// ReceiveEventCombo checks the tab element for a matching event combo and
// executes associated logic. If no match is found, the event is passed to the
// tab element's children
func (t *TabsEl) ReceiveEventCombo(ctx yh.Context, evs []*tcell.EventKey) (
    evResp yh.EventResponse) {

    switch {
    case yh.UCtrlEKC.Matches(evs): // move left in tab list
        if t.openTabIndex > 0 {

            evResp.SetInputabilityChanges(t.OpenTab(t.Tabs[t.openTabIndex-1].id, ctx))
            t.adjustTabsRowXOffset(ctx)
        }

        return evResp

    case yh.PCtrlEKC.Matches(evs): // go to next tab in tab list
        if t.openTabIndex < len(t.Tabs)-1 {

            evResp.SetInputabilityChanges(t.OpenTab(t.Tabs[t.openTabIndex+1].id, ctx))
            t.adjustTabsRowXOffset(ctx)
        }

        return evResp

    case yh.TabEKC.Matches(evs): // move tab left in tab list
        // NOTE: Tab and Ctrl+I are the same for some reason

        if t.openTabIndex > 0 {
            t.Tabs = moveTabInArr(t.Tabs, t.openTabIndex, t.openTabIndex-1)
            t.openTabIndex--
        }

        t.adjustTabsRowXOffset(ctx)
        return evResp

    case yh.OCtrlEKC.Matches(evs): // move tab right in tab list

        if t.openTabIndex < len(t.Tabs)-1 {
            t.Tabs = moveTabInArr(t.Tabs, t.openTabIndex, t.openTabIndex+1)
            t.openTabIndex++
        }

        t.adjustTabsRowXOffset(ctx)
        return evResp

    case yh.TCtrlEKC.Matches(evs): // create new tab

        // create new tab
        titleStr, p := t.GenerateJunkTabDeets()
        evResp.SetInputabilityChanges(t.OpenNewTabAtPos(ctx, titleStr, &p, t.openTabIndex+1))
        return evResp

    default:
    }

    _, evResp = t.EO.KeyEventsProcess(evs)

    return evResp
}

// Passes ChangesInInputability to the parent of this element, while
// optionally making changes to the calling element organizer's prioritizers.
func (t *TabsEl) PropagateUpwardChangesToInputability(childEl yh.Element,
    ic yh.InputabilityChanges, updateThisElementsPrioritizers bool) {

    t.PropagateUpwardChangesToInputabilityHelper(childEl, t, ic,
        updateThisElementsPrioritizers)
}

// OpenNewTabAtPos creates a new tab at the given position in the Tabs list, and
// then focuses on it
// It takes a yh.Context to get the screen size, a title for the tab list, and a
// pointer to an element as the content of the tab
func (t *TabsEl) OpenNewTabAtPos(ctx yh.Context, title string, el yh.Element,
    pos int) (ic yh.InputabilityChanges) {

    _ = t.CreateNewTabAtPos(title, el, pos)
    ic = t.OpenTab(t.Tabs[pos].id, ctx)
    t.openTabIndex = pos
    t.adjustTabsRowXOffset(ctx)

    return ic
}

// CloseTabAtPos closes the tab at the given position in the Tabs list
func (t *TabsEl) CloseTabAtPos(ctx yh.Context, pos int) (ic yh.InputabilityChanges) {

    t.Tabs = removeTabFromArr(t.Tabs, pos)

    switch {
    case pos == t.openTabIndex && pos == len(t.Tabs):
        // close current tab which is also last tab
        t.openTabIndex--
        ic = t.OpenTab(t.Tabs[t.openTabIndex].id, ctx)
    case pos == t.openTabIndex && pos < len(t.Tabs):
        // close current tab which is not last tab
        t.openTabIndex = pos
        ic = t.OpenTab(t.Tabs[t.openTabIndex].id, ctx)
    default:
    }

    t.adjustTabsRowXOffset(ctx)

    return ic
}

// CloseCurrentTab closes the currently open tab
func (t *TabsEl) CloseCurrentTab(ctx yh.Context) (ic yh.InputabilityChanges) {

    return t.CloseTabAtPos(ctx, t.openTabIndex)
}

// ReceiveMouseEvent directs the mouse event to the proper destination within
// the Tabs element.
// It either switches the focus to the tab that was clicked or
// sends the event to the element associated with the open tab.
func (t *TabsEl) ReceiveMouseEvent(ctx yh.Context, ev *tcell.EventMouse) (
    evResp yh.EventResponse) {

    // NOTE: position is adjusted to fit the context of the current element, ie
    // 0,0 is the top left of the Tabs element
    clickX, clickY := ev.Position()

    switch clickY {
    case 0: // if clicked on the Tabs row
        evResp.SetInputabilityChanges(t.handleTabsRowClick(ctx, clickX, clickY))
        return evResp

    default: // if clicked on the content pane
        id := t.Tabs[t.openTabIndex].id
        el := t.EO.GetElementByID(id)
        _, evResp = el.ReceiveEvent(ctx, yh.MouseEvent(ev))
        return evResp
    }
}

// Drawing compiles all of the DrawChPos necessary to draw this element
func (t *TabsEl) Drawing(ctx yh.Context) (chs []yh.DrawChPos) {

    x, y := 0, 0

    tbdcps := t.tabBar(ctx) // get tab bar DrawChPos'

    // compile tab bar locations
    for _, dcp := range tbdcps {
        chs = append(chs, yh.NewDrawChPos(dcp, x, y))
        x++
    }

    // draw current tab's content
    tab := t.EO.GetElementByID(t.Tabs[t.openTabIndex].id)
    tCtx := t.EO.GetContextForElID(t.Tabs[t.openTabIndex].id) // get tab context
    tdcps := tab.Drawing(tCtx)

    // adjust pane locations and trim to adjust for tab bar above
    for _, dcp := range tdcps {

        dcp.Y++

        // exclude any drawChPos that exceed the tabEl ctx
        if dcp.Y > ctx.S.Height-1 {
            continue
        }

        chs = append(chs, dcp)
    }

    return chs
}

// ReceiveCommandEvent attempts to execute a command through the element
// organizer (and it's associated children elements) of the Tabs element before
// checking the Tabs element itself
func (t *TabsEl) ReceiveCommandEvent(ctx yh.Context, ev yh.CommandEvent,
) (cmdExecuted bool, resp yh.EventResponse) {

    // attempt to execute command through child tree
    cmdExecuted, resp = t.EO.ReceiveCommandEvent(ev)

    if cmdExecuted {

        if resp.GetDestruct() { // destroy the current tab
            resp.SetInputabilityChanges(t.CloseTabAtPos(ctx, t.openTabIndex))
            resp.RemoveDestruct()
        }
        return cmdExecuted, resp
    }

    // if no children executed cmd, check for commands registered w/ TabsEl
    switch ev.Cmd {
    case "tab":
        cmdExecuted, resp = t.handleTabCommand(ctx, ev.Args)
    }
    return cmdExecuted, resp
}

// ------------------------------------------------------------

// RemoveTab removes tab from Tabs list and updates priorities to focus on next tab
func (t *TabsEl) RemoveTab(elID yh.ElementID, ctx yh.Context) (ic yh.InputabilityChanges) {

    t.EO.RemoveElement(elID) // remove from element organizer

    // remove tab from Tabs arr
    i := t.tabsIndexByID(elID)
    t.Tabs = removeTabFromArr(t.Tabs, i)

    switch {
    case i < t.openTabIndex: // tab to left of focused removed
        t.openTabIndex--

    case i == t.openTabIndex: // tab in focus removed

        if t.openTabIndex >= len(t.Tabs) { // tab in focus furtherst left
            t.openTabIndex--
        }

        ic = t.OpenTab(t.Tabs[t.openTabIndex].id, ctx)

    }

    t.adjustTabsRowXOffset(ctx)

    return ic
}

// CreateNewTabAtPos creates a new tab at the given position in the Tabs list
// and returns the new tab's yh.ElementID
// NOTE: this does not change the focus to the new tab
func (t *TabsEl) CreateNewTabAtPos(name string, el yh.Element, pos int) yh.ElementID {

    // TODO fix the location
    id := t.EO.AddElement(el, t, yh.NewEmptyLocations(), true)
    newTab := NewTab(name, id)

    if pos >= len(t.Tabs)-1 { // tab is being added to the end of the Tabs list
        t.Tabs = append(t.Tabs, newTab)
    } else { // tab is being added to the middle of the Tabs list
        t.Tabs = insertTabInArr(t.Tabs, newTab, pos)
    }

    return id
}

// adjustTabsRowXOffset ensures that the open tab is visible in the tab row
// and adjusts the offset if necessary
func (t *TabsEl) adjustTabsRowXOffset(ctx yh.Context) {

    var ex int // end x pos of tab in focus w/ no offset
    for _, tab := range t.Tabs[:t.openTabIndex+1] {
        ex += len(tab.name) + tabBreakLen
    }

    // start x pos of tab in focus w/ no offset
    sx := ex - len(t.Tabs[t.openTabIndex].name) - tabBreakLen

    ttl := len(t.tabText())            // total length of tab row
    tex := ttl - int(t.tabsRowXOffset) // final end x pos of tab row w/ offset
    s := ctx.S.Width - newTabBtnLen    // space available for Tabs

    switch {
    case ex >= s: // focused tab off right side of screen
        diff := math.Abs(float64(ttl - s))
        t.tabsRowXOffset = uint(diff)

    case sx-int(t.tabsRowXOffset) < 0: // focused tab off left side of screen
        diff := math.Abs(float64(sx))
        t.tabsRowXOffset = uint(diff)

    case ttl >= s && tex < s:
        // tab row is too long for screen but not filling available tab space
        diff := math.Abs(float64(ttl - s))
        t.tabsRowXOffset = uint(diff)
    }
}

// TODO Remove yh.Context from input
func (t *TabsEl) UnfocusTab(id yh.ElementID, ctx yh.Context) yh.InputabilityChanges {

    // UNFOCUS should:
    // - leave the tab's children's priorities alone
    // - change the Tab's personal priority for SelfEvs to 3
    // - change the Tab's priority for all other elements to 3

    ic := t.EO.ChangePriorityForEl(id, yh.Unfocused)

    t.EO.SetVisibilityForEl(id, false)                         // hide tab
    t.EO.UpdateElPrimaryLocation(id, yh.NewNegativeLocation()) // set empty location for current

    return ic

}

func (t *TabsEl) FocusTab(id yh.ElementID, ctx yh.Context) yh.InputabilityChanges {

    ic := t.EO.ChangePriorityForEl(id, yh.Focused) // increase priority of new open tab

    t.EO.SetVisibilityForEl(id, true) // show tab
    loc := yh.NewLocation(0, ctx.S.Width-1, 1, ctx.S.Height-1, 0)
    t.EO.UpdateElPrimaryLocation(id, loc) // update location of new open tab

    return ic

}

// OpenTab changes the open tab to the tab with the given id, lowering
// the priority of the old open tab and raising the priority of the new open tab
func (t *TabsEl) OpenTab(id yh.ElementID, ctx yh.Context) yh.InputabilityChanges {

    // unfocus old tab ---------------------------------------
    curID := t.Tabs[t.openTabIndex].id // current id
    ic := t.UnfocusTab(curID, ctx)

    // focus/setup new open tab -------------------------------
    t.openTabIndex = t.tabsIndexByID(id) // set new open tab index
    cti2 := t.FocusTab(id, ctx)
    ic.Concat(cti2)

    return ic
}

func (t *TabsEl) ResizeEvent(ctx yh.Context) {

    // update location of open tab
    id := t.Tabs[t.openTabIndex].id
    loc := yh.NewLocation(0, ctx.S.Width-1, 1, ctx.S.Height-1, 0)
    t.EO.UpdateElPrimaryLocation(id, loc) // change location of new open tab

    // update location of all children of open tab
    for id, el := range t.EO.Elements {
        elCtx := t.EO.GetContextForElID(id)
        el.ReceiveEvent(elCtx, yh.ResizeEvent{})
    }
}

//---------------------------------------------------------------------

// draw tab bar with '+' button on the far right
func (t *TabsEl) tabBar(ctx yh.Context) []yh.DrawCh {

    out := []yh.DrawCh{}
    w := ctx.S.Width // TabEl width

    // make 'new tab' btn
    pdc := yh.NewDrawCh(' ', false, t.newTabStyle) // new btn padding draw ch
    nbdc := []yh.DrawCh{}                          // new btn draw chs
    nbdc = append(nbdc, pdc)
    nbdc = append(nbdc, yh.NewDrawCh('+', false, t.newTabStyle))
    nbdc = append(nbdc, pdc)

    tt := t.tabText() // get tab name list DrawChs

    switch {
    case len(tt) < w: // tab text doesn't fill screen

        // add 'new' btn to end of tab list
        out = append(out, tt...)
        out = append(out, nbdc...)

        // fill in the remainder of the row with empty cells
        x := len(tt)
        for ; x < w; x++ {
            out = append(out, yh.NewDrawCh(' ', false, t.emptyStyle))
        }

    default: // tab list wider than screen

        // trim slice of DrawChs to fit screen width
        tto := tt[t.tabsRowXOffset:] // offset Tabs text DrawChs

        // trim slice of tab text DrawChs to fit 'new' btn
        out = append(out, tto[:w-len(nbdc)]...)
        out = append(out, nbdc...) // add 'new' btn to end of tab list
    }

    return out
}

// get yh.DrawCh's for names of all current Tabs in tab bar
func (t *TabsEl) tabText() []yh.DrawCh {

    out := []yh.DrawCh{}
    for i, tab := range t.Tabs {

        style := t.unselectedStyle
        if i == t.openTabIndex {
            style = t.selectedStyle
        }

        // add padding to front of tab name
        pdc := yh.NewDrawCh(' ', false, style) // padding draw ch
        out = append(out, pdc)

        // add each tab name to tab text yh.DrawCh slice
        for _, r := range tab.name {
            out = append(out, yh.NewDrawCh(r, false, style))
        }

        out = append(out, pdc)                             // add padding to end of tab name
        out = append(out, yh.NewDrawCh('⎤', false, style)) // add tab seperator element
    }
    return out
}

// handleTabsRowClick handles a click on the Tabs row
func (t *TabsEl) handleTabsRowClick(ctx yh.Context, clickX,
    clickY int) (ic yh.InputabilityChanges) {

    sx := -int(t.tabsRowXOffset) // start x pos of tab relative to screen start

    for i, tab := range t.Tabs {

        // find end x pos of tab
        w := len(tab.name) + tabBreakLen + tabPadLen // tab width

        ex := sx + w - 1 // end x pos of tab relative to screen start

        // determine if click landed on this tab in list
        if clickX >= sx && clickX <= ex && clickY == 0 {

            t.openTabIndex = i // shift to selected tab
            ic = t.OpenTab(tab.id, ctx)
            return ic
        }

        sx += w
    }

    // determine if click lands on new tab btn
    if clickX >= sx && clickX <= sx+newTabBtnLen && clickY == 0 {

        // create new tab
        titleStr, p := t.GenerateJunkTabDeets()
        pos := t.openTabIndex + 1
        ic = t.OpenNewTabAtPos(ctx, titleStr, &p, pos)
    }

    return ic
}

// tabsIndexByID returns the index of the tab with the given id, in the Tabs list
func (t *TabsEl) tabsIndexByID(id yh.ElementID) int {

    for i, tab := range t.Tabs {
        if tab.id == id {
            return i
        }
    }
    return -1
}

// handleTabCommand handles the "tab" command
func (t *TabsEl) handleTabCommand(ctx yh.Context, args []string) (cmdExecuted bool,
    resp yh.EventResponse) {

    cmdExecuted = false
    titleStr, p := t.GenerateJunkTabDeets()
    pos := t.openTabIndex + 1

    switch len(args) {

    case 0: // no args, create new tab at end of Tabs list
        resp.SetInputabilityChanges(t.OpenNewTabAtPos(ctx, titleStr, &p, pos))
        cmdExecuted = true

    case 1: // one arg, create new tab at given pos

        // check if first arg is a number
        i, err := strconv.Atoi(args[0])

        switch {
        case err != nil: // arg not a number
            errStr := fmt.Sprintf("unknown arg: %v called with 'tab'", args[0])
            resp.SetMetadata(yh.StrToDrawChs(errStr, tcell.StyleDefault))

        case i > len(t.Tabs)-1: // pos is greater than number of Tabs
            errStr := fmt.Sprintf("position out of range: %v", i)
            resp.SetMetadata(yh.StrToDrawChs(errStr, tcell.StyleDefault))

        default: // valid args
            resp.SetInputabilityChanges(t.OpenNewTabAtPos(ctx, titleStr, &p, i))
            cmdExecuted = true
        }

    default: // more than one arg
        errStr := "too many args: 'tab' takes 1 argument"
        resp.SetMetadata(yh.StrToDrawChs(errStr, tcell.StyleDefault))
    }

    return cmdExecuted, resp
}

// inserts Tab at given index in array of Tabs
func insertTabInArr(array []Tab, value Tab, index int) []Tab {
    arr := append([]Tab{value}, array[index:]...) // second half of input arr
    return append(array[:index], arr...)
}

// removes Tab at given index from array of Tabs
func removeTabFromArr(array []Tab, index int) []Tab {
    return append(array[:index], array[index+1:]...)
}

// moves Tab from initial to given index in array of Tabs
func moveTabInArr(array []Tab, srcIndex int, dstIndex int) []Tab {

    // prevent movement out of bounds of array
    if srcIndex == 0 && dstIndex <= 0 || srcIndex == len(array)-1 &&
        dstIndex >= len(array)-1 {
        return array
    }

    value := array[srcIndex]
    arr := removeTabFromArr(array, srcIndex)
    return insertTabInArr(arr, value, dstIndex)
}

// TODO THROW-AWAY FOR GENERATING TABS --- DELETE LATER
func (t *TabsEl) GenerateJunkTabDeets() (titleStr string, p StandardPane) {

    // ----------------------------------------------------------------------
    // TODO THROWAWAY TEMP CODE
    // generate random color so stuff is easier to differentiate
    rn := rand.Intn(3)
    var style tcell.Style

    switch rn {
    case 0:
        style = tcell.StyleDefault.Foreground(tcell.ColorWhite).Background(tcell.ColorBlue)
    case 1:
        style = tcell.StyleDefault.Foreground(tcell.ColorWhite).Background(tcell.ColorGreen)
    case 2:
        style = tcell.StyleDefault.Foreground(tcell.ColorWhite).Background(tcell.ColorRed)
    }

    // generate title based on largest id
    for i := 0; i < 8; i++ {
        largestIDStr := strconv.Itoa(int(t.EO.Largest + 2))
        titleStr = titleStr + largestIDStr
    }

    pContent := [][]yh.DrawCh{yh.StrToDrawChs(titleStr, style)}
    pDefaultCh := yh.NewDrawCh(' ', false, style)
    pDefaultLine := []yh.DrawCh{}
    p = NewStandardPane(pContent, pDefaultCh, pDefaultLine, 0, 0)

    return titleStr, p
    // TODO THROWAWAY TEMP CODE ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    // ----------------------------------------------------------------------
}
*/
