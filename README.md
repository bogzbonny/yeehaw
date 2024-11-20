<!--
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
░ꕤ                                 |    |    |                                              ꕤ ░
░         _________               \|/  \|/  \|/           _̉_̉_̉_̉         3                      ░
░        /         \              \|/  \|/  \|/   ☉     \/  x \              ______.          ░
░        | yeeeehhaaw!!!!!!!!!!   \|/  \|/  \|/        \       \         ___/_____ꕤ_\___,     ░
░  \_____/   _    _ \_____/        |    |    |        \/    _\  \          /|||||||||\        ░
░           >    o< ,                _______________  /   /     ..        / ⹁╷,   ⹁╷, \       ░
░        C     \                    /   █ █  █ █   ma  \  \               ╳  .     .  ╳   7   ░
░        `           \             / /   █   █▀█     j    |        well   ╳     /     ╳       ░
░          \> \-̲̅-̲̅./   |            \/    ▀   ▀ ▀      e   \    howdee     ╳  \     r  ╳       ░
░         | \     `.  /          \_/     __________/// s   |     there!   ╳     -̅     ╳_      ░
░         |  \      `----<<<-        \     |        /   t /                       ╷    \      ░
░        /    |__|__|                /     }       /    i \                                   ░
░ꕤ                                  /     /        \   c  /                                 ꕤ ░
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
-->
[IMAGE](TODO IMAGE OF BANNER HERE)

# YEEHAW

~ Batteries Included Text Based Application Framework ~

yeehaw was born out of a need for an embeddable and reusable interface-element
pattern for sophisticated text based applications. 

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
 - sliders
 - dials

## Planned Stuff:
 - put a whole dang yeehaw-TUI into a stateful ratatui widget, why not!
 - mini-TUIs in the CLI (aka. use a TUI in-line with your command without taking
                         up the whole terminal)
 - accordion stack container
 - hover comments for elements
 - vertical tabs (like brave browser) 
 - [.ans]-animation player (using an extended-ansi format)
 - optional mouse pixel support
 - wire-connectors between elements
 - color selector element
 - table element
 - an interactive debugging application for yeehaw TUIs
 - TUI Snapshot Tester
 - drag and drop TUI application builder (as a TUI of course)
 - build out a system of feature-flags for feature restriction / compile time
   improvement.

# Design Overview

Element ownership overview: TUI Elements are arranged in a hierarchical manner
while retaining semi-autonomy. Events are routed from the top down, and
responses can be propagated upwards from deeply nested elements. Additionally,
elements may maintain direct communication lines with any other element through
the use of hooks and other element-specific function variables (e.g. the button
click function on a button element). Parent elements retain general authority
over child elements; determining how the flow of events are channeled, and the
location and size of child elements. Simple elements are only required to have
spatial awareness within the confines provided to them - although autonomy is
still given for them to change their ordering and position within their
immediate parent element (with great power comes great responsibility).  

The core Element Trait has designed to be extensible for custom event/response
kinds enabling developers to create entirely new sub-classes of elements which
can reuse the event routing system logic. 

Looking to understand more? Checkout:
 - [examples](TODO)
 - [Element Trait](TODO)
 - [Pane](TODO) <- the standard base for all built in elements
 - [ParentPane](TODO) <- the standard base for all elements which hold other elements
 - [Context](TODO) <- an object which can be found nearly everywhere
 - [DynVal](TODO) <- the basis for all layouts and sizes


### Design Principles 

 - Elements should present information as cleanly as possible.
   - tooling should be provided to minimize the need for use of box character
     borders, for instance through contrasting backgrounds.
 - The element trait, and yeehaw's design in general should be as versatile as
   possible - allowing for the development of highly specific obscure elements 
   and features without having to break the core design.
 - Developing a simple element should require as no information about its
   surrounding environment. This said, more complex elements should still be
   able to responsibly interact with its surroundings directly if necessary -
   elements should __not__ be limited to only interacting with its parent in the
   rigid element-hierarchy through event responses. Although this rigidity
   provides consistency for overall design, it can drastically complicate
   certain inter-element interactions.
 - Keep as much stuff `pub` as possible to allow for more experimentation
   without requiring forks of the repo. (Eventually put all the excess pub under
   an `internals` feature flag to reduce breaking changes).
 - Favour robustness over correctness for release mode (but vise-versa during
   debug mode). Many small and strange UI bugs are resolvable via user
   intervention. Ideally the TUI should never panic during release mode.

### Non-Objectives

 - catering (too much) to non-UTF-8 or non-true-color terminals
 - minor performance improvements at the cost of developer ergonomics

## Stability, Upcoming Refactors, Bugs 

If you plan to build on yeehaw right now, that's great news! I'd like to keep
you apprised of some upcoming changes. If you do wish to experiment and or start
development on yeehaw I wouldn't be too afraid of these upcoming changes, the
majority of foreseeable major refactors have already been completed.  While
yeehaw is pre-1.0.0 all breaking changes will take place with a semver minor
version upgrades which will be all new releases. In the short-term I don't plan
on providing patch updates for bug fixes for minor versions.

I'll try'n help out anyone who needs a hand understanding how to update their
code if its been broken by a new release. Additionally a breaking changes doc
with upgrade instructions shall be maintained. 

HAVE NO FEAR

 - There ain't much automated testing in here at all, soon a TUI snapshot tester
   is going to be developed, which should bring up coverage from about 0% as it
   stands. 
 - Taffy is going to be integrated in as an extension to the `DynLocationSet`
   system. It won't change the existing location mechanisms just build on
   top of them.
 - Proper window minimization behaviour is blocking on the Taffy integration such
   that the minimized windows can follow a nice easy grid pattern. Currently
   minimization still somewhat works, however multiple minimized windows will
   stack on each other in the same location. 
 - The $EDITOR text editor element - aka the element where you could use any
   editor like neovim/vim(/emacs I think?) currently doesn't provide good
   support for users who HAVEN'T set their $EDITOR env variable. This will be
   fixed at some point soon.
 - Default colors everywhere are going to be replaced with a defaults in a theme
   manager. Using the Theme Manager the developer can start from a nice overall
   default then modify it to their liking. Note the Theme manager will not
   inhibit users from specifying specific colors anywhere they choose. 
 - Gradients on irregular angles are not stable, the goal is to have the
   gradient actually reflect a visual angle taking into account the width and
   the height of each cell. Currently the angles work under an assumption of
   equal cell width and height, sometimes it produces funny/unexpected results
   for a gradient which has is supposed to just be at a 45-degree angle and
   occur only once across the whole target area (\x60DynVal::FULL\x60). Gradients on
   angles which are repetitive (`DynVal::fixed(..)`) work good, however the way
   the angle is interpreted will likely change to account for cell dimensions.
   Gradients on right-angles (0, 90, 180, 270 degrees) are stable.

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
