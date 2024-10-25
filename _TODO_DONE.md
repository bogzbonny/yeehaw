

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
        // XXX this should just be loc width (post refactor of dyn_location to element)
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




01. make the element-id a name (string) which is unique across the entire cui application.
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

refer to cui/examples/issue_manager/main.rs


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
Example! cui/examples/issue_manager/main.rs
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
  from the CUI.EO.Prioritizer

it seems as though, when initially moving from WP1 to WP2, the pane controls for
the SVP are not propagated up to the CUI.EO. But when moving back to WP1 from
WP2, the pane controls for the SHP (Which were never properly registered) are
properly deregistered from the CUI.EO. But since they were never initially
added, the ones being removed are actually for the SHP, which can now no longer
receive the commands, even though it still has them as SelfEvs

I suspect this issues is arising somewhere near
StackPanes.ChangeFocusToNextPane() in regards to the IC that is being returned 

it appears to be that, when the SHP changes focus from the SVP to WP1, the change in
inputability to the SVP is propagated all the way up to the CUI.EO. At that
point, while processing the IC, the CUI.EO will remove ALL instances of the pane
control combos in its prioritizer as all of them are registered to its only
child - the SHP. And since WP1 isn't registering any pane
control combos, the CUI.EO winds up with no registered pane control combos.

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

- create parameter whereby at the start of the CUI the multipanes can determine
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

Change the the commandEl to take in the CUI EO instead of the TabsEl. 
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
