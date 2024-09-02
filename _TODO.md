
01. write debug_pane

^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  DONE  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

01. rewrite horizontal/vertical stack panes
01. translate tabs NOTE do this after stack so the tabs can be a stack
     - just use buttons as the tabs?!
01. translate scrollable pane (or work into the standard pane? as an option) 
01. translate file_navigator

05. Time Base Events. add a "future event" to a part of the EventResponse. In
    the future event there is a timestamp which says when this event should be
    activated. This can be triggered in the render loop and the event will then
    routed through the standard event loop as normal. This can be used to
    replicate a heartbeat for a element, or to simulate a visual effect such as
    a button click.

05. create builder types for each widget. 
     - annoying to send in the ctx and hat objects each time.
     - sometimes there are fields in the widget that are only 
       used for the creation of the widget, and not for the widget itself.
     - combine hat and ctx objects for widget creation. 

05. borders-pane wrapper
     - option for each of up/down/right/left
     - custom up/down/right/left DrawChs AND corner pieces
     - single/double/bold lines
     - built in scrollbar
     - drag-resizing - drag while on the edge to resize the location

10. gradient colour types, don't ask me how exactly however this is basically
    what we should do.
     - refactor colour to make a call each draw
       - maybe make colour an enum for serialization purposes. 
     - maybe the gradient moves based on the screen position.
     - keep it linear gradients for now
     - gradient params: pos-offset x/y (as SclVal!), change-rate x/y,
        grad-colours and positions(SclVal?!) (need multiple positions for rainbows). 

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

30. refactor: remove ExtraLocations from EventResponse

05. Remove Refresh logic from Elements. currently when an element is destroyed
    or replaced, the parents call some Refresh logic, this should be removed in
    favour of specifically removing the priorities by the element id of the
    element being destroyed or replaced

05. Proper overwrite when writting a transparent character. Build in
    functionality to retrieve and draw what the content underneath should be
    even if it's not currently drawn will require new fn on Element
    "GetDrawingAtPos" as well as determining the layer order at a given
    position.

10. When the keyboard is matching an event combo provided to it, it should be
    recording a partial match (and a suggested maximum wait time to recheck for
    this match) whereby the caller can then make a choice given the associated
    priority to this combo whether to wait the time before checking for other
    matches or to ignore the wait and to proceed attempting to match the
    character in other ways.  

10. File navigator
    - ability to hide dotfiles ("ex. .git") navigator (toggle this functionality
      with Shift-i) 
    - scroll when the expansion exceeds element size (this logic is already in
     standard pane just needs to be hooked up)
    - save sub-folder expansion when a parent folder closes and then reopens. 
      - the folder keeps records of its navItems once they've been populated.
         - would need to "refresh" this list with each open could cause
           problems.
    - fix the up-dir (..) button 

