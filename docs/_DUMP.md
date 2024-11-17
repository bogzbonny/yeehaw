
01. listbox scrollbar never activated
01. listbox not selecting with up and down arrows after a resize
    - independant if we use a flex or fixed width
    - seems to be routed to the ParentPaneOfSelectable, but NOT the top-level
      ListBox el even though the ListBox has it set to receivable 
01. 74b6e9b breaks checkbox drawing
01. Each element should become SELF selected when it receives a mouse event then
    unselect itself on the first external mouse click 
     - this way an element such as the dropdown list would still be able to use
     the up and down arrow keys once its selected EVEN IF it wasn't a part of a 
     ParentPaneOfSelectables
     - external mouse click logic to exist in SelectablePane
     - currently not deregistering OR registering properly from mouse click
01. micro shadow buttons 
      ░▏
01. Special border for windows: left & right scrollbars, no top, thin 8th left
    line ** Bottom righthand corner as scrollbar!

01. Buttons on window test are acting mad funny
     - click but then release somewhere else (making the button focused) then
       click again - tries to re-register events
     - replace with microshadow buttons while I'm at it
01. window test, all the main buttons are staying highlighted after they're clicked
          (easy fix) 
01. click down in textbox (or dropdownlist), then click again and drag to listbox then unclick.. this will panic on priotizer
    registers
      - likely due to the fact that external mouse events are processed after
        mouse events... 
        - solution, only check for conflicts in prioratizer after all events
          have been processed.
      - hmmmm but doesn't QUITE make sense, should have deselected from the
        textbox on first drag outside of the textbox
      - interesting so... tb hasn't actually been selected because the Up click
        was never sent to the widget... 
      - STILL Buggy for click dropdownlist, click empty space, then click button
01. scrolling tb while deselected doesn't scroll the linenumbers or the
    scrollbars

WIDGET RECALL REFACTOR
 - Selectibility Hook
    - this one is a bit harder maybe we have to introduce hooks on 
      arbitrary attribute setting. The existing hook structure should
      be able to hold hooks of this variety 
    - could maybe do something funny where the hook is actually set to 
      the name of the attribute KEY plus a prefix! thus allowing the hook
      system to specify the exact type of the hook.
    - WONT DO Remove "Unselectable type" from selectibility which will just be the regular element
 - deleted ElWidgetPane for a new type ParentPaneWithSelectibility
    - basically the parentpane with tab capturing and refocusing
    - using the ParentPane store to hold index of the currently selected widget 
    - would just need to specify each element as either "selectible" or "not"
      when it gets added to the widget macro
 - Test if each element has selectibility by use of the attributes
 - move Widgets -> Elements (basically only used for labels)

