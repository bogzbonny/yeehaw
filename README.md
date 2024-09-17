# YEEHAW

~ Batteries Included Text Based Application Framework ~

'yeehaw' was born out of a need for an adaptable design for sophisticated text
based applications, with the goal of presenting as much information as cleanly
as possible. 

Core to 'yeehaw' is an element ownership model. TUI Elements are arranged in a
hierarchical manner and route event information (keyboard/mouse events) between
them. Parent elements hold ownership over child elements and determine how the
flow of events is channeled, in addition they also determine which child
elements are viewed and where within the parent element they are displayed.
Elements are only required to have spatial awareness within the confines which
have been assigned to them from parent elements.  

# Reasons why you want your application to be text-based

1) It's the only way you'll ever be cool again
2) They're rapidly iterable
3) They're distraction free
4) They're conceptually simple, it's just a grid 

## Tribute

[ratatui](https://ratatui.rs/) obviously rocks. its design goals are
slightly different

[dioxus](https://github.com/dioxuslabs/dioxus) seems lit - they have a goal of
TUI support in the future.

[rooibos](https://github.com/aschey/rooibos) similar project, different approach

## Examples

[TODO gifs] NOTE use VHS to produce gifs

## Existing Elements:
 - tabs 
 - stack panes (think vim buffers) 
 - scrollable panes
 - top-down menu
 - right click menu
 - file viewer
 - file navigator (think nerdtree)
 - widgets:
   - megafonts
   - button
   - checkbox
   - dropdown-list
   - label
   - listbox
   - radio-buttons
   - scrollbars
   - textbox (editable) 
   - titles
   - toggles

## Planned
 - ratatui element
 - accordion stack container
 - hover comments
 - vertical tabs
 - vim-style command input system
 - widgets:
   - colour selector
   - table 
 - ratatui imported:
   - image viewer 
   - vim-style text editor
 - TUI Application Builder 
   - basically drag and drop style element builder
 - Interactive Debugging TUI 
 - TUI Snapshot Tester
 - Ansi-animation viewer (using WIMP asc format)

## Planned Tooling
 - seperate project: WIMP
   - like durdraw but with more features: 
     - layers within each frame
     - alpha
   - Text based image editors
     - https://github.com/mkrueger/icy_tools?tab=readme-ov-file
       - pretty cool need to build custom 
       - doesn't yet support unicode
       - has a bunch of animation file formats built in
     - https://github.com/cmang/durdraw/
        - has ansi-animations
        - use a new custom ansi animation format, simply define a sleep
          functionality with APC codes (like kitty uses)
          <ESC>_sleep<ms><ESC>\
          <ESC>_sleep16<ESC>\
     - https://aac.iverv.com/about
     - Similar https://terminalroot.com/use-ms-paint-directly-in-terminal/
     - https://www.gridsagegames.com/rexpaint/
     - https://github.com/EtoDemerzel0427/ANSI-art


