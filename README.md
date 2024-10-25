# YEEHAW

~ Batteries Included Text Based Application Framework ~

'yeehaw' was born out of a need for an adaptable design for sophisticated text
based applications, with the goal of presenting as much information as cleanly
as possible. 

# Reasons Why You Want Your Application to be Text-Based

1) It's the only way you'll ever be cool again
2) They're rapidly iterable during development
2) They're conceptually straightforward, it's just a grid 
4) They minimize hidden UI features, it's all on the grid 

## Examples

[TODO gifs] -> VHS to produce gifs

# Design Overview

Core to 'yeehaw' is an element ownership model. TUI Elements are arranged in a
hierarchical manner and route event information (keyboard/mouse events) between
them. Parent elements hold ownership over child elements and determine how the
flow of events is channeled, in addition they also determine which child
elements are viewed and where within the parent element they are displayed.
Elements are only required to have spatial awareness within the confines which
have been assigned to them from parent elements.  

## Tribute

[notcurses](https://github.com/dankamongmen/notcurses) insane

[jexer](https://gitlab.com/AutumnMeowMeow/jexer) what the heck!!!

[ratatui](https://ratatui.rs/) obviously rocks, [well done](https://www.youtube.com/watch?v=9wm1D6Rk8TE)

[dioxus](https://github.com/dioxuslabs/dioxus) seems cool - they have a goal of
TUI support in the future.

[rooibos](https://github.com/aschey/rooibos) similar project, different approach


## Existing Elements:
 - tabs 
 - stack panes (think vim buffers) 
 - scrollable panes
 - top-down menu
 - right click menu
 - file viewer
 - file navigator (think nerdtree)
 - image viewer 
 - widgets:
   - figlet fonts 
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
 - windows
 - built in terminal
    - https://github.com/a-kenji/tui-term/blob/development/examples/smux.rs
    - easy use of Editor
 - optional mouse pixel support
 - accordion stack container
 - hover comments
 - vertical tabs
 - vim-style command input system (complete with events and routing)
 - interactive debugging TUI application
 - TUI Snapshot Tester
 - ANSI-animation viewer (using extended asc format)
 - TUI Application Builder 
   - basically drag and drop style element builder
 - widgets:
   - color selector
   - table 

## Planned Tooling
 - seperate project: WIMP
   - layers within each frame
     - use https://github.com/tomcur/termsnap/blob/main/src/main.rs
        for layer icons
     - then use https://www.reddit.com/r/rust/comments/p4l610/render_svg_to_pngother_format/ 
       https://github.com/RazrFalcon/resvg?tab=readme-ov-file
       to render to png
     - alpha channel
   - use a new custom ansi animation format, simply define a sleep
     functionality with APC codes (like kitty uses)
        <ESC>_sleep<ms><ESC>\
        <ESC>_sleep16.66<ESC>\
        <ESC>_repeat<ESC>\ // for repeating sequences ? maybe not?
        - super basic application for viewing with these sequences
          - extcat or excat or ecat
     - Deduplication considerations: when writing frames, all the duplicate
       draws should simply be ignored and not written, only the changed places
  - inspiration: 
    - https://github.com/mkrueger/icy_tools?tab=readme-ov-file
    - https://github.com/cmang/durdraw/
    - https://terminalroot.com/use-ms-paint-directly-in-terminal/
    - https://www.gridsagegames.com/rexpaint/
    - https://github.com/EtoDemerzel0427/ANSI-art
