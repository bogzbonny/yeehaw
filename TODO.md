
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

^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  DONE  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

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

01. scrolling tb while deselected doesn't scroll the linenumbers or the
    scrollbars

01. window test, all the main buttons are staying highlighted after they're clicked
          (easy fix) 

01. resizing a scrollable pane should modify the offset of that pane to account
    for the extra space (instead of automatically extending out of range)


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
DONE ^^^^^^

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
widget_textbox.rs          | 
widget_textbox_numbers.rs  | 
widget_toggle.rs           | DONE

05. introduce errors, remove all unwraps

01. docs docs docs

01. gifs gifs gifs

01. add docs, crates button banners to github readme

01. showcase example 
      - sweet figlet text up top "Yeahaw showcase"
      - dial with window types 
        - terminal as one of them
      - window generator button
      - a menu
         - some hidden funny stuff on some items
         - tic tac toe
      - a split stack with some tabs
        and funky gradient examples
         - maybe some dials which can change the
           gradient colors / angle / size
      - a big "DO NOT PRESS button" 
        which instigates the blue screen of death
      - neovim editor
      - buncha widgets which dont do much but log their results 
        in a textbox

01. Make the context size more clear.
     - The context provided always contains the size of the element. However
       during initialization, before the size of an element is known, the
       context fed in will be the parent context. This is confusing as heck and
       I hope to rectify this, possibly by providing both the parent-element
       size and size-for-the-element as options within the context. 
     - parent size should not be an option, only the child size should be an
       option. parent size must always be known.
       - calling pane.width(ctx) should be the same as ctx.child.width if the child
         size is provided

^^^^^^^^^^^^^^^^^^^^^^^^ PRE-RELEASE ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
01. running "top" inside terminal window shoots the cursor outside of the
    window! should correct for this

05. integrate in is_dirty logic into the Pane to the drawing. if is_dirty is
    true the pane will call 'update_dirty_content', otherwise it will just
    return whatever is in the pane's content during drawing. 
     - note this will be backwards compatible, as this doesn't actually need to
       change the element interface, as all this logic can happen at the pane
       level, the default pane drawing functionality will just use an is_dirty
       check.
     - Users of the pane would then register a fn variable on the pane for 
       performing the content updates for when the pane is_dirty

01. border pane text locations (either right, centre, left eg) should all be
    possible at the same time not ONLY one of those three options

01. dropdownlist option to deselect on enter (useful for usage in non-selection
    parent pane)

01. terminal_editor - get the no-editor elements hooked up.
01. terminal_editor - autoexpanding based on text size in buffer (like zell
    editing) 
      - need to provide configuration arguments by editor type
      - I think maybe it would work if we use the no-buffer option in neovim.
      - need for zelt editing the buffer directly `set autoread`
      - set swp file location manually so can access the swp files to read them
         - nvim -c 'set directory=~/my_swap_files//' your_file.txt

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

10. scrollbar style on pane option... use a half(widthwise) block character instead of the
    thick border line... looks nice

┌─────────────────────┐
│                     │
│                     │
│                     ▖
│                     ▌
│                     ▌
│                     ▌
│                     ▘
│                     │
└◁───▀▀▀▀▀▀▀▀▀▀▀▘────▷┘

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

05. Time Base Events. add a "future event" to a part of the EventResponse. In
    the future event there is a timestamp which says when this event should be
    activated. This can be triggered in the render loop and the event will then
    routed through the standard event loop as normal. This can be used to
    replicate a heartbeat for a element, or to simulate a visual effect such as
    a button click (useful for button when Enter key is hit).

05. tui export visual area to either DynamicImage, .png, (optionally or .ans)
      - useful for WIMP

05. accordion stack
     - could have a static exterior dimension in which case one stack element
       would always have to be open
     - Optionally could allow for growing and shrinking its total size in which case it
       could store its size if all the elements where minimized
        - multiple stacks could be open in this situation
     - Each header should remain when the element is open 
     - optional vertical accordian stack

