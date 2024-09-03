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

[ratatui](https://ratatui.rs/) rocks! Check it out. Their design goals are
slightly different, it's a more mature TUI framework which you should consider.

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
 - vim-style command input
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

## Planned Elements
 - accordion stack container
 - hover comments
 - vertical tabs
 - widgets:
   - colour selector
   - pixel art viewer
   - image viewer
   - table 
   - vim-style text editor

## Planned Tooling
 - TUI Application Builder 
   - basically drag and drop style element builder
 - Interactive Debugging TUI 
 - Wimp
   - Text based image editor 
     - Similar https://terminalroot.com/use-ms-paint-directly-in-terminal/
   - ANSI-GIFs = ANSGIF
 - TUI Tester

