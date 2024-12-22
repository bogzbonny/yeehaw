
01. README doc about performance and drawing
     - slightly laggy in debug mode but should be good in release mode
        - due to nested element containers, more deeply nested calls 
01. hello world example in README.md
01. passing mouse movements into terminal pane
01. option to enforce that drawing which are outside of border are cropped?
     - MAYBE?? do after window generator is complete
01. execute a command on opening a tab (for the first time?) useful for showcase
    tab
01. scrollable pane which grows if above min dimension
01. better labels for elements - build into element
10. slider element, allow better dragging were you don't have to stay inside the
    element
01. terminal pane always has receivable events, even when it is unfocused (it
    should never have them) 

^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  DONE  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

01. Drawing overhaul
    - add an offset to the position gradient (so that the gradient can have an
      offset baked in without actually drawing the gradient).
    - move the time/position based gradient calculations from the organizer 
      and to the high level TUI
      - also need to set draw size (instead will always draw will big one)
         - set only once the first time 
    - now caching by element should work
   - MAYBE don't explicitly cache (and not call drawing) but still call drawing 
      each draw cycle, however each element can return special "Unchanged"
      messages which then tells the parent to use its cached value. 
          - nested containers:
            - the parent-parent-container should be able to update a sub-section 
              of the child-container, this will maybe introduce slight more
              complexity as `fn drawing` should likely be able to return
              multiple (ElementID, Vec<DrawPosCh>) chunks - that or a slightly 
              new mechanism.
   - will still need a new fn flattened_all_drawing which reads from the cache 
     and provides all DrawChPos's for the tui
   - Bugs
      - tabs - tabs will show when selected for the first time, but then they
        will never show when reselected
      - image test - the final pane is not being removed visually
      - stack test - borders are not refreshing properly 
      - window - moving the window around leaves stuff in the dust
          -  I think this is because the context fed into the window doesn't
            change but the location of the sub-elements does. SO the element
            organizer DOES need to keep a cache of the previous inputs so that
            it can update the positions of the content, even if the content
            doesn't change. - either that or force the drawing to give an update 
            even when one isn't neccesary based on the context
          -  window closing doesn't work
          - scrollable pane doesn't change positions work
      - file_nav shows nothing
      - menu test - only the final 1 menu item is ever showing 
      - nvim editor (example editor) doesn't refresh right when closed
      - textbox doesn't have keyboard events
      - listbox doesn't keyboard events
      - showcase window generated is unfocusing other widgets beyond return 
         - focuser isn't working properly it seems
            - the window-generation-zone still seems to be considered focused,
              because if I click the tabs then back again it will once again
              focus
            - the highest level vertical stack is being set to unfocused.. and
              then the lower level receives the event but is already considered
              focused so it wont send a focus event upward
      - clicking on the window doesn't bring it forward
         - broke with original code refactor, TUI draw cache never reorders
           items, need z index in there. 
            - HOWEVER z-index is only supposed to be WITHIN 1 element organizer. 
              - "bring to front" only ever looks at local z
      - terminal popup from window generator isn't receiving events.. or maybe
        not processing them
     ^DONE
   - Bugs
      - showcase train completion doesn't reset the color of the pane (should be
        black and not say exit
      - terminal inner-showcase has funny bg color on widget dial
      - terminal inner-showcase is not removing cursor in sl
      - expanding pane scrollable is not refershing properly on scrolls
         - looks like the leftmost chs are not being moved

01. textbox paste shouldn't work when non-editable

01. cleanup the widgets example 
    - showcase should just use the same widgets example pane

