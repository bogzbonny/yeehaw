THIS document is a dumping grounds for design decisions/brainstorming for
various elements and features. The intention is that text from this document
will eventually be sifted through and included in actual docs - at which point
this document will be deleted
__________________________________________________


05. Ability to use arrow keys to select menu item when menu is open
10. table element
     - option to use underline instead of box drawing ch
     - autowidth or fixed width options
01. reduce CPU load by attempting to minimize the amount of calculations in the
    main tui render loop
     - aparently this line is a bottleneck https://github.com/bogzbonny/yeehaw/blob/main/yeehaw/src/tui.rs#L479
        - Yeah that line is basically checking to see if the content on the
          screen needs to be redrawn at the provided position, yeehaw keeps a
          hashmap of what it last drew on the screen which it's referencing
          there. I'm sure there's a cheaper way to do perform this logic, either
          through just using a 2D array or dashmap. 
     - also this section: https://github.com/bogzbonny/yeehaw/blob/main/yeehaw/src/tui.rs#L433 
           yeehaw/src/tui.rs:433 
     - BUGS
       - seems like each elements are overwriting things to their right?
01. BUG the do not click window still has the tabs-top in it
01. go through and remove / or make dyn element fns named with_width with_height
01. widgets example should be the same as showcase
01. upgrade max_of to .with_max (and min)
01. replace receive_event_inner with just receive_event throughout
01. unify all the action function names and with fns to just be "with_fn"
    "set_fn"
01. remove ctx from click functions thoughout elements... what a pain they are
     - either that or make them a super basic context, with just the ev_sender
       and hat... or not even if you need the context, you should just clone it
       in.
