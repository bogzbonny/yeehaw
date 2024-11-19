```
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
░ꕤ                                 |    |    |                                              ꕤ ░       
░         _________               \|/  \|/  \|/           _̉_̉_̉_̉         3                      ░      
░        /         \              \|/  \|/  \|/   ☉     \/  x \              ______.          ░     
░        | yeeeehhaaw!!!!!!!!!!   \|/  \|/  \|/        \       \         ___/_____ꕤ_\___,     ░       
░  \_____/   _    _ \_____/        |    |    |        \/    _\  \          /|||||||||\        ░        
░           >    o< ,                _______________  /   /     ..        / ⹁╷,    ⹁╷,\       ░     
░        C     \                    /   █ █  █ █   ma  \  \               ╳  .      . ╳   7   ░        
░        `           \             / /   █   █▀█     j    |        well   ╳     /     ╳       ░         
░          \> \-̲̅-̲̅./   |            \/    ▀   ▀ ▀      e   \    howdee     ╳  \     r  ╳       ░       
░         | \     `.  /          \_/     __________/// s   |     there!   ╳     -̅     ╳_      ░
░         |  \      `----<<<-        \     |        /   t /                       ╷    \      ░              
░        /    |__|__|                /     }       /    i \                                   ░       
░ꕤ                                  /     /        \   c  /                                 ꕤ ░                     
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

