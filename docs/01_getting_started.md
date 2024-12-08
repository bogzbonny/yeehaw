
## Quickstart:   <!-- NOTE duplicate in README.md -->

A hello world example with a label and a reactive button:

``` rust
use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element");

    // place the label at 30% of the screen width and height
    let label = Label::new(&ctx, "Hello, World!").at(0.3.into(), 0.3.into());

    let label_ = label.clone(); // clone required so we can move it into the button closure
    let button = Button::new(
        &ctx,
        "Click Here!",
        Box::new(move |_, _| {
            label_.set_text("Button clicked!".to_string());
            EventResponses::default()
        }),
    )
    // place the button at 30% of the screen width and 30% of the screen height + 1 character
    .at(0.3.into(), DynVal::new_flex(0.3).plus(1.into()));

    let _ = main_el.add_element(Box::new(label));
    let _ = main_el.add_element(Box::new(button));
    tui.run(Box::new(main_el)).await
}
```

## Existing Elements:  <!-- NOTE duplicate in README.md -->
[TODO link to a separate markdown with a bunch of GIFS]

#### Widgets
 - $EDITOR textbox (e.g. ACTUAL neovim... wow!)  
 - basic textbox
 - top-down menus
 - right click menu
 - image viewer (thanks to [ratatui-image](https://github.com/benjajaja/ratatui-image))
 - file viewer
 - file navigator (think nerdtree)
 - terminal (that can open other TUIs! ..dang)
 - figlet fonts (aka MEGA TEXT)
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

#### Containers
 - tabs 
 - windows
 - stack panes (think vim-buffers) 
 - scrollable panes with scrollbars

## Planned Stuff: <!-- NOTE duplicate in README.md --> 
 - embed a whole dang yeehaw TUI into stateful ratatui widget, why not!
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

# Design Overview <!-- NOTE duplicate in README.md -->

Element ownership overview: TUI Elements are arranged in a hierarchical manner
while retaining semi-autonomy. Events are routed from the top down, and
responses can be propagated upwards from deeply nested elements. Additionally,
elements may directly effect any other element through a variety of hooks (e.g.
the button click function on a button element). Parent elements retain general
authority over child elements; they determine how the flow of events are
channeled, and the location and size of child elements. Simple elements are only
required to have spatial awareness within the confines provided to them -
although autonomy is still given for them to change their ordering and position
within their immediate parent element (with great power comes great
responsibility).  

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

## Stability, Upcoming Refactors, Bugs <!-- NOTE duplicate in README.md -->

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
 - Default colors everywhere are going to be replaced with a defaults in a theme
   manager. Using the Theme Manager the developer can start from a nice overall
   default then modify it to their liking. Note the Theme manager will not
   inhibit users from specifying specific colors anywhere they choose. 
 - Gradients on irregular angles are not stable, the goal is to have the
   gradient actually reflect a visual angle taking into account the width and
   the height of each cell. Currently the angles work under an assumption of
   equal cell width and height, sometimes it produces funny/unexpected results
   for a gradient which has is supposed to just be at a 45-degree angle and
   occur only once across the whole target area (`DynVal::full()`). Gradients on
   angles which are repetitive (`DynVal::fixed(..)`) work good, however the way
   the angle is interpreted will likely change to account for cell dimensions.
   Gradients on right-angles (0, 90, 180, 270 degrees) are stable.
 - Optimization: Lots of this code base has not been heavily optimized at the
   granular level, although certain effort has been made for higher level
   optimizations (printing cache, non-viewable elements are not rendered). As
   yeehaw continues to evolve there will be a greater effort put into
   optimizations, especially where visibly poor performance exists. Some
   potential improvements will include more caching to `drawing` within
   individual element implementions maybe even building in a few common caching
   patterns which arise into the `pane` object.

## Performance Considerations <!-- NOTE duplicate in README.md -->

TL;DR - Elements should cache their own drawing content, also things may be
slighly laggy while in debug mode if deeply nested containers are used.

The current design of the drawing system favours graphical flexibility over
performance. Each element must implement the `Drawing` function which in turn
returns a list of individual positions to draw (`DrawChPos`) which are relative
to itself, for each redraw a container element will then reposition all the draw
information of sub-elements. Additionally each container also processes styles
which change relative to time or position (gradients), All this reprocessing
which takes place in container elements is computationally inefficient as it
occurs with each redraw frame. The inefficiency introduced by this design
decision may lead to slightly laggy interfaces (but only) when compiled in debug
mode and if deeply nested containers are used. Release mode should never
experience noticeable lag. Use of parallel computation with rayon has been
implemented to help mitigate these inefficiencies. A complex refactor which
introduced caching at the ParentPane (specifically the organizer) level was once
attempted but found to cause more problems for Element developers and only minor
performance boosts. As such Elements are expected to cache their own drawing
information to minimize the computational burden at render time. A common caching 
pattern will soon be integrated into the `Pane` to make Element drawing
development a little bit more straightforward. 
