# YEEHAW

~ Batteries Included Text Based Application Framework ~

'yeehaw' was born out of a need for an adaptable design for sophisticated text
based applications, with the goal of presenting as much information as cleanly
as possible. 

**Reasons why you need your application to be text-based:**
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
 - Catering to non-UTF-8 or non-true-color terminals.
 - Minor performance improvements at the cost of developer ergonomics

## Existing Elements:
[TODO link to a seperate markdown with a bunch of GIFS]
 - tabs 
 - stack panes (think vim buffers) 
 - scrollable panes
 - top-down menu
 - right click menu
 - file viewer
 - file navigator (think nerdtree)
 - image viewer 
 - windows
 - terminal (that can open other TUIs!)
 - basic textbox
 - $EDITOR textbox (ex. ACTUAL neovim)  
 - figlet fonts (aka MEGAFONTS)
 - button
 - checkbox
 - dropdown-list
 - label
 - listbox (optional multi-entry)
 - radio-buttons
 - scrollbars
 - numbers textbox
 - toggles
 - generalized label decorators on all widgets

## Planned
 - Put a whole dang yeehaw-TUI into a Stateful ratatui widget
 - mini-TUIs in the CLI (aka. without taking up the whole screen)
 - accordion stack container
 - hover comments anywhere
 - vertical tabs (like brave browser) 
 - vim-style command input system (complete with events and routing)
 - ANSI-animation player (using extended asc format)
 - optional mouse pixel support
 - wire-connectors
 - dials
 - color selector element
 - table element
 - Interactive debugging TUI application
 - TUI Snapshot Tester
 - Drag and Drop TUI Application Builder (as a TUI of course)

## Tribute

[notcurses](https://github.com/dankamongmen/notcurses) insane

[jexer](https://gitlab.com/AutumnMeowMeow/jexer) what the heck!!!

[ratatui](https://ratatui.rs/) obviously rocks, [well done](https://www.youtube.com/watch?v=9wm1D6Rk8TE)

[bubbletea](https://github.com/charmbracelet/bubbletea) lookin' good! (golang)

## Contributing 

It'd be cool for this repo to become a monolith. I want all sorts of weird
gadgets in this baby. All ideas will be considered with an open mind, if you'd
like to build and element and merge it into yeehaw It'd be an honour. All
contributions will be merged with the implicit assumption that they will use the
LICENSE as this repo. Additionally this repo will be transitioning to dynamic
ownership based on contributions in the future, so if your code becomes merged
then your be gaining a part piece of ownership whenever dynamic ownership is
integrated in (more on that later!).