01. resizing of tab with image in wezterm is leaving artifacts for kitty too.
01. tuvix image off by one
01. terminal inside window with border doesn't work
01. disable debug logging in examples
01. update rust version, update deps
01. add docs, crates button banners to github readme
01. just use an image for the banner (and include the existing text as 
    markdown comment
01. add landing page docs for docs.rs
     - resolve broken links 
        cargo doc --open
01. gif of showcase example - will need to record with non-VHS/asciicinema-tool
    to get mouse movements
     keep under 10mb to upload to github (upload then close issue)
     - ffmpeg -i showcase.mov -b:v 1000k showcase.mp4
01. textbox cut/paste shouldn't work when non-editable
01. dropdown list extends into white space if picking beyond the end (clicking
    that whitespace panics)
01. textbox shouldn't capture scrolls mouse events if it doesn't need to. 
01. textbox is for some reason getting j and h instead of enter INSIDE of a
    showcase-inner... truly bizarre
      - enter/backspace still works in other elements within the showcase-inner
         for some reason its JUST the textbox
01. funny coloring artifacts on the slider when moving out of frame in a
    scrollable pane
      - I think it has to do with what the actual gradient is being drawn as!
10. tile color
     - create a pattern-repeatable Color, kind of like a texture gradient
10. switch to vt100_yh or fork https://docs.rs/vt100-ctt/latest/vt100_yh/struct.Screen.html
     - make a PR to expose the grid so that one can actually iterate the cells
     - integrate in SGR-Pixel mode into vt100
01. showcase example 
     - a menu
     - tabs
       - terminal
       - the showcase within the showcase?? spiral 
     - window generator zone
       - dial with window pane types: basic, scrollable, scrollable-expanding, terminal, 
       - border-kind: none, basic, normal scrollbars, resizer with border-scrollbars, double width, large, tight, 
                      border-with-text, border-resizer, border-mover
       - shadow cb
       - generate button
       - widgets
         - buncha widgets which dont do much but log their results 
           in a textbox
         - TWO dials as eyes (draw a mouth label below, maybe a nose too)
           - no labels for the eyes
           - make them lock together, and change the mouth based on their 
             position
       - gradient
         - a dial with a few choice fg ascii arts 
           - none, butterfly, spiral, saturn, rust logo, chompy
         - toggle for FG and BG Color
           - dial gradient kind: solid color, time-gradient, radial gradient,
             linear gradient. radial-time, linear-time
           - some sliders: 1 for each color (greyed out when colors disabled)
             maybe with an RGB tb for css color input
           - dropdown, number of colors (greyed out for solid-color kind)
           - slider, gradient size between colors (enforce static) 
           - slider with numbers-tb for gradient angle
           - slider, change speed, ms for time gradients
     - colors, certain gradients with alpha do not blend properly probably
       because the transparancy source is incorrect.
     - make sure this looks good at different scales 
     - a big "DO NOT PRESS button" which instigates the blue screen of death
     - TABS Zone
       - neovim $EDITOR tab
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
      - expanding pane scrollable is not refershing properly on scrolls
         - looks like the leftmost chs are not being moved
      - terminal inner-showcase is not removing cursor in sl
         - the cursor is removed once an event is send into the "showcase" pane
           but not before... confirmed that it IS the cursor which the yeehaw-terminal pane is
           drawing. This event is removing the cursor because it will set
           hide_cursor to true, which is otherwise not yet set.
      - showcase train completion doesn't reset the color of the pane (should be
        black and not say exit
           - the issue is that the terminal pane doesn't have receivable events
             now that the prioritizer was removed and the receivable events are 
             all lumped together (for focused and non-focused)
      - terminal inner-showcase has funny bg color on widget dial
         - VT100 wierdness: It turns RGB colors into defaults for some insane
           reason... just dont' just default colors

10. Mouse routing.
    - integrate in better capture event logic, if the mouse event is NOT
      captured, send the event to the next priority down. [DONE]
       - this could potentially also be applied to the accomplish the scroll
         situation as described above.. first send the event to the inner pane,
         then if the mouse scroll event is not captured then send it to the
         scrollable pane. - maybe then wouldn't need the mouse event logic??

30. WONT DO irregular gradient lines
    - OUTWARD
      - a gradient moving outward from an irregular set of coordinates (making a
        line
      - basically just a bunch of radial point gradients however when they
        interact the lowest gradient position should just be used (as opposed to
        a blend)

40. WONT DO Subscription based events on common objects. 
     - like leptos. any element could subscribe to an object (with any other
       element can change). When that object changes it would send out events to
       any other elements which subscribed to it... OR maybe it would just make
       sense to use hooks this way you don't need all the parents of the
       destination to also subscribe to the hook. USE HOOKS!
       - Actually could be really easy with the Event Router - could use Custom
         Event
       - question is: what events should actually be broadcast?

05. introduce errors, remove all unwraps
05. clicking while on the menu bar should collapse & deselect the bar
01. improve efficiency the showcase already feels laggy
     - seems specifically to do with nesting things in element organizers
         - the widget_test has lots lots more elements but no lag
         - each layer of element organizer causes additional redraw adjustments
           to be required
     - for time based redraws the element could have a receivable event which it
       registers
     - each element organizer could keep a queue of the previous draw characters
       which 

01. Draggin of border panes doesn't work in showcase

01. resize event in the "wrong direction" for a stack should not just ignore the
    resize but pass on that resize to the next higher element (such that if the
    next higher element was a stack which could actually use that resize command
    it would utilize it)
01. fix the terminal_editor
01. terminal not shutting down when showcase shuts down
     - I think due to tabs not propogating the closedown events 
01. ensure that the line_numbers small textbox is the placed correctly in the
    editor
01. fix the line_numbers textbox
     - this seems that it may be a buffering issue, when I rescale it seems to
       correct (in widgets test)
01. scrollbars dont properly get created in editor (only later)
    - seems to be using the height of the entire screen here
    - the ctx height of the scrollbar is not adjusted when the screen size
      changes (resize)
01. terminal_editor - get the no-editor elements hooked up.

30. WONT DO the ArbSelector works fine
PIXEL MODE SPLITS for Complex Selector
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

01. window_test: maximizing terminal window doesn't enlarge the inner terminal

01. running "top" inside terminal window shoots the cursor outside of the
    window! should correct for this

01. It'd be cool to come up with a "Arbitrary Selector" generalization for the dials. 
 - for now, this arbitrary selector should probably be the only way to initialize
   a dial... eventually we could automatically produce the maps, but it can get
   complicated as the text of the different options changes.
 - probably each version of the dial (3 postion... 8 position etc) should be a
   different arbitrary selector. 
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
   could be "snapped to"
 - Arbitrary Selector, feed in:
    - positions map 
    - base DrawCh2D
    - selection-made DrawChPos changes
       - these are drawn on top of the base for each selection
       - for the dial for ex, this is all the differences


10. automated dial for arb selector
    - feed in selection style
    - feed in text for each dial position 
      - dial to rectify positions to arb_selector positions
      - aka you can easily skip the tight-inbetween positions
    - feed in spacing option
       - ultra-compact (like 1 letter options) - max 8 positions
       - compact (first options)  - max 8 positions
       - semi-compact - max 12 positions 
       - spacious - max 12 positions 
    - dial automatically create arb_selector position map from texts
               op                op                op                op 
   __    __    __    __    __    __    __    __    __    __    __    __ 
  ╱° ╲  ╱ °╲  ╱  °  ╱  ∘  ╱  ╲  ╱  ╲  ╱  ╲  ╱  ╲  ╱  ╲  ╱  ╲  ⚬  ╲  °  ╲
  ╲__╱  ╲__╱  ╲__╱  ╲__╱  ╲__°  ╲__∘  ╲_∘╱  ╲∘_╱  ⚬__╱  °__╱  ╲__╱  ╲__╱  
                   
One letter labels
   A__B      A__B       A__B       A__B     ultra compact
 H ╱° ╲ C  H ╱  ⚬ C   H ╱  ╲ C   H ╱° ╲ C
 G ╲__╱ D  G ╲__╱ D   G ╲__° D   G ╲__╱ D              
   F  E      F  E       F  E       F  E  

           OptionH __ OptionA         compact
         OptionG  ╱° ╲  OptionB
         OptionF  ╲__╱  OptionC
           OptionE    OptionD  

           OptionL  OptionA           semi-compact 
        OptionK   __   OptionB
       OptionJ   ╱  ╲   OptionC
       OptionI   °__╱   OptionD
        OptionH        OptionE
           OptionG  OptionF    

            OptionL  OptionA           Spacious
         OptionK   __   OptionB
       OptionJ    ╱  ╲    OptionC
       OptionI    °__╱    OptionD
         OptionH        OptionE
            OptionG  OptionF      

      OptionL  OptionA        KKKLLLLLLLLLLAAAAAAAAAABBB
   OptionK   __   OptionB     JKKKKKKKKKLLLAAABBBBBBBBBC
 OptionJ    ╱  ╲    OptionC   JJJJJJJJJJJJJCCCCCCCCCCCCC
 OptionI    °__╱    OptionD   IIIIIIIIIIIIIDDDDDDDDDDDDD 
   OptionH        OptionE     IHHHHHHHHHGGGFFFEEEEEEEEED
      OptionG  OptionF        HHHGGGGGGGGGGFFFFFFFFFFEEE 
 
    - exterior whitespace allocation: 
       - top row divide 3/5 to B and 2/5 to A
       - middle row divide 3/5 to B and 2/5 to C
       - examples: 

         OptionA        AAAAAAAAAABBB
        _   OptionB     AAABBBBBBBBBC
         ╲    OptionC   CCCCCCCCCCCCC

         OptionA        AAAAAAAAAABBB
        _   OpB         AAABBBBBBBBCC
         ╲    OptionC   CCCCCCCCCCCCC

         OptA           AAAAAAAABBBBB
        _   OptionB     AAABBBBBBBCCC
         ╲    OptionC   CCCCCCCCCCCCC


01. Make the context size more clear.
     - The context provided always contains the size of the element. However
       during initialization, before the size of an element is known, the
       context fed in will be the parent context. This is confusing as heck and
       I hope to rectify this, possibly by providing both the parent-element
       size and size-for-the-element as options within the context. 
     - Actually, NO parent size, parent context is provided so one can get the
       size that way.
     - parent size should not be an option, only the child size should be an
       option. parent size must always be known (get from parent Ctx)
       - calling pane.width(ctx) should be the same as ctx.child.width if the child
         size is provided
     - Maybe there should be an Initialization context where the Size is an
       option and the regular context where the size is nolonger an option!?
         - makes it maybe a bit annoying with some duplicated logic however?
01. right click menu of the textbox deselects the textbox

01. scrollbars on textbox grow ever so slightly once the texbox is entered.
      - scrollbars on textbox have bad starting size
10. element slider bars / track bars
   ━━━━━━━━━━╉─────────────────────          
   ━━━━━━━━━━⛊─────────────────────          
   ██████████╋━━━━━━━━━━━━━━━━━━━━━          
   ━━━━━━━━━● 
   ━━━━━━━━❍─────────────────────── 
   Options:
    - styles:
      - one sided
        - end ch
        - left ch      NOTE to set it to a different color, just use a pos gradient
        - right ch
10. organize elements into subfolders
      - containers
      - widgets
      - panes... keep in root?
         - (pane,parent_pane)
      - terminal 
      - misc 
         - shadow
         - menu
         - right_click_menu

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