01. showcase example 
     - a menu
       - some hidden funny stuff on some items
     - tabs
       - terminal
       - the showcase within the showcase?? spiral 
     - window generator zone
       - dial with window pane types: basic, scrollable, scrollable-expanding, terminal, 
       - border-kind: none, basic, normal scrollbars, resizer with border-scrollbars, double width, large, tight, 
                      border-with-text, border-resizer, border-mover
       - shadow cb
       - generate button
     ^ DONE

     - TABS Zone
       - image
       - file_nav
       - gradient 
          - could have some ascii art drawings to pick from for the FG
             - a dial with a few choice ascii arts
          - toggle for FG and BG gradients
              - dial gradient kind: solid color, time-gradient, radial gradient, linear
                gradient. radial-time, linear-time
              - slider, gradient size between colors (enforce static) 
              - slider, change speed, ms for time gradients
              - slider with numbers-tb for gradient angle
              - dial, number of colors (greyed out for solid-color kind)
              - some sliders: 1 for each color (greyed out when colors
                disabled) maybe with an RGB tb for css color input
          - dial, fg ascii art (default as a butterfly)
       - widgets
         - neovim editor
         - buncha widgets which dont do much but log their results 
           in a textbox
         - TWO dials as eyes (draw a mouth label below, maybe a nose too)
           - no labels for the eyes
           - make them lock together, and change the mouth based on their 
             position
     - a big "DO NOT PRESS button" which instigates the blue screen of death

01. gif of showcase example - will need to record with non-VHS/asciicinema-tool
    to get mouse movements
     - put the gif in a seperate repo for now to keep the repo lean.

01. add landing page docs for docs.rs
     - better, doc specific intro text before the example 
     - link to examples 
     - resolve broken links 
        cargo doc --open

