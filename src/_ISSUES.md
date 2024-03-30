<!---
This issues file is used to document issues within this fold. 
 - To close an issue move the issue to a subheader of the CLOSED ISSUES section
 - To open an issue: 
   - calculate a unique issue number from today's date:
      YYMM-DDff
      - where YY is rightmost digits of the year (2007=07)
      -       MM is the month
      -       DD is the month's day 
      -       ff is your favourite number between 00-99
      - ensure that your issue number is unique, if it is not, get a new favourite
        number
   - add the your issue to the OPEN ISSUES section with the following layout:

# [YOUR-ISSUE-NO.] - Your Issue Title
Your issue description... 
---> 

# OPEN ISSUES

##[2304-2404] Close right click menus on resize
Right click menus should be closed on resize

##[2302-2203] Optimize For Elements Not Requiring Content Updates
AFFECTED FILES: cui.go

Regarding the Render() func

Could optimize for elements not requiring updates by sending in a timestamp.
Would then need a force-render option (for startup)

Note: duffy found this TODO and isn't quite sure what it means. It is regarding
the issue of the constant drawing updates that the CUI is constantly asking
every element in the tree to provide to it. It may no longer be relevant since
the conception of the "Heartbeat" with an opt-in system...

##[2302-1303] - Update ElementIDs To Be Unique and Useful 
Assigned To: Bogz

ElementIDs are currently integers reflecting the order of addition of an element
to a parent organizer. These are non-unique ids that are repeated throughout a
CUI application. This gives them very limited use in debugging.
They should be changed to unique identifiers that, ideally, can be customized
for added usefulness and good times.

##[2211-2401] - Remove Refresh logic from Elements
currently when an element is destroyed or replaced, the parents call some
Refresh logic, this should be removed in favour of specifically removing the
priorities by the element id of the element being destroyed or replaced


##[2211-2401] - Buggy Resizing For Vertical/HorizontalPanes
AFFECTED FILES: HorizontalPanes.go, VerticalPanes.go

Ratios aren't being maintained properly
  - when growing, top vert pane stays the same size, but bottom pane grows
  - when growing, left hor loses size, right hor gains size


##[2211-0700] - Handling Resizes
AFFECTED FILES: All Elements

Currently (commit 68165ef), ResizeEvent() has been added to the element
interface. When a resize event is detected by the CUI, ResizeEvent() is called
on the main element. If this is a standard pane, nothing will happen. If this is
a VerticalPanes: 
vp.PSizes.Adjust(ctx.S, vp.EO)
is called and:
ResizeEvent(ctx Context) 
is called on every child element.

This was introduced to replace the previous logic that had 	vp.PSizes.Adjust(ctx.S, vp.EO)
being called every time vp.Drawing was called which was every millisecond. This
was needless computation.

This solution might be fine but there also might be a better way to fix it by
going deeper into how VerticalPanes are handled. Not sure but this solution was
simpler.
I feel like it's cumbersome to have to manually handle resize events inside of
an element instead of just automatically doing it. Somehow.



## [2206-2400] - Separate event loop from drawing functions
AFFECTED FILES: cui.go

Bugs might occur if event loop calls drawing functions. Although, my grasp on
tcell processes is tenuous and the current setup might be fine since only
updates are happening everytime an event occurs

## [2206-1000] - Proper overwrite when writting a transparent character 
EFFECTED FILES: `0__cui.go`, `1a_element.go`

Build in functionality to retrieve and draw what the content underneath
should be even if it's not currently drawn will require new fn on Element
"GetDrawingAtPos" as well as determining the layer order at a given position.


## [2206-1001] - Proper overwrite when writting a transparent character 
EFFECTED FILES: `1b_keyboard.go`

When the keyboard is matching an event combo provided to it, it should be
recording a partial match (and a suggested maximum wait time to recheck for this
match) whereby the caller can then make a choice given the associated priority
to this combo whether to wait the time before checking for other matches or to
ignore the wait and to proceed attempting to match the character in other ways.  


## [2206-1100] - Time base element Events
EFFECTED FILES: `element.go`

Each element should be able to modify its content based on time rather than a
keyboard stroke of cursor event. The recommended approach is to allow each
element to register a heartbeat function with its parent, which the main thread
would then process as a time-based heartbeat event each beat.

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

# CLOSED ISSUES

##[2302-2202] Buggy Y positioning for RCM in multipanes

When two WidgetPanes are in a MultiPane (vertpanes) and the bottom one is right
clicked, the RCM appears in the correct x position but the incorrect y position.

refer to cui/examples/issue_manager/main.go


##[2302-2201] WidgetPane in StackPanes bleeding out of bounds

When focus of a MultiPane is shifted to a pane containing a WidgetPane, any
widgets with a size greater than that dictated by the MultiPane will bleed out
of the bounds of the multipane.

The widget should be cut off instead.


##[2302-1302] - Rename All instances of CBA in the codebase to CBA
AFFECTED FILES: Lots of them

Understanding Based Ownership is now Understanding Based Authority

##[2302-1301] - Pane Control Commands Not Registering Properly In StackPanes
AFFECTED FILES: PaneHorizontalStd.go, PaneVerticalStd.go

In certain situations, the pane command key combos (Ctrl+Ww, Ctrl+WW, etc) are
not being registered correctly.
Example! cui/examples/issue_manager/main.go
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

##[2211-2202] - Think about Adding OverallPriority to StandardPane
DON'T IMPLEMENT
AFFECTED FILES: StandardPane.go

- SEPARATE from priorities of keystrokes and commands
- wouldn't affect prioritizers
- would be useful in situation of HorizontalPanes

##[2211-2200] - Priority Panic
AFFECTED FILES: ParentPanes.go, HorizontalPanes.go, VerticalPanes.go

- create parameter whereby at the start of the CUI the multipanes can determine
  their logic for dealing with 2 evs registered at the same priority. default
  could be just send to the first one. second would be panicking if two were
  registered as the same

##[2211-2201] - Standard vs Basic StackPanes
AFFECTED FILES: HorizontalPanes.go, VerticalPanes.go

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


