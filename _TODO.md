

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
     - create SclLocationSet type
     - modify EO to use SclLocationSet instead of LocationSet (everything
       dynamic on context) 

^^^^^^^^^^^^^^  DONE ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

01. refactors
     - move 'SclLocation' to the element from the EO  
        - will need to remove pos_x, pos_y, width, height from Pane
     - move 'visible' to the element from the EO
     - refactor the Element hashmap to just be a vec (no element id stored) 
        - effectivly moving the element ID back to the element.

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

01. create builder types for each widget. 
     - annoying to send in the ctx and hat objects each time.
     - sometimes there are fields in the widget that are only 
       used for the creation of the widget, and not for the widget itself.
     - combine hat and ctx objects for widget creation. 

