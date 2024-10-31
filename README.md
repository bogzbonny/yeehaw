# YEEHAW

~ Batteries Included Text Based Application Framework ~

'yeehaw' was born out of a need for an adaptable design for sophisticated text
based applications, with the goal of presenting as much information as cleanly
as possible. 

# Reasons Why You Want Your Application to be Text-Based

1) it's the only way you'll ever be cool again
2) they're conceptually straightforward, it's just a grid 
3) they're rapidly iterable during development
4) they're extremely extensible -> nesting other TUIs in your TUI is a
   flippin breeze with yeehaw
5) they fas

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

### Non-objectives
Catering to non- UTF-8 or non- true-color terminals.

## Existing Elements:
 - tabs 
 - stack panes (think vim buffers) 
 - scrollable panes
 - top-down menu
 - right click menu
 - file viewer
 - file navigator (think nerdtree)
 - image viewer 
 - windows
 - widgets:
   - figlet fonts 
   - button
   - checkbox
   - dropdown-list
   - label
   - listbox (optional multi-entry)
   - radio-buttons
   - scrollbars
   - textbox (editable) 
   - numbers textbox
   - titles
   - toggle (yes -> no) 
   - generalized label decorators on all widgets

## Planned
 - built in terminal
    - https://github.com/a-kenji/tui-term/blob/development/examples/smux.rs
    - easy use of Editor
 - accordion stack container
 - hover comments
 - vertical tabs
 - vim-style command input system (complete with events and routing)
 - ANSI-animation viewer (using extended asc format)
 - optional mouse pixel support
 - wire-connectors
    - for visualizing routing of information between elements
    - could be directional or non-directional (aka use an arrow or not)
 - widgets:
   - color selector
   - table 
 - Interactive debugging TUI application
   - https://github.com/eclipse-iceoryx/iceoryx2 
 - TUI Snapshot Tester
   - use a toggle to switch between result/expected
   - diff view (only show the differences)
   - eventually allow for multi-stage
   - see `script` standard binary
 - TUI Application Builder 
   - basically drag and drop style element builder
   - resizing of the view-pane to test TUI pages at different 
     sizes

## Tribute

[notcurses](https://github.com/dankamongmen/notcurses) insane

[jexer](https://gitlab.com/AutumnMeowMeow/jexer) what the heck!!!

[ratatui](https://ratatui.rs/) obviously rocks, [well done](https://www.youtube.com/watch?v=9wm1D6Rk8TE)

[dioxus](https://github.com/dioxuslabs/dioxus) seems cool - they have a goal of
TUI support in the future.

