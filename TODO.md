
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  DONE  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

WIDGET RECALL
 - deleted ElWidgetPane for a new type ParentPaneWithSelectibility
    - basically the parentpane with tab capturing
    - using the ParentPane store to hold index of the currently selected widget 
    - would just need to specify each element as either "selectible" or "not"
      when it gets added to the widget macro
 - move Widgets -> Elements (basically only used for labels)
 - Remove "Unselectable type" from selectibility which will just be the regular element
 - Test if each trait has has selectibility by use of the attributes
 - Selectibility Hook
    - this one is a bit harder maybe we have to introduce hooks on 
      arbitrary attribute setting. The existing hook structure should
      be able to hold hooks of this variety 
    - could maybe do something funny where the hook is actually set to 
      the name of the attribute KEY plus a prefix! thus allowing the hook
      system to specify the exact type of the hook.

05. create builder types for each widget. 
     - sometimes there are fields in the widget that are only 
       used for the creation of the widget, and not for the widget itself.

05. Into<Widgets> Trait which can be applied to each widget builder so that
    .to_widgets() doesn't need to be manually called during construction
    (applied in add_widget)

_____________


10. integrate in trait upcasting for Widget Type once available (remove
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

01. make crate into a workspace

01. Snapshot tui Tester (just call this tui-tester, binary: tuit (lol)) 
     - always multi-stage
       - record each action then take a snapshot, however don't save the
         snapshot if it's the same as the previous snapshot. 
       - Option to record with all time indices (slower to test)  
       - Option to just take a snapshot every X ms.
         - or Option to just record a screen change when it happens on its own?
     - Binary Mode or Yeehaw Mode (start with Yeehaw Mode)
     - Integrate into regular testing
     - TUI ideally we should keep everything in one window.
       -  
       - diff view (only show the differences)
       - use a toggle to switch between result/expected/diff
       - toggle to switch on and off the mouse
       - top: Button Run
       - playback: stack 
         left                                right
         scrollable pane with the actual     events playback
     - Other similar:
       - "script" standard binary
       - microsoft has something

01. support taffy as a layout structure.
     - Taffy low-level API (0.6.0 fixes ownership issues I was facing)
     - I THINK it only makes sense to only use taffy optionally within an
       element and keep using the Dynamic-Location. There is a lot of weird
       stuff that enforcing taffy globally makes us do. 
        - [DONE] If we are to integrate in to a new Location type which can be either
          DynLocationSet or TafLocation then we would need to somehow either
          remove the Cache on Clone or have the Cache be in a Rc<RefCel<>>
          (wierd) 
        - [DONE] first would need to refactor ZIndex to work in opposite order
          of current workings
        - Integrate TafLocation as an Option on Pane? ParentPane?
          - Have the TafLocation Simply change the DynLocation (fixed) every
            time it changes or there is a resize.
          - will need to deal with the Taffy NodeId type now... could probably
            just tack on an u64 absolute type to the ElementId type
              - either that or have to store a vec of all the elementIds
              - OR could just sort all the ElementIds per Organizer and call
                that the vec... not the most efficient but easy enough to tack
                ontop without adding more crap to the organizer
     - model after the partial owned model https://github.com/DioxusLabs/taffy/blob/main/examples/custom_tree_owned_partial.rs
     OLD NOTES
       - WONT DO use the taffy high level API
          - taffy recompute logic to take place in the drawing function
          - track the last size, only recompute the taffy tree on size changes
            OR taffy style change event (create a new EventResponse Type)
             - this response type would effect a "dirty" flag which would exist
               at the cui level. 
          - each element would have a taffy type which is taffy::Style and
            taffy::Layout. The layout would get recomputed and set after each
            taffy tree computation. 
          - each element organizer would need to have a helper function for
            helping to compute the taffy tree such that it could be called into
            to add leaf nodes ect. 
        - WONT DO would need to mimic some form of the "plus" function for the Taffy Style.  
          - IMPOSSIBLE needs things to just be wrapped in further containers
          - this becomes annoying for things like grouped widgets (textbox with
            scrollbar... OR anything with labels). which will then need a wrapped
            into a parent pane and have the events propogated downward. 
             - the grouping of widgets would then need to fulfill the Widget
               interface and act like one.
        - WONT DO How would this even work for something like a menu?
          - menu bar has a position (arbitrary). 
          - next menu expansion would need to have a position of that original 
            arbitrary position + some offset
             - could make the whole menu a parent pane, BUT then it would
               introduce a bunch of empty transparent space which would be awkward
               to then propogate the events downward from.
             - Maybe could work if there was a "flatten" the tree for locations
               - each sub-item would be a leaf of the menu-bar however a final
                 location would be flattened down such that it was not a sub-item
                 of the menu-bar but of the same parent the menu-bar has 

05. STACKING of minimized windows in parent pane
    MAYBE do only after taffy is integrated... could help for these guys
      - minimized guys should stack in the parent pane (ex. to the right)
         - this should be easy for adding new panes on top
         - the difficulty comes if you maximize any early minimized window. The
           righthand minimized windows would need to be shifted to the right.
            KEY: MINIMIZED_ELEMENTS
            Values: 
               orientation: Enum (left or right)
               els: Vec<(ElementID, pos)> 
            - with all this information, each time a window maximized or
              minimized itself it should be able to reorient the locations 
              of the other panes if need be.
    reorient minimized window in the parent when there is a resize. 
     - for instance if the minimized windows used the entire bottom 
       of the parent pane, then those elements should be shuffled to the higher
       row if the parent pane is resized smaller.
     - this will be technically a bit complex maybe, I imagine we 
       need a special hook for moving around minimized panes. Alternatively 
       we could just hard code in special logic if the MINIMIZED_ELEMENTS key
       exists in the store. 
        - maybe we could allow other elements to register hooks on event kinds.
           - shouldn't be that complex. Each window could just ensure that the 
           parent pane has this hook registered for resized on each
           minimization.

05. borders-pane wrapper
     - option for each of up/down/right/left
        - aka. doesn't need to be fully bordered.
     - custom up/down/right/left DrawChs AND corner pieces
     - single/double/bold lines defaults 
     - built in scrollbars optionally
     - drag-resizing - drag while on the edge to resize the location

05. accordion stack
     - could have a static exterior dimension in which case one stack element
       would always have to be open
     - Optionally could allow for growing and shrinking its total size in which case it
       could store its size if all the elements where minimized
     - Each header should remain when the element is open 
     - optional vertical accordian stack

05. Time Base Events. add a "future event" to a part of the EventResponse. In
    the future event there is a timestamp which says when this event should be
    activated. This can be triggered in the render loop and the event will then
    routed through the standard event loop as normal. This can be used to
    replicate a heartbeat for a element, or to simulate a visual effect such as
    a button click (useful for button when Enter key is hit).

05. I think that in widget_test the textarea is passing in a width that is 1 to
    small! - once you move the cursor the scrollbar changes size ever so
    slightly

10. cui export visual area to either DynamicImage, .png, or .ans
      - useful for WIMP

20. color-pallet widget

20. cui get the color under the cursor - useful for color pickers or from actual image pallets

05. Command functionality

20. table widget

05. Collapse Element Wrapper... 
      - should be able to collapse to a single line (vert or horiz) with custom
        text. 
      - when the element it open the collapse triangle button could just be a
        single button or an entire line
      - when an entire line is used it should be able to be draggable to effect
        the size of the element


10. introduce errors, remove all unwraps

10. vertical tabs (like brave)

10. widget slider bars / track bars
   ██████████████████━━━━━━━━━━━━━━   ┳   ┳ 1
                                      ┃   ┃
   ━━━━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━   ╋   ╋ 2
                                      ┃   ┃
   ██████████╋━━━━━━━━━━━━━━━━━━━━━  ▶╋   ╋ 3
                                      ┃  ▶┃
   ━━━━━━━━━━╋██████████████╋━━━━━━   ╋   ╋ 4
                                      ┃   ┃
   ══════════╪═════════════════════   ╋   ╋ 5
                                      ┃   ┃
   ══════════╪██████████████╪══════   ╋   ╋ 6 
                                      ┃   ┃
   ══════════╪▓▓▓▓▓▓▓▓▓▓▓▓▓▓╪══════   ┻   ┻ 7 

   ━━━━━━━━━● 
      ●━━━━━━━━━━━━━━━━━━━━● 
   ━━━━━━━━❍─────────────────────── 
   ▁▂▃▄▅▆▇


10. Dial
    - if there are labels could bold the one which is selected
    - could provide continious value if pixel mode enabled
   __    __    __    __    __    __    __    __ 
  ╱° ╲  ╱ °╲  ╱  ⚬  ╱  ╲  ╱  ╲  ╱  ╲  ╱  ╲  ⚬  ╲
  ╲__╱  ╲__╱  ╲__╱  ╲__°  ╲_⚬╱  ╲⚬_╱  °__╱  ╲__╱  
                   
One letter labels
   A__B      A__B       A__B       A__B 
 H ╱° ╲ C  H ╱  ⚬ C   H ╱  ╲ C   H ╱° ╲ C
 G ╲__╱ D  G ╲__╱ D   G ╲__° D   G ╲__╱ D              
   F  E      F  E       F  E       F  E  

           OptionH __ OptionA
         OptionG  ╱° ╲  OptionB
         OptionF  ╲__╱  OptionC
           OptionE    OptionD

           OptionH  OptionA
                ⟍ __ ⟋ 
       OptionG - ╱  ╲ - OptionB
       OptionF - °__╱ - OptionC
                ⟋    ⟍ 
           OptionE  OptionD

            OptionH  OptionA
         OptionK.⟍ __ ⟋ .OptionB
       OptionJ -  ╱  ╲  - OptionC
       OptionI -  ⚬__╱  - OptionD    // can use lower then upper dots on these positions
         OptionH´⟋    ⟍ `OptionE
            OptionG  OptionF

            OptionH  OptionA
         OptionK   __    OptionB
       OptionJ    ╱  ╲    OptionC
       OptionI    °__╱    OptionD
         OptionH         OptionE
            OptionG  OptionF

10. progress bar
    - optionally with an embedded word
    - use a gradient color! 
    - imagine that the progressbar was just a gradient changing around a box
      border

10. Loading fidgets spinners
    - 🌑🌒🌓🌔🌕🌖🌗🌘
    - braile movers
    - block movers of a few varieties
      - these guy movers▁▂▃▄▅▆▇
      - https://symbl.cc/en/unicode/blocks/block-elements/
    - something with the sand timers 
    - ◐◓◑◒
    - △▷▽◁
    - ◢◥◤◣
    - ◥◢◣◤

10. feature: hover comments

10. widget: date selector

10. widget: color selector
10. widget: table (see ratatui)
10. button: visualize button being clicked
10. TGIF

10. When the keyboard is matching an event combo provided to it, it should be
    recording a partial match (and a suggested maximum wait time to recheck for
    priority to this combo whether to wait the time before checking for other
    matches or to ignore the wait and to proceed attempting to match the
    character in other ways.  

10. File navigator
    - ability to hide dotfiles ("ex. .git") navigator (toggle this functionality
      with Shift-i) 
    - scroll when the expansion exceeds element size (this logic is already in
     standard pane just needs to be hooked up)
    - save sub-folder expansion when a parent folder closes and then reopens. 
    this match) whereby the caller can then make a choice given the associated
      - the folder keeps records of its navItems once they've been populated.
         - would need to "refresh" this list with each open could cause
           problems.
    - fix the up-dir (..) button 

10. MousePossibility events: 
    - adjust mouse event logic to mirror that of the keyboard, each element
      can define what kind of mouse logic it is able to receive. 
    - this way priority can be defined between different types of mouse events,
      noteably within a scrollable pane, the scroll event could be routed to the 
      scrollpane if it is not over a textbox widget but routed to the textbox
      widget if the the event takes place over the widget AND the priority of
      the widget is greater than the priority of the scrollpane
    - integrate in better capture event logic, if the mouse event is NOT
      captured, send the event to the next priority down. [DONE]
       - this could potentially also be applied to the accomplish the scroll
         situation as described above.. first send the event to the inner pane,
         then if the mouse scroll event is not captured then send it to the
         scrollable pane. - maybe then wouldn't need the mouse event logic??

20. Add another cargo repo like AssertCmd for tui
     name: TuiTester?
     - https://github.com/aschey/tui-tester
     - what about https://github.com/microsoft/tui-test is this necessary?
     - open and record mouse and keystroke events
     - save only the final cui output
     - test for the final cui output being the same from
       the provided binary.
     - view what the output should look like
     - if a test is failing, but the output is correct but just changed
        there should be an option to quickly rerecord what the test should look
        like now.
     - use the .ans format (such as
       https://terminalroot.com/use-ms-paint-directly-in-terminal/) uses. 
       this format can be viewed in the terminal with "cat my_ansi_image.ans"

30. irregular gradient lines
    - OUTWARD
      - a gradient moving outward from an irregular set of coordinates (making a
        line
      - basically just a bunch of radial point gradients however when they
        interact the lowest gradient position should just be used (as opposed to
        a blend)
    - ALONG 
      - gradient moving along the irregular line 
      - could be used to simulate a gradient border (imagine the gradient
        follows this line:         ┌──────────┐
                                   │          │
                                   │          │
                                   │          │
                                   └──────────┘

30. gradients on angles: get the actual aspect ratio from the terminal and integrate it in. 

40. jexer custom mouse types (requires image support, and mouse pixel tracking) 

40. Scrollbar bug: when dragging scrollbar with mouse, will drag good for a bit
    then close to the end it just moves all the way to the maximum

40. Subscription based events on common objects. 
     - like leptos. any element could subscribe to an object (with any other
       element can change). When that object changes it would send out events to
       any other elements which subscribed to it... OR maybe it would just make
       sense to use hooks this way you don't need all the parents of the
       destination to also subscribe to the hook. USE HOOKS!
       - Actually could be really easy with the Event Router - could use Custom
         Event
       - question is: what events should actually be broadcast?

50. LOW PRIORITY CAN JUST USE $EDITOR widget: vim-style textbox
     - with two scrollbars the mode can be placed in 
       the decorations area!