05. Collapse Element Wrapper... -> same as accordion stack?
      - should be able to collapse to a single line (vert or horiz) with custom
        text. 
      - when the element it open the collapse triangle button could just be a
        single button or an entire line
      - when an entire line is used it should be able to be draggable to effect
        the size of the element

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


10. Dial 8 or 12 positions
    - if there are labels could bold the one which is selected
    - could provide continious value if pixel mode enabled

               op                op                op                op 
   __    __    __    __    __    __    __    __    __    __    __    __ 
  ╱° ╲  ╱ °╲  ╱  °  ╱  ⚬  ╱  ╲  ╱  ╲  ╱  ╲  ╱  ╲  ╱  ╲  ╱  ╲  ⚬  ╲  °  ╲
  ╲__╱  ╲__╱  ╲__╱  ╲__╱  ╲__°  ╲__⚬  ╲_⚬╱  ╲⚬_╱  ⚬__╱  °__╱  ╲__╱  ╲__╱  
                   
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

            OptionL  OptionA
         OptionK.⟍ __ ⟋.OptionB
       OptionJ -  ╱  ╲  - OptionC
       OptionI -  ⚬__╱  - OptionD    // can use lower then upper dots on these positions
         OptionH´⟋    ⟍`OptionE
            OptionG  OptionF      

            OptionL  OptionA
         OptionK   __   OptionB
       OptionJ    ╱  ╲    OptionC
       OptionI    °__╱    OptionD
         OptionH        OptionE
            OptionG  OptionF

It'd be cool to come up with a "Complex Selector" generalization for the dials. 
 - for now, this complex selector should probably be the only way to initialize
   a dial... eventually we could automatically produce the maps, but it can get
   complicated as the text of the different options changes.
 - probably each version of the dial (3 postion... 8 position etc) should be a
   different complex selector. 
 - All the different states could be fed in manually as DrawCh2Ds 
 - we would probably want different states for "selecting" (brighter colors) and
   "selected" dimmer colors. 
 - Feed in a map of all the different selection positions:

      OptionL  OptionA          KKKLLLLLLLLLLAAAAAAAAAABBB
   OptionK.⟍ __ ⟋.OptionB       JKKKKKKKKKLLLAAABBBBBBBBBC
 OptionJ -  ╱  ╲  - OptionC     JJJJJJJJJJJJJCCCCCCCCCCCCC
 OptionI -  ⚬__╱  - OptionD     IIIIIIIIIIIIIDDDDDDDDDDDDD 
   OptionH´⟋    ⟍`OptionE       IHHHHHHHHHGGGFFFEEEEEEEEED
      OptionG  OptionF          HHHGGGGGGGGGGFFFFFFFFFFEEE 

 - if certain positions are excluded their selection positions could be a '0'
 - If the mouse is dragging outside of the selector zone, the nearest position
   could be "snapped to".

10. progress bar
    - optionally with an embedded word
    - use a gradient color! 
    - imagine that the progressbar was just a gradient changing around a box
      border

10. allow for the time gradient to execute once instead of on repeat. 

10. character content "gradients" - aka the characters change 

10. Loading spinners
    - maybe the easiest thing would be to allow for a character changes based on
      time (like a time gradient except for the character actually displayed).
       - could have position character changes too although maybe not as useful?
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


10. dragable file-like icon object:
       ┌────┐
       │prev│
       └────┘
     my-file.txt

   - double click hook action
   - make a few different fun icons, (a scroll for text files?)

10. color-pallet element

10. wire-connectors
    - for visualizing routing of information between elements
    - could be directional or non-directional (aka use an arrow or not)
    - it would be cool if it could be used with a border pane WITHOUT
      actually needing to do anything special in the border pane
       - this may need new drawing logic to allow it perform conditional logic
         of the DrawCh based on the cell underneath of it
          - kind of like how transparency takes the cell underneath maybe
            the ChPlus could also have custom applications based on whats under

