

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

^^^^^^^^^^^^^^  DONE ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


01. textbox horizontal bar not working when wordwrap disabled (doesn't enable
    itself)
