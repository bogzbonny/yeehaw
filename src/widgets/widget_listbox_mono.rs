/*
package widgets

import (
	tcell "github.com/gdamore/tcell/v2"
	yh "keybase.io/nwmod/nwmod/yeehaw"
	els "keybase.io/nwmod/nwmod/yeehaw/elements"
)

type ListBoxMono struct {
	*ListBox
	SelectedIndex int
}

var _ Widget = &ListBoxMono{}

// Creates a new ListBoxMono.
// NOTE: a negative selectedIndex or an index outside the range of the item
// array will result in no item being selected
func NewListBoxMono(pCtx yh.Context, items []ListBoxItem, SelectionChangedFn func([]ListBoxItem) yh.EventResponse,
	selectedIndex int) *ListBoxMono {

	l := &ListBoxMono{
		ListBox:       NewListBox(pCtx, items, SelectionChangedFn),
		SelectedIndex: selectedIndex,
	}

	for i := range l.Items {
		if i == selectedIndex {
			l.Items[i].IsSelected = true
		} else {
			l.Items[i].IsSelected = false
		}
	}
	return l
}

func (lbm *ListBoxMono) WithSize(width, height SclVal) *ListBoxMono {
	lbm.Width, lbm.Height = width, height
	lbm.ListBox.UpdateContentText()
	return lbm
}

func (lbm *ListBoxMono) WithStyles(styles WBStyles) *ListBoxMono {
	lbm.Styles = styles
	lbm.ListBox.UpdateContentText()
	return lbm
}

func (lbm *ListBoxMono) WithLinesPerItem(lines int) *ListBoxMono {
	lbm.LinesPerItem = lines
	lbm.Height = NewStatic(len(lbm.Items) * lines)
	lbm.UpdateContentText()
	return lbm
}

func (lbm *ListBoxMono) WithLeftScrollbar() *ListBoxMono {
	lbm.YScrollbar = LeftScrollbar
	return lbm
}

func (lbm *ListBoxMono) WithRightScrollbar() *ListBoxMono {
	lbm.YScrollbar = RightScrollbar
	return lbm
}

func (lbm *ListBoxMono) WithRightClickMenu(rcm *els.RightClickMenuTemplate) *ListBoxMono {
	lbm.RightClickMenu = rcm
	return lbm
}

func (lbm *ListBoxMono) At(locX, locY SclVal) *ListBoxMono {
	lbm.WidgetBase.At(locX, locY)
	return lbm
}

func (lbm *ListBoxMono) ToWidgets() Widgets {
	out := Widgets{lbm}
	lbm.AppendScrollbarWidget(&out)
	return out
}

// Handles the selection of an item at the given index
// NOTE: this function does not check if the index is valid
func (lbm *ListBoxMono) SelectIndex(index int) {

	switch {
	case lbm.SelectedIndex != index && lbm.SelectedIndex >= 0: // another item is selected
		// toggle clicked item
		lbm.Items[index].IsSelected = !lbm.Items[index].IsSelected

		// toggle previously selected item
		// check if previous item still exists
		if lbm.SelectedIndex <= len(lbm.Items)-1 {
			lbm.Items[lbm.SelectedIndex].IsSelected = !lbm.Items[lbm.SelectedIndex].IsSelected
		}
		lbm.SelectedIndex = index // update selected index

	case lbm.SelectedIndex < 0: // no item selected
		// toggle clicked item
		lbm.Items[index].IsSelected = !lbm.Items[index].IsSelected

		lbm.SelectedIndex = index // update selected index
	}

	lbm.Cursor = index // update cursor

	if lbm.SelectionChangedFn != nil {
		lbm.SelectionChangedFn(lbm.Items)
	}

}

// remove item from list
func (lbm *ListBoxMono) RemoveItem(index int) {

	// update items
	newItems := append(lbm.Items[:index], lbm.Items[index+1:]...)
	lbm.SetItems(newItems)

	if len(lbm.Items) == 0 { // no issues left
		lbm.SelectedIndex = -1
		lbm.UpdateContentText()
		return
	}

	// update selected index
	switch {
	case lbm.SelectedIndex == len(lbm.Items) &&
		len(lbm.Items) > 1: // issue at end of list deleted

		lbm.SelectedIndex--
		lbm.Cursor--
		lbm.Items[lbm.SelectedIndex].IsSelected = true

	case lbm.SelectedIndex < len(lbm.Items): // go to next issue

		lbm.Items[lbm.SelectedIndex].IsSelected = true

	case lbm.SelectedIndex == len(lbm.Items) &&
		len(lbm.Items) == 1: // final issue deleted

		newIndex := 0
		lbm.SelectedIndex = newIndex
		lbm.Items[newIndex].IsSelected = true
	}

	lbm.UpdateContentText()
}

func (lbm *ListBoxMono) SetSelectability(s Selectability) yh.EventResponse {
	if lbm.Selectedness == Selected && s != Selected {
		lbm.Cursor = 0
	}
	if s == Selected && lbm.SelectedIndex != -1 {
		lbm.Cursor = lbm.SelectedIndex
	}

	return lbm.WidgetBase.SetSelectability(s)
}

func (lbm *ListBoxMono) ReceiveKeyEventCombo(evs []*tcell.EventKey) (captured bool, resp yh.EventResponse) {
	if lbm.Selectedness != Selected { // widget not selected
		return false, yh.NewEventResponse()
	}

	if yh.EnterEKC.Matches(evs) {
		if lbm.Cursor >= 0 && lbm.Cursor < len(lbm.Items) {
			lbm.SelectIndex(lbm.Cursor)
		}
		return true, yh.NewEventResponse()
	}
	return lbm.ListBox.ReceiveKeyEventCombo(evs)
}

func (lbm *ListBoxMono) ReceiveMouseEvent(ev *tcell.EventMouse) (captured bool, resp yh.EventResponse) {

	if ev.Buttons() == tcell.Button1 {
		_, y := ev.Position() // click position
		// get item index at click position
		itemIndex := lbm.GetItemIndexForViewY(y)
		if itemIndex < 0 || itemIndex >= len(lbm.Items) {
			// invalid item index
			return false, yh.NewEventResponse()
		}
		lbm.SelectIndex(itemIndex)
		return true, yh.NewEventResponse()
	}
	return lbm.ListBox.ReceiveMouseEvent(ev)
}
*/
