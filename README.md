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

## Tribute

[notcurses](https://github.com/dankamongmen/notcurses) insane

[jexer](https://gitlab.com/AutumnMeowMeow/jexer) what the heck!!!

[ratatui](https://ratatui.rs/) obviously rocks, [well done](https://www.youtube.com/watch?v=9wm1D6Rk8TE)

[dioxus](https://github.com/dioxuslabs/dioxus) seems cool - they have a goal of
TUI support in the future.