20. Prompt-Window
     - basically an old school prompt window which says some biz then gives you
       a couple options
     - Optionally it could also sieze control of the whole screen, not allowing
       you to interact with the other elements until you answer the prompt
         - could use a big transparent pane that captures all events for this
         - could "flash" the topzone of the window when the users clicks
           elsewhere than the window

20. ScrollablePane: Ensure Element visible. Feed in an element-is then the scrollable pane 
    should move the view to ensure that the provided element is visible.

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
    
10. vertical tabs (like brave)

11. tabs with border:
╭─────╮────╮────╮──────╮  ╭─────╭────╮────╮──────╮ ╭─────╭────╭────╮──────╮
│tabs1│tab2│tab3│ tab4 │  │tabs1│tab2│tab3│ tab4 │ │tabs1│tab2│tab3│ tab4 │
│     └────────────────╮  ├─────┘    └────┴──────┤ ├─────┴────┘    └──────┤
│                      │  │                      │ │                      │
│                      │  │                      │ │                      │
│                      │  │                      │ │                      │
│                      │  │                      │ │                      │
└──────────────────────┘  └──────────────────────┘ └──────────────────────┘
10. table widget        

10. hover comments
     - hover comment event which is triggered after a certain amount of time
     - TUI option to disable hover comments
     - should just be a special floaty window (with "high z" use BrintToFront)
     - destroyed on the first external event that it receives
     - All this logic should exist at the Pane level 
       - will have to refactor code such that everything now DOES call the pane
         receive event function.

10. widget: date selector

10. widget: color selector

10. TGIF

10. When the keyboard is matching an event combo provided to it, it should be
    recording a partial match (and a suggested maximum wait time to recheck for
    priority to this combo whether to wait the time before checking for other
    matches or to ignore the wait and to proceed attempting to match the
    character in other ways.  

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


20. Add another cargo repo like AssertCmd for tui
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

30. :Command pane and functionality
      - use custom event routing system

40. custom mouse types using images (requires image support, and mouse pixel tracking) 

40. tui get the color under the cursor pixel - useful for color pickers or from actual image pallets

30. PIXEL MODE SPLITS for Complex Selector
 - SPLITS: for pixel selection mode need a way to represent split selection 
   within one cell. Probably we just need to define special characters for these
   positions. The user would have feed in manual definitions (NOTE don't attempt
   to abstract more complex patterns, too much work too implement).

      OptionL  OptionA          KKKLLLLLLLLLLAAAAAAAAAABBB     
   OptionK.⟍ __ ⟋.OptionB       8KKKKKKKK2LLLAAA1BBBBBBBB5
 OptionJ -  ╱  ╲  - OptionC     JJJJJJJJJ88885555CCCCCCCCC
 OptionI -  ⚬__╱  - OptionD     IIIIIIIII77776666DDDDDDDDD 
   OptionH´⟋    ⟍`OptionE       7HHHHHHHH3GGGFFF4EEEEEEEE6
      OptionG  OptionF          HHHGGGGGGGGGGFFFFFFFFFFEEE 

1 = upper A lower B
2 = upper L lower K
3 = upper H lower G
4 = upper E lower F 
5 = upper B lower C 
6 = upper D lower E
7 = upper I lower H
8 = upper K lower J
   > allow for possible splits:
     - diagonal (both ways)
     - half (horizontal and vertical)
     - quarters (square and diagonal) 

40. Subscription based events on common objects. 
     - like leptos. any element could subscribe to an object (with any other
       element can change). When that object changes it would send out events to
       any other elements which subscribed to it... OR maybe it would just make
       sense to use hooks this way you don't need all the parents of the
       destination to also subscribe to the hook. USE HOOKS!
       - Actually could be really easy with the Event Router - could use Custom
         Event
       - question is: what events should actually be broadcast?

50. LOW PRIORITY CAN JUST USE $EDITOR. widget: vim-style textbox
     - with two scrollbars the mode can be placed in 
       the decorations corner!