01. just use an image for the banner (and include the existing text as 
    markdown comment

01. add docs, crates button banners to github readme

01. make sure duf can run it

01. update rust version, update deps

01. go through and remove / or make dyn element fns named with_width with_height

^^^^^^^^^^^^^^^^^^^^^^^^ PRE-RELEASE ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

05. showcase window generator
     - the selectable pane keyboard stops working after the first window is generated.
        - maybe because window generation calles unfocus others
05. showcase bugs with keyboard event routing within recursive showcase tab

05. Theme Manager - it would be awesome to be able to just get the colors for
    each element from a provided theme manager
     - could replicate a new trait similar to Into<Color/Style> but where the
       element KIND is actually fed in. regular colors could be provided but a 
       single theme could just be passed around manually. 
     - Alternatively maybe the theme could just be added into the Context (in
       Rc<RefCell<>> like the hat). Furthermore the color could simply be taken 
       from the theme in the default 'new' functions, then special colors could
       be applied using with decorators. 
        -  note even though the theme is in a context, an element could replace
           the theme in a new context if there was going to be a different
           sub-theme in a particular grouping of sub-elements
     - Theme should use a map of of strings for the names of theme, so that its
       fully extensible and future proof. 
     - Note we could also make a new Color kind which is "Color from theme for
       "button"" for example. This way the Color could be modifed at the theme
       level (day/night switch for example) and the colors would automatically
       refresh everywhere

__________________________________________________________________________
REFACTORS


01. Improving speed for Terminal Element
    - The terminal can't really be optimized in quite the same way as the
      element; although it would be possible to take a diff on the terminal 
      and then only feed back the changed terminal-chs, we would need the
      ability to quickly replace a limited subset of chs - two options: 
 DO THIS -> (1) setup a higher level grid over the Terminal Element and update
            piecemeal on this grid using the standard Update DrawAction
            (2) create a new kind of update like Extend but which actually replaces
            individual positions if they exist... this would require storing the
            DrawChPos in either a hashmap, or iterating through it all the time,
            either option seems not super ideal. 

10. MousePossibility events: 
    - adjust mouse event logic to mirror that of the keyboard, each element
      can define what kind of mouse logic it is able to receive. 
    - MAYBE NOT - Check if this already works. this way priority can be defined
      between different types of mouse events, noteably within a scrollable
      pane, the scroll event could be routed to the scrollpane if it is not over
      a textbox element but routed to the textbox element if the the event takes
      place over the element AND the priority of the element is greater than the
      priority of the scrollpane
    - This will be useful for pixel-mode mouse logic. Most elements likely do
      not want pixel mode events

10. REMOVE/OR FIX USE OF extra locations (only used in menu currently)
    menu items-extra-locations are not routed properly for higher level external panes
     - this is because the parentpane element does not register extra-locations
       so when the menu uses extra locations beyond the scope of its parent
       pane, the menus parent pane is never routed to.
        - complex to allow for routing, because really all of these extra
          locations should also have an element-id associated with them, so that
          if there are multiple duplicate extra-locations from sub-elements for
          a parent pane, if one of the children wants to deregister, then it
          should not effect the other child also using that extra-location.
     - once fixed could change in the showcase example where the extra locations are
       registered in the highest level element and not the header element
         - // NOTE we are adding this to el instead of in the header_pane as there the extra-locations
           // are not currently propogated upward. (so this becomes a problem is the menu bar extends
           // beyond the header_pane). In the future this should be fixed.
           //el.pane.add_element(Box::new(mb));
     - Simple solution: Make the menu-item a first class element which gets
       registered with the parent pane, then have that menu item, check with its
       host menu-bar before drawing 

10. When the keyboard is matching an event combo provided to it, it should be
    recording a partial match (and a suggested maximum wait time to recheck for
    priority to this combo whether to wait the time before checking for other
    matches or to ignore the wait and to proceed attempting to match the
    character in other ways.  

10. switch to vt100_ctt or fork https://docs.rs/vt100-ctt/latest/vt100_ctt/struct.Screen.html
     - make a PR to expose the grid so that one can actually iterate the cells
     - integrate in SGR-Pixel mode into vt100

__________________________________________________________________________
FEATURES

05. Time Base Events. add a "future event" to a part of the EventResponse. In
    the future event there is a timestamp which says when this event should be
    activated. This can be triggered in the render loop and the event will then
    routed through the standard event loop as normal. This can be used to
    replicate a heartbeat for a element, or to simulate a visual effect such as
    a button click (useful for button when Enter key is hit).

05. integrate in is_dirty logic into the Pane to the drawing. if is_dirty is
    true the pane will call 'update_dirty_content', otherwise it will just
    return whatever is in the pane's content during drawing. 
     - note this will be backwards compatible, as this doesn't actually need to
       change the element interface, as all this logic can happen at the pane
       level, the default pane drawing functionality will just use an is_dirty
       check.
     - Users of the pane would then register a fn variable on the pane for 
       performing the content updates for when the pane is_dirty

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
               at the tui level. 
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

10. allow for the time gradient to execute once instead of on repeat. 

10. character content "gradients" - aka the actual characters changes with time

10. Conditional ChPlus's 
     - use cases: wire connectors, border combiners
     - draw based on what's underneith
        - could alternatively have a draw based on whats beside this character
          too... may be slighly computationally burdensome... 
        - the alternative for the border combiner. 
           - allow the border combiner to 
     - would be nice to use for both wire connectors, as well as borders 
       within borders (such as the shiftors within a stack
 

10. Hyperlink element which can open in a browser when clicked

10. hover comments
     - hover comment event which is triggered after a certain amount of time
     - TUI option to disable hover comments
     - should just be a special floaty window (with "high z" use BrintToFront)
     - destroyed on the first external event that it receives
     - All this logic should exist at the Pane level 
       - will have to refactor code such that everything now DOES call the pane
         receive event function.


__________________________________________________________________________
BUGFIXES/PATCHES


10. resizing a scrollable pane should modify the offset of that pane to account
    for the extra space (instead of automatically extending out of range)

01. dropdownlist option to deselect on enter (useful for usage in non-selection
    parent pane)

01. border pane text locations (either right, centre, left eg) should all be
    possible at the same time not ONLY one of those three options


10. scrollbar style on pane option... use a half(widthwise) block character instead of the
    thick border line... looks nice
     - maybe dont use half characters for this one only full characters
     - also maybe don't use arrows for this style
     - could provide the "inner" or "outer" option

┌─────────────────────┐
│                     │
│                     ▌
│                     ▌
│                     │
└────▀▀▀▀▀▀▀▀▀▀▀▀─────┘

10. File navigator updates 
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
    - mouse functionality

05. showcase window generator bugs
    - weirdness with underline color and alpha, sometimes makes it faded,
      sometimes not.  

05. label underline colors dont seem to work at all

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

05. Ability to use arrow keys to select menu item when menu is open


20. when drag resizing stack panes, sometimes panes which are further in the
    stack from drag location change by 1... should attempt to correct for this.

20. ArbSelector users should be able to feed in which keys are used for moving
    forward or backwards instead of enforcing left and right

20. gradient moving along the irregular line 
      - could be used to simulate a gradient border (imagine the gradient
        follows this line:         ┌──────────┐
                                   │          │
        imagine it just does a     │          │
          shimmer once...          │          │
                                   └──────────┘

__________________________________________________________________________
CONTAINERS

05. accordion stack
     - Actually this is just a minor extension of a vertical stack. 
     - need to make the Collapser Property for a whole border side and have it
       work with the corner collapser.
     - could have a static exterior dimension in which case one stack element
       would always have to be open
     - Optionally could allow for growing and shrinking its total size in which case it
       could store its size if all the elements where minimized
        - multiple stacks could be open in this situation
     - Each header should remain when the element is open 
     - optional vertical accordian stack

05. Collapse Element Wrapper... -> same as accordion stack?
     - This is just border with a special corner functionality.
     - OLD
       - should be able to collapse to a single line (vert or horiz) with custom
         text. 
       - when the element it open the collapse triangle button could just be a
         single button or an entire line
       - when an entire line is used it should be able to be draggable to effect
         the size of the element

10. vertical tabs (like brave)

10. container element: grid selector
     - like a stack except with x and y, utilizing the stacks as sub elements
       - if the middle line was to shift on a 2x2 grid it should shift for both
         the left and right sides aka they are locked.

__________________________________________________________________________
WIDGETS

10. wire-connectors
    - for visualizing routing of information between elements
    - could be directional or non-directional (aka use an arrow or not)
    - it would be cool if it could be used with a border pane WITHOUT
      actually needing to do anything special in the border pane
       - this may need new drawing logic to allow it perform conditional logic
         of the DrawCh based on the cell underneath of it
          - kind of like how transparency takes the cell underneath maybe
            the ChPlus could also have custom applications based on whats under

10. progress bar
   ██████████████                  end
   ██████████████████              end
   ▊▊▊▊▊▊▊▊▊▊▊▊                    oooo nice
    - optionally with an embedded word
    - use a gradient color! 
    - imagine that the progressbar was just a gradient changing around a box
      border

10. Emojiblast
     - https://www.emojiblast.dev/demos/basic
     - https://github.com/JoshuaKGoldberg/emoji-blast
     - needs time based events
     - could be triggered by a button for instance
     - single cell point of origin
     - explode out at first then fall down
     - reuse many of the options from the html version

10. Loading spinners
    - maybe the easiest thing would be to allow for a character changes based on
      time (like a time gradient except for the character actually displayed).
       - could have position character changes too although maybe not as useful?
    - SO many of the extended geometric shapes:
       https://symbl.cc/en/unicode/blocks/geometric-shapes-extended/
    - 🌑🌒🌓🌔🌕🌖🌗🌘
    - braile movers
    - block movers of a few varieties
      - these guy movers▁▂▃▄▅▆▇
      - https://symbl.cc/en/unicode/blocks/block-elements/
    - something with the sand timers 
    - ◐◓◑◒ △▷▽◁ ◢◥◤◣ ◥◢◣◤
    - could do larger multi-ch spinners too like a fancy sand timer with braille
      sand... maybe this is something else

20. labelled element slider bars / track bars
      - Very similar to the radio bar.. except it expands
      - label locations (before or after bar)
      - label chs on slider: start, middle, end
      - inbetween label ch
      - label positions 
      - selector
         - position (opposite-label-locations or ON)
         - selector CH
      ┳   ┳ 1
      ┃   ┃
      ╋   ╋ 2
      ┃   ┃
     ▶╋   ╋ 3
      ┃  ▶┃
      ╋   ╋ 4
      ┃   ┃
      ╋   ╋ 5                ▼
      ┃   ┃      ┣━━╋━━╋━━╋━━╋━━╋━━╋━━╋━━╋━━╋━━╋━━┫
      ╋   ╋ 6    1  2  3  4  5  6  7  8  9  1  1  1
      ┃   ┃                                 0  1  2
      ┻   ┻ 7 

20. double sided slider bar
     - start ch
     - end ch
     - inbetween ch
     - exterior ch
   ━━━━━━━━━━╋██████████████╋━━━━━━          
      ●━━━━━━━━━━━━━━━━━━━━● 

20. dragable file-like/binary-like icon object:
       ┌────┐
       │prev│
       └────┘
     my-file.txt

   - double click hook action
   - make a few different fun icons, (a scroll for text files?)

20. Prompt-Window
     - basically an old school prompt window which says some biz then gives you
       a couple options
     - Optionally it could also sieze control of the whole screen, not allowing
       you to interact with the other elements until you answer the prompt
         - could use a big transparent pane that captures all events for this
         - could "flash" the topzone of the window when the users clicks
           elsewhere than the window

10. table element
     - option to use underline instead of box drawing ch
     - autowidth or fixed width options

20. listbox over entire elements
    - abstract the listbox selector except to allow for an
      arbitrary interface.
    - THEN build a new viewer which could house an entire element within
      a "selectable item"
      - selecting that item could overlay a colored border for instance.
    - allow for elements to be listed in any fashion, possibly completely 
      independantly of the listbox logic altogether, all it would need to have 
      is a list of all the elements (which defines the order of those elements). 
    - These elements would need to be able handling 
      special events: "cursor highlight", "select", "cursor unhighlight" 
    - would need ScrollablePane Ensure Element Visible
20. ScrollablePane: Ensure Element visible. Feed in an element-id then the scrollable pane 
    should move the view to ensure that the provided element is visible.
     - useful for listbox over entire element

10. element: date selector

10. element: color selector

20. graphs and charts obviously
      - braille dots graph or 4 quadrant blocks
      - bar/column chart
      - area chart (use block 4 quadrant characters)
      - block pyramid chart
      - also build in use of 6 quadrant chs

10. 2D selector space
     - title and ticks as optional
     - use for color selector
     - could eventually provide inter-pixel values too
     - optional coloring function for bg
     - cursor obviously an option
    ┌────────────┐ 
   1┤            │
 B  ┤    x       │
 U  ┤            │
 S  ┤            │
 T 0┤            │
    └────────────┘
         BOOM        

__________________________________________________________________________
PROGRAMS

01. Snapshot TUI Tester (just call this tui-tester, binary: tuit (lol)) 
     - always multi-stage
       - record each action then take a snapshot, however don't save the
         snapshot if it's the same as the previous snapshot. 
       - Option to record with all time indices (slower to test)  
       - Option to just take a snapshot every X ms.
         - or Option to just record a screen change when it happens on its own?
     - Binary Mode or Yeehaw Mode (start with Yeehaw Mode)
     - Integrate into regular testing
     - TUI ideally we should keep everything in one window.
       - diff view (only show the differences)
       - use a toggle to switch between result/expected/diff
       - toggle to switch on and off the mouse
       - top: Button Run
       - playback: stack 
         left                                right
         scrollable pane with the actual     events playback
     - Other similar:
       - "script" standard binary
       OLD: Add another cargo project like AssertCmd for tui
            name: TuiTester?
         - https://github.com/aschey/tui-tester
         - what about https://github.com/microsoft/tui-test is this necessary?
         - open and record mouse and keystroke events
         - save only the final tui output
         - test for the final tui output being the same from
           the provided binary.
         - view what the output should look like
         - if a test is failing, but the output is correct but just changed
            there should be an option to quickly rerecord what the test should look
            like now.
         - use the .ans format (such as
           https://terminalroot.com/use-ms-paint-directly-in-terminal/) uses. 
           this format can be viewed in the terminal with "cat my_ansi_image.ans"

20. Drag and Drop TUI Application Builder (as a TUI of course)
     - basically drag and drop style element builder - with a "Code Copy" button
     - resizing of the view-pane to test TUI pages at different 
       sizes
     - preview mode where you could actually interact with all the elements
     - eventually the ability to load in code for an existing element then 
       play around with the sub-elements

20. Interactive debugging TUI application
   - use https://github.com/eclipse-iceoryx/iceoryx2 for communication?
     - or can just write to a json file

     [reload]  aspect(ddlist)
                - location
                - self-rec evs
    ┌───────┐┌────────┐┌──────────────┐┌─────────────────┐
    │events ││elements││ element-parts││ OUTPUT          │ 
    │       ││        ││              ││                 │
    │       ││        ││              ││                 │
    │       ││        ││              ││                 │
    │       ││        ││              ││                 │
    │       ││        ││              ││                 │
    └───────┘└────────┘└──────────────┘└─────────────────┘
    [add another group]

__________________________________________________________________________
LOW-PRIORITY

30. provide a sync version of TUI 

30. gradients on angles: get the actual aspect ratio from the terminal and integrate it in. 

30. :Command pane and functionality
      - use custom event routing system

30. tabs with border, LOW PRIORITY
╭─────╮────╮────╮──────╮  ╭─────╭────╮────╮──────╮ ╭─────╭────╭────╮──────╮
│tabs1│tab2│tab3│ tab4 │  │tabs1│tab2│tab3│ tab4 │ │tabs1│tab2│tab3│ tab4 │
│     └────────────────╮  ├─────┘    └────┴──────┤ ├─────┴────┘    └──────┤
│                      │  │                      │ │                      │
│                      │  │                      │ │                      │
│                      │  │                      │ │                      │
│                      │  │                      │ │                      │
└──────────────────────┘  └──────────────────────┘ └──────────────────────┘

40. custom mouse types using images (requires image support, and mouse pixel tracking) 

40. volume bar (color in)  
   ▁▂▃▄▅▆▇
    - drag up/right to increase down/left to decrease 
    - model this after the slider widget... pretty similar

40. tui get the color under the cursor pixel - useful for color pickers or from actual image pallets

50. LOW PRIORITY CAN JUST USE $EDITOR. element: vim-style textbox
     - with two scrollbars the mode can be placed in 
       the decorations corner!
_______________________________________________________________________
WIMP reqd features

10. color-pallet element
10. TGIF
30. tui get the final color under the cursor (more than just what's in the
    element, get the full final output with alpha's applied)
05. tui export visual area to either DynamicImage, .png, (optionally or .ans)
      - useful for WIMP

_______________________________________________________________________
ZELT reqd features
10. terminal_editor - autoexpanding based on text size in buffer (like zell
    editing) 
      - need to provide configuration arguments by editor type
      - I think maybe it would work if we use the no-buffer option in neovim.
      - need for zelt editing the buffer directly `set autoread`
      - set swp file location manually so can access the swp files to read them
         - nvim -c 'set directory=~/my_swap_files//' your_file.txt
      - EASY
         - keep expanding the ctx until the upper left box (prev context)
           doesn't change. (ignoreing bottom 2 or 3 lines)
    
