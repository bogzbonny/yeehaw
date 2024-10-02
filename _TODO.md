
hl. tab key not working to go between widgets in pane_scrollable_test (nor
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

^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  DONE  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


05. button selectable (can hit with enter key)

05. color "darker", "lighter" methods

05. table widget

01. basic file viewer 

05. scrollbar shouldn't move if uninitialized and a drag mouse enters it

01. translate file_navigator

05. use .flf (figlet font) format instead of custom megatext
     - https://docs.rs/figlet-rs/latest/figlet_rs/

01. support taffy as a layout structure.
     - Taffy low-level API (MUST use master branch)
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

05. Time Base Events. add a "future event" to a part of the EventResponse. In
    the future event there is a timestamp which says when this event should be
    activated. This can be triggered in the render loop and the event will then
    routed through the standard event loop as normal. This can be used to
    replicate a heartbeat for a element, or to simulate a visual effect such as
    a button click.

10. cui export visual area to either DynamicImage, .png, or .ans
      - useful for WIMP

10. gradient color types, don't ask me how exactly however this is basically
    what we should do.
     - refactor color to make a call each draw
       - maybe make color an enum for serialization purposes. 
     - maybe the gradient moves based on the **LOCAL** screen position. (aka
       position within the parent)
     - only linear gradient, but can simulate other functions with different
       position colors
     - gradient params: pos-offset x/y (as DynVal!), 
        grad-colors and positions(DynVal?!) (need multiple positions for rainbows). 
     - after the final position is reached (and before the final position if
       there is an offset) repeat the pattern ... would need the "final length"
       (aka what is the gradient inbetween the final color and the first color)
   - Color will need a "blend" function with another color for the gradients
       blended = color1.blend(percent, color2)
   - Time gradient
     - should pass in the time with the draw context
   - time and screen position gradient.
      - maybe this could be a time gradient, however each color on the gradient
        scale WAS another gradient. (or vice versa) 

20. color-pallet widget

20. cui get the color under the cursor - useful for color pickers or image pallets

05. Command functionality

05. Collapse Element Wrapper... 
      - should be able to collapse to a single line (vert or horiz) with custom
        text. 
      - when the element it open the collapse triangle button could just be a
        single button or an entire line
      - when an entire line is used it should be able to be draggable to effect
        the size of the element

05. Subscription based events on common objects. 
     - like leptos. any element could subscribe to an object (with any other
       element can change). When that object changes it would send out events to
       any other elements which subscribed to it... OR maybe it would just make
       sense to use hooks this way you don't need all the parents of the
       destination to also subscribe to the hook. USE HOOKS!

05. create builder types for each widget. 
     - annoying to send in the ctx and hat objects each time.
     - sometimes there are fields in the widget that are only 
       used for the creation of the widget, and not for the widget itself.
     - combine hat and ctx objects for widget creation. 

05. Into<Widgets> Trait which can be applied to each widget builder so that
    .to_widgets() doesn't need to be manually called during construction
    (applied in add_widget).

05. borders-pane wrapper
     - option for each of up/down/right/left
     - custom up/down/right/left DrawChs AND corner pieces
     - single/double/bold lines
     - built in scrollbar
     - drag-resizing - drag while on the edge to resize the location

05. Remove Refresh logic from Elements. currently when an element is destroyed
    or replaced, the parents call some Refresh logic, this should be removed in
    favour of specifically removing the priorities by the element id of the
    element being destroyed or replaced
     - is this still an issue?

10. introduce errors, remove all unwraps

10. vertical tabs (like brave)

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
    - it would be cool to integrate in better capture event logic too, if the
      mouse event is NOT captured, send the event to the next priority down. 
       - this could potentially also be applied to the accomplish the scroll
         situation as described above.. first send the event to the inner pane,
         then if the mouse scroll event is not captured then send it to the
         scrollable pane.

10. integrate in trait upcasting for Widget Type once available (remove
    el_widget_pane drawing functionality in favour of the parent pane draw).
    https://github.com/rust-lang/rust/issues/65991


30. figure out a nicer way of inheriting element functions from the above
    element besides lots of boilerplate, probably though the use of a macro

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

40. jexer custom mouse types (requires sixel, and mouse pixel tracking) 