.to_widgets() actually used here:
yeehaw/src/widgets/widget_textbox.rs:348:    pub fn to_widgets(mut self, ctx: &Context) -> Widgets {
yeehaw/src/widgets/widget_textbox_numbers.rs:118:    pub fn to_widgets(&self, ctx: &Context) -> Widgets {

 - ensure that pre-hooks of set selectibility propogate their resps upwards
   (instead of returning them)

el_widget_pane.rs          | DONE/TO_DELETE                                  
mod.rs                     | TO_DELETE                       
widget.rs                  | TO_DELETE                          
widget_button.rs           | DONE
widget_checkbox.rs         | DONE
widget_dropdownlist.rs     | DONE
widget_figlet.rs           | DONE (not selectable)
widget_label.rs            | DONE (not selectable)                            
widget_listbox.rs          | DONE
widget_radio.rs            | DONE
widget_scrollbar.rs        | DONE (not selectable)
widget_textbox.rs          | DONE
widget_textbox_numbers.rs  | DONE
widget_toggle.rs           | DONE
01. replace window CornerAdjuster with border::Corner

01. borders-pane wrapper
     - option for each of up/down/right/left
        - aka. doesn't need to be fully bordered.
     - custom up/down/right/left DrawChs AND corner pieces
     - single/double/bold lines defaults 
     - built in scrollbars optionally
     - drag-resizing - drag while on the edge to resize the location

01. merge PaneWithScrollbars into border pane - introduce scrollbars into border
    pane

40. Scrollbar bug: when dragging scrollbar with mouse, will drag good for a bit
    then close to the end it just moves all the way to the maximum
     - this seems to only be an issue when the scrollbar takes up about half of
       the scrollbar area or more
     - it happens in both vertical and horizontal scrollbars
     - it happens in both scroll forwards and scroll backwards
     - seems like the error exists in drag_backwards_by_1_ch (and forwards_by..)
01. titles inside border: ┌hello───┐
                          │        │
                          │        │
                          └────────┘
01. add tags to cargo file 
01. Add license
01. add license and license-file fields to cargo.toml
     - https://doc.rust-lang.org/cargo/reference/manifest.html
01. create pane basics proceedural macro 
     - it's crazy the amount of duplication required for ParentPane, and now
       SelectablePane, this should be a macro to define all the basic get/set
       functions 
        - will need to be divided into "with" functions and "non-with" functions
          (aka self modifying)

01. listbox scrollbar not showing up when the width is flex

05. NO NEED create builder types for each widget. 
     - sometimes there are fields in the widget that are only 
       used for the creation of the widget, and not for the widget itself.
05. NO NEED Into<Widgets> Trait which can be applied to each widget builder so that
    .to_widgets() doesn't need to be manually called during construction
    (applied in add_widget)
01. make crate into a workspace

10. WONT DO integrate in trait upcasting for Widget Type once available (remove
    el_widget_pane drawing functionality in favour of the parent pane draw).
    https://github.com/rust-lang/rust/issues/65991

01. editor element
    - uses the $EDITOR env variable
    - execute with something like: "$EDITOR; exit" 
       - looks like we won't even need to use the "exit" command 
         if we use the command builder... it will close at the end
         of the command!
       - use the editor with a temp file - check after each event for updates to
         that file
       - WONT DO consider closeable vs non-closeable version of this widget
          - I guess if you wanted an Editor to NOT close, when you closed the
            editor the Pane should be replaced with just the containing text 
              - OR could take a snapshot right when the editor exits
                and use that snapshot except maybe make it a bit more pale

01. fg color alpha channel should be able to choose between the either taking
    from the bg color or the fg of the character below
     - would need to calculate the bg color first then
    Maybe to make things open-ended we could have these kinds of alpha:
    bg-alpha    from: lower-bg, lower-fg
    fg/ul-alpha from: lower-bg, lower-fg, upper-bg
    NOTE we can't allow for the bg to be alpha on the upper-fg or else 
         we have a computational loop which would be annoying to resolve
    These options should exist in the Style and not the Color. 
pub struct Style {
    pub fg: Option<(Color, FgTranspSrc)>,
    pub bg: Option<(Color, BgTranspSrc)>,
    pub underline: Option<(Color, FgTranspSrc)>,
    pub attr: Attributes,
}

01. window shadow!
     - bg color transparent, fg color transparent TO bg color 

10. button: visualize button being clicked
01. multiple terminal windows focus and panic 
    - I THINK because the ParentPane isn't focusing/defocusing so
      not the receivable events are changing only what's registered to the 
      ParentPane.pane which is nothing?!
    - okay - how do we make the window REfocus when it's selected again??!
      - do we change priority to use priority + Z-index?? NOPE
      - do we send back a new event to the Parent to "unfocus all other elements"
        then refocus this element YUP
      - We need ctx on change priority BECAUSE for widgets, changing priority
        means deselecting other widgets which means they need 

01. window x button is killing ALL the terminal windows

01. proper shutdown of other threads in terminal pane (terminal_test doesn't
    completely shut down).
01. masterpty likes to die after 10 readers have been created
     - maybe should use more slaves?
     - needed to use spawn_blocking
01. make window top bar slightly lighter when it's focused
01. window_test scrollable_windows seem to scroll 1 too far!
01. file nav test seems not to work?
     - commit 10a47dd broke it
       - keyboard command nolonger passed up
       - see lines 342 in organizer.rs - which to me make sense however break
         this example
01. textbox greyed out initial message ("type here...") 
01. window_test term window broken (double registers) 
    window_test broken by cca0752
01. file_nav_test broken
    file_nav_test broken by bbd32af
01. 12a753e breaks widget_test and window_test
01. seperate out Event from ReceivableEvent
01. terminal editor logic should not check for editor closure during drawing but use a hook!
01. window_resize not working

30. figure out a nicer way of inheriting element functions from the above
    element besides lots of boilerplate, probably though the use of a macro
      - maybe? https://github.com/datapture/hereditary
      - maybe https://docs.rs/delegate/latest/delegate/
      - gpt query:
      help me write a rust macro which will implement a trait from the first 
      field in a struct which will already implement this trait. The macro
      should allow you to override function implementations, only functions
      which are not written in the macro will be generated based off of the
      first struct field.
help me write a rust macro which will implement a trait from the first 
      field in a struct which will already implement this trait. The macro
      should allow you to override function implementations, only functions
      which are not written in the macro will be generated based off of the
      first struct field. These Overriding functions may have a `&self` field as their first argument
followup
is it possible or not possible to way to make this macro automatically take the
first field of MyStruct rather than having to manually input it in the macro. If
it is possible what would that look like 
     - see macro_brainstorm.md
help me program a rust procedural macro which adds 2 functions (named
"my_function1" and "my_function2") to a Trait impl block if either of functions
doesn't exist in that Trait impl block that this macro is applied too
05. use .flf (figlet font) format instead of custom megatext
     - https://docs.rs/figlet-rs/latest/figlet_rs/
01. alpha not working for bg of debug window in window_test
01. SB bug, if setting size to flex(1.).minus(fixed(x)) for each x the scrollbar
    will actually remove two character spaces (instead of expected 1)
      - I suspect this has to do with the domain_incr
01. wierd bugs with maximized windows (window_test)
 - HAS TODO WITH STACK PANE:             self.normalize_locations(ctx); helps
    - Also when adjusting up the CornerAdjuster gets higher!
       - seems to be only in the y dir not the x-dir
       - seems to be in x and y for scrollable pane inside window
    - create two windows
    - maximize one
    - minimize one
    - the maximized one will not draw on top of the minimized windows 
      however it likely is receiving the events for the entire screen 
      as the minimized window cannot be restored until you shrink the 
      maximized window back.
    - might be a bug with either parent pane or vertical stack
01. floating window element
      - TopBar - title, x button, drag to move the whole window
      - restore minimize only on upclick
      - lower righthand triangle for resizing
      - test with scrollable pane
      - when maximized, if the corner adjustor is used, then reset the maxizer
        button to not maximized.
      - prevent the window from moving further left than the screen... 
         - this makes the buttons and the corner adjuster stop working
      - bug; the top bar is receiving events for top row of the inner
        pane
01. right click menu is way to large
    - only in scrollable_panes_test not in widget_test
    - doesn't occur when the ctx visible region is disabled
    - choice of visible region seems reasonable
    - seems to be a part of the drawing routine for pane

01. menu_test seems to not select the final final sub menu (hi in diner) 
    - NOT due to 9bdebc2 (HEAD -> main, origin/main) made external mouse events relative
    - definately has to do with the event not being routed to the menubar as if
      the only item is the menubar then the menu works fine

01. Ensure that HorizontalStack has all the new functions added to VerticalStack
01. WONT DO - makes things to confusing. make parent Not an Option in Pane
01. terminal element
    - https://github.com/a-kenji/tui-term/blob/development/examples/smux.rs
    - doesn't close window when exit is executed
01. make window automatically focus when it's selected
01. BringToFront a new window when it's created

01. tab key not working to go between widgets in pane_scrollable_test (nor
    escape?) - works for scrollablepane now, but not for pane_with_scrollbars
01. widget_organizer should extend regular organizer not be its totally own
    thing
01. translate tabs 
     - just use buttons as the tabs?!
       - maybe not for tab dragging?
         - have a few buttons that live after the tabs (for the a + button for
           instance)
     - button click should have the button as an input such that it can change
       color when selected
05. ratatui wrapper
     - okay so most rat widget objects are CREATED for each render.. 
     - any wrapper is not super useful unless the details of the widget are
       known - probably best to just help with low level conversions such as
       Buffer
     - https://github.com/benjajaja/ratatui-image

05. Jexer style button clicking 

01. maybe the enum of color could just have a "Transparent"
     color - then remove the transparent bool from DrawCh
     - maybe transparent should be an alpha setting... could still be an integer 
       and could blend with the color behind it. 
        - if applied to the fg, the current fg character would still be the ch
          up to a threshold of maybe 50% alpha (in which case it would use the
          character behind it. 
01. basic file viewer 
01. translate file_navigator
01. prioritizing bug in file_nav_test: 
    - nav is not unfocusing properly 
    - click on the file_viewer and get the duplicate junk
05. color "darker", "lighter" methods
10. button selectable (can hit with enter key)
05. scrollbar shouldn't move if uninitialized and a drag mouse enters it
2. gradient on angles > 90 doesn't work, fix
10. blending two time gradients overflows
10. blending a gradient and time gradient fails
10. gradient color 
     - posibilities: 
         - mono-directional = going either up or down / or diagonal
           as continious (non - radial) 
            - can also be thought of as radiating from a straight line (the
              perpendicular line) 
         - radial 
           - radiating from a point instead of a line
     - gradient moves based on the **LOCAL** screen position. (aka
       position within the parent)
   - Time gradient
     - should pass in the time with the draw context
   - time and screen position gradient.
      - maybe this could be a time gradient, however each color on the gradient
        scale WAS another gradient. (or vice versa) 
     - refactor color to make a call each draw
       - maybe make color an enum for serialization purposes. 
   - Color will need a "blend" function with another color for the gradients
       blended = color1.blend(percent, color2)
     - only linear gradient, but can simulate other functions with different
       position colors
     - gradient params: pos-offset x/y (as DynVal!), 
        grad-colors and positions(DynVal?!) (need multiple positions for rainbows). 
     - after the final position is reached (and before the final position if
       there is an offset) repeat the pattern ... would need the "final length"
       (aka what is the gradient inbetween the final color and the first color)

01. write debug_pane
01. textbox rcm bug.. need to go to the upper right hand corner to first
    activate the rcm 
30. refactor: remove ExtraLocations from EventResponse
05. menu.rs: 
        // this should just be loc width (post refactor of dyn_location to element)
01. Hooks 
     - HashMap(HookKind, Vec(ElementID, fn Hook))
     - register_hook
     - type HookKind = String
        - use string to allow for totally custom widget hook kinds
     - remove_hook(el_id, kind) 
     - remove_all_hooks(el_id)
     - pre/post event hook
     - pre/post location change hook 
     - pre/post visibility change hook
05. Proper overwrite when writing a transparent character. Build in
    functionality to retrieve and draw what the content underneath should be
    even if it's not currently drawn will require new fn on Element
    "GetDrawingAtPos" as well as determining the layer order at a given
    position.
     - I don't think this is an issue now that drawing is contained to the
       single draw function.
01. rewrite horizontal/vertical stack panes

01. WONT DO remove extra locations
     - menu item should manually refer back to the menu-bar element when an
       event is called
     - turns out this is actually very useful if we want to have parent panes
       which have elements outside of their original location (obv!). Otherwise
       we would need to have the parent pane constantly grow and shrink its main
       dimention which would be annoying to track.. basically the same as using
       taffy. - we would then need to do the wierd thing of passing back
       "non-captured" to the EO which would then need to send the event down to
       the next z-index... too-much extra complexity compared to just allowing
       for extra locations
01. menubar doesn't properly render output on top of element below
01. ensure that menu will work in a vertical pane where it goes over other
    content
01. remove loc and vis from add_element within element organizer
01. remove visibility from context
01. special way to not draw outside of max context (scrollable_pane) 
     - may need to add something special to the context.
01. translate scrollable pane 
     - scrollbars should be optional (can scroll with mouse wheel otherwise)
     - interaction with border pane?




01. make the element-id a name (string) which is unique across the entire tui application.
     - the element-id will nolonger be assigned by each element organizer, but
       assigned to each element at it's creation. 
     - a global object called sorting_hat will assign each elements name at
       instantiation. 
     - the sorting_hat object will name elements based on element_type. 
01. Kitty issue: when resizing a lot, there are some artifacts
01. wezterm issue, at larger sizes, there are flashy rendering issues. 
     - likely has to do with that all drawCh (even hidden ones) are sent during
       render, so it IS forced to render all hidden elements, then rerender the
       upper layer, rather than just rendering the highest layer.
01. textbox horizontal bar not working when wordwrap disabled (doesn't enable
    itself)
01. horizontal scrollbar dragging not working (vertical is though) 
     - clicking still working
01. textbox  right-arrow wont get you to the last FINAL extra cursor position of the text. 
    - however the down arrow can get you there
01. refactors
     - create DynLocationSet type
     - modify EO to use DynLocationSet instead of LocationSet (everything
       dynamic on context) 
     - move 'DynLocation' to the element from the EO  
        - will need to remove pos_x, pos_y, width, height from Pane
     - move 'visible' to the element from the EO
     - remove unnecessary element event response items now that the location 
       and visibility are a part of the element

##[2302-2202] Buggy Y positioning for RCM in multipanes

When two WidgetPanes are in a MultiPane (vertpanes) and the bottom one is right
clicked, the RCM appears in the correct x position but the incorrect y position.

refer to tui/examples/issue_manager/main.rs


##[2302-2201] WidgetPane in StackPanes bleeding out of bounds

When focus of a MultiPane is shifted to a pane containing a WidgetPane, any
widgets with a size greater than that dictated by the MultiPane will bleed out
of the bounds of the multipane.

The widget should be cut off instead.


##[2302-1302] - Rename All instances of CBA in the codebase to CBA
AFFECTED FILES: Lots of them

Understanding Based Ownership is now Understanding Based Authority

##[2302-1301] - Pane Control Commands Not Registering Properly In StackPanes
AFFECTED FILES: PaneHorizontalStd.rs, PaneVerticalStd.rs

In certain situations, the pane command key combos (Ctrl+Ww, Ctrl+WW, etc) are
not being registered correctly.
Example! tui/examples/issue_manager/main.rs
Given:
- Main element is SHP
- SHP has two panes: WidgetPane & SVP. SVP contains two WidgetPanes
         SHP
          │
 ┌────────┴─────────┐
 │                  │
  ┌───────┬────────┐
  │       │        │ ─┐
  │       │  WP2   │  │
  │ WP1   │        │  │
  │       ├────────┤  ├─ SVP
  │       │  WP3   │  │
  │       │        │ ─┘
  └───────┴────────┘
- the WP1 starts in focus
- Pressing Ctrl+Ww moves to WP2
- Pressing Ctrl+WW moves to WP1
- Pressing Ctrl+Ww or Ctrl+WW will do nothing as they have been deregistered
  from the TUI.EO.Prioritizer

it seems as though, when initially moving from WP1 to WP2, the pane controls for
the SVP are not propagated up to the TUI.EO. But when moving back to WP1 from
WP2, the pane controls for the SHP (Which were never properly registered) are
properly deregistered from the TUI.EO. But since they were never initially
added, the ones being removed are actually for the SHP, which can now no longer
receive the commands, even though it still has them as SelfEvs

I suspect this issues is arising somewhere near
StackPanes.ChangeFocusToNextPane() in regards to the IC that is being returned 

it appears to be that, when the SHP changes focus from the SVP to WP1, the change in
inputability to the SVP is propagated all the way up to the TUI.EO. At that
point, while processing the IC, the TUI.EO will remove ALL instances of the pane
control combos in its prioritizer as all of them are registered to its only
child - the SHP. And since WP1 isn't registering any pane
control combos, the TUI.EO winds up with no registered pane control combos.

a potential solution would be for a ParentPane to check if any of the
InputabilityChanges removals overlap with the ParentPane's selfEvs and then send
those along as additions to be re-added after the removals have stripped all
matching events from the grandparent's prioritizer. 

This would be fairly straightforward to accomplish for StackPanes by doing it in
ChangeFocusToPane but that wouldn't solve the problem for any ParentPane.
Actually, ChangeFocusToPane is built on top of ChangePriorityForEl which is a
method of ParentPane. 

mp.ChangeFocusToPane() 
   -> mp.Focus/UnfocusPane() 
         -> pp.ChangePriorityForEl() 
               -> pp.EO.ChangePriorityForEl()

ParentPane could look through the IC returned by EO.ChangePriorityForEl, check
the RmRecEvs for matches to ParentPane.SelfEvs and update the IC.AddRecEvs
accordingly.

##[2211-2202] - Think about Adding OverallPriority to Pane
DON'T IMPLEMENT
AFFECTED FILES: pane.rs

- SEPARATE from priorities of keystrokes and commands
- wouldn't affect prioritizers
- would be useful in situation of HorizontalPanes

##[2211-2200] - Priority Panic
AFFECTED FILES: ParentPanes.rs, HorizontalPanes.rs, VerticalPanes.rs

- create parameter whereby at the start of the TUI the multipanes can determine
  their logic for dealing with 2 evs registered at the same priority. default
  could be just send to the first one. second would be panicking if two were
  registered as the same

##[2211-2201] - Standard vs Basic StackPanes
AFFECTED FILES: HorizontalPanes.rs, VerticalPanes.rs

- create StandardHorizontalPanes & StandardVerticalPanes, extended from
  HorizontalPanes & VerticalPanes, respectively.
- Standard versions should have built in logic for switching panes
- add open pane paramater - ElementID that tracks current OPEN pane
- add events for switching panes (Ctrl+W, w, j, k, etc)

##[2211-2100] - Command Restructure 
AFFECTED FILES: Many Elements

Change the the commandEl to take in the TUI EO instead of the TabsEl. 
- Eliminate the tabsElContextCreator. 

Old (na?):

The commandEl should be restructured to only call the Registered Elements
("CallerEl") through each elements natural parent ("ParentEl").

Currently the CommandEl calls the CallerEl directly. 

Goal: 
 - Get rid of the tabsElContextCreator that's currently required as the correct
   context will be available through the parent element
 - Should also solve the issue where the CallerEl needs to effect it's
   InputabilityChanges in the ParentEl
