# YEEHAW

The Text Based Application Framework. 

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
5) You're server-side WebUIs are centralized and weak as shit

## Tribute

[ratatui](https://ratatui.rs/) rocks! Check it out. The design goals are
slightly different but it's a mature TUI framework you should consider.

## Examples

[TODO gifs]

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
 - widgets:
   - colour selector
   - pixel art viewer
   - table 

## Planned Tools
 - TUI Application Builder 
   - basically drag and drop style element builder
 - Interactive Debugging TUI 
 - Wimp
   - Text based image editor (Think 'GIMP') 
   - Text-GIFs = TGIF

_________________________________________________

general purpose complexity.