```

# YEEHAW

~ Batteries Included Text Based Application Framework ~

yeehaw was born out of a need for an adaptable, embeddable, and fun element
design for sophisticated text based applications. 

**Reasons why you need your application to be text-based:**
1) it's the only way you'll ever be cool again
2) they're conceptually straightforward, it's just a grid 
3) they're rapidly iterable during development
4) they're extremely extensible -> nesting other TUIs in your TUI is a
   flippin breeze with yeehaw
5) they fas

## Examples

[>>>>>>>>>>>>>> MORE GIFS <<<<<<<<<<<<<<<<<<](TODO LINK TO MORE GIFS)

[TODO] Insert Showcase gif

## Existing Elements:
[TODO link to a separate markdown with a bunch of GIFS]

 - $EDITOR textbox (e.g. ACTUAL neovim... wow!)  
 - basic textbox
 - tabs 
 - stack panes (think vim-buffers) 
 - scrollable panes with scrollbars
 - top-down menus
 - right click menu
 - file viewer
 - file navigator (think nerdtree)
 - image viewer (thanks to [ratatui-image](https://github.com/benjajaja/ratatui-image))
 - windows
 - terminal (that can open other TUIs! ..dang)
 - figlet fonts (aka MONSTER FONTS)
 - buttons
 - checkboxes
 - dropdown-lists
 - labels
 - listboxes
 - radio-buttons
 - scrollbars
 - toggles
 - generalized label decorators on all elements
 - progress bars/sliders

## Planned
 - put a whole dang yeehaw-TUI into a stateful ratatui widget, why not!
 - mini-TUIs in the CLI (aka. without taking up the whole screen)
 - accordion stack container
 - hover comments anywhere
 - vertical tabs (like brave browser) 
 - ANSI-animation player (using an extended-ansi format)
 - optional mouse pixel support
 - wire-connectors (think comfy-ui)
 - dials
 - color selector element
 - table element
 - an interactive debugging application for yeehaw TUIs
 - TUI Snapshot Tester
 - drag and drop TUI application builder (as a TUI of course)

# Design Overview

Core to yeehaw is the element ownership model. TUI Elements are arranged in a
hierarchical but still semi-autonomous manner. Event information
(keyboard/mouse/custom events) is routed from the top down, and responses can be
repropagated upwards from deeply nested elements. Additionally elements may
maintain direct communication lines with any other elements through the use of
hooks and other element-specific function variables (e.g. the button click
function on a button element). Parent elements retain authority over child
elements and determine how the flow of events is channeled, in addition they
also determine which child elements are viewed and where within the parent
element they are displayed. Elements are only required to have spatial awareness
within the confines which have been assigned to them from parent elements,
although autonomy is given for them to change their ordering and position within
their immediate parent element.  

Looking to understand more? Checkout:
 - [examples](TODO)
 - [Element Trait](TODO)
 - [Pane](TODO) <- the standard base for all built in elements
 - [ParentPane](TODO) <- the standard base for all elements which hold other elements
 - [Context](TODO) <- an object which can be found nearly everywhere
 - [DynVal](TODO) <- the basis for all layouts and sizes


### Objectives [WIP]

 - elements should presenting information as cleanly as possible
   - tooling should be provided to minimize the need for use of box character
     borders, for instance through contrasting backgrounds
 - the element trait, and yeehaw's design in general should be as versatile as
   possible - allowing for the development of highly specific and obscure elements 
   and features without having to break the core design.
 - developing a simple element should require as no information about its
   surrounding environment. This said, more complex elements should still be
   able to responsibly interact with its surroundings directly if necessary -
   elements should not be limited to only interacting with its parent in the
   rigid element-hierarchy through event responses.  

### Non-Objectives

 - catering to non-UTF-8 or non-true-color terminals (too much)
 - minor performance improvements at the cost of developer ergonomics

## Stability, Upcoming Refactors, Bugs 

If you plan to build on yeehaw right now, that's great news! I'd like to keep
you apprised of some upcoming changes. If you do wish to experiment and or start
development on yeehaw I wouldn't be too afraid of these upcoming changes, I'll
try'n help out anyone who needs a hand fixing things broken by upcoming
refactors / update a breaking changes doc with upgrade instructions. 
HAVE NO FEAR

- There ain't much automated testing in here at all, soon a TUI snapshot tester
  is going to be developed, which should bring up coverage from about 0% as it
  stands. 
- Taffy is going to be integrated in. It shouldn't change the existing location
  mechanisms just build on top of them.
- Proper window minimization behaviour is blocking on the Taffy integration such
  that the minimized windows can follow a nice easy grid pattern. Currently
  minimization still somewhat works, however multiple minimized windows will
  stack on each other in the same location. 
- The $EDITOR text editor element - aka the element where you could use any
  editor like neovim/vim/emacs(I think?) currently doesn't provide good support
  for users who HAVEN'T set their $EDITOR env variable. This will be fixed at
  some point soon.
- gradients on angles are not fully stable, the goal is to have the gradient
  actually reflect a visual angle taking into account the width and the height
  of each cell. Currently the angles work under an assumption of equal cell
  width and height, sometimes it produces funny/unexpected results for a
  gradient which has is supposed to just be at a 45-degree angle and occur only
  once across the whole target area (`DynVal::full()`). Gradients on angles which are
  repetitive (`DynVal::fixed(..)`), or gradients on right-angles (0, 90, 180, 270
  degrees) are considerably more stable.

## Tribute

[notcurses](https://github.com/dankamongmen/notcurses) insane

[jexer](https://gitlab.com/AutumnMeowMeow/jexer) what the heck!!!

[ratatui](https://ratatui.rs/) obviously rocks, [well done](https://www.youtube.com/watch?v=9wm1D6Rk8TE)

[bubbletea](https://github.com/charmbracelet/bubbletea) lookin' good! (golang)

## Contributing 

It'd be cool for this repo to become a mega repo. I want all sorts of funky
widgets in this baby with first class support from this project. All ideas will
be considered with an open mind, if you'd like to build and element and merge it
into yeehaw It'd be an honour. If you'd like to build a element with highly
specific needs and the current Element trait is non-satisfactory, let's upgrade
it. 
This repo will be transitioning to dynamic ownership based on contributions in
the future, so if your code becomes merged then your be gaining a specialized
part piece of ownership in the project whenever dynamic ownership is integrated
in (more on that later!).

Any contribution you intentionally submit for inclusion in the work, as defined
in the Apache-2.0 license, shall be Apache-2.0 license, without any additional
terms or conditions.
