
`yeehaw` is a batteries included TUI framework with a rendering strategy, full
event routing system and a plethora of elements included.  

## Quickstart:   <!-- NOTE duplicate in README.md:51 -->

A hello world example with a label and a reactive button:

<!-- NOTE do not run as crossterm panics in doc run mode -->
``` no_run
use yeehaw::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut tui, ctx) = Tui::new()?;
    let main_el = ParentPane::new(&ctx, "main_element");

    // place the label at 30% of the screen width and height
    let label = Label::new(&ctx, "Hello, World!").at(0.3, 0.3);

    // place the button 1 character below the label
    let x = DynVal::new_flex(0.3); // 30% screen width
    let y = DynVal::new_flex(0.3).plus(1.into()); // 30% screen height + 1 ch

    let label_ = label.clone(); // clone for closure move
    let button = Button::new(&ctx, "Click Here!")
        .with_fn(Box::new(move |_, _| {
            label_.set_text("Button clicked!".to_string());
            EventResponses::default()
        }))
        .at(x, y);

    main_el.add_element(Box::new(label));
    main_el.add_element(Box::new(button));
    tui.run(Box::new(main_el)).await
}
```

## Existing Elements:  <!-- NOTE duplicate in README.md:84 -->

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

## Cool Features
 - mini-TUIs in the CLI (aka. use a TUI in-line with your command without taking
   up the whole terminal)

## Planned Stuff: <!-- NOTE duplicate in README.md:118 --> 
 - embed a whole dang yeehaw TUI into stateful ratatui widget, why not!
 - accordion stack container
 - hover comments for elements
 - vertical tabs (like brave browser) 
 - \[.ans\]-animation player (using an extended-ansi format)
 - optional mouse **pixel** support (there's already mouse support)
 - wire-connectors between elements
 - color selector element
 - table element
 - an interactive debugging application for yeehaw TUIs
 - TUI Snapshot Tester
 - drag and drop TUI application builder (as a TUI of course)
 - build out a system of feature-flags for feature restriction / compile time
   improvement.
 - a sync version of the TUI for those who don't want async

# Design Overview <!-- NOTE duplicate in README.md:135 -->

Elements are arranged in a hierarchical manner while retaining semi-autonomy.
Events (keyboard/mouse/custom) are routed from the top down (adjusted to be
relative to the each destination Element) and responses then propagate back
upwards along the Element ownership tree. Parent elements retain general
authority over child elements; they determine how the flow of events are
channeled and the location and size of child elements. Simple elements are only
required to have spatial awareness within the confines provided to them (events
are made relative to them by their parents) - although autonomy is still given
for them to change their ordering and position within their immediate parent
element (with great power comes great responsibility).  

In addition to the event-response communication structure elements may directly
interact through element-specific hooks (e.g. the button click function on a
button element). 

The core Element Trait has designed to be extensible for custom event/response
kinds enabling developers to create entirely new sub-classes of elements which
can reuse the event routing system logic. 

Looking to understand more? Checkout:
 - [Examples](https://github.com/bogzbonny/yeehaw/tree/main/yeehaw/examples)
 - [Element Trait](crate::element::Element)
 - [Pane](crate::elements::Pane) <- the standard base for all built in elements
 - [ParentPane](crate::elements::ParentPane) <- the standard base for all elements which hold other elements
 - [Context](crate::context::Context) <- an object which can be found nearly everywhere
 - [DynVal](crate::dyn_value::DynVal) <- the basis for all layouts and sizes

## Stability, Upcoming Refactors, Bugs <!-- NOTE duplicate in README.md:192 -->

If you plan on building/experimenting with yeehaw right now, that's great news!
I'd like to keep you apprised of some upcoming changes. If you do wish to
experiment and or start development on yeehaw I wouldn't be too afraid of these
upcoming changes, the majority of foreseeable major refactors have already been
completed.  While yeehaw is pre-1.0.0 all breaking changes will bd semver minor
version upgrades - in other words I don't plan on providing patch updates for bug
fixes for minor versions.

I'll try'n help out anyone who needs a hand understanding how to update their
code if its been broken by a new release. Additionally a breaking changes doc
with upgrade instructions shall be maintained. 

HAVE NO FEAR

 - The terminal widget consumes a lot of cpu because its constantly
   recalculating (non-intelligently). This is going to be refactored and should
   drastically reduce it's CPU usage. 
 - There is a planned migration of the backend of the Terminal from vt100 to
   wezterm-term + termwiz. This should have no effect on downstream users.
 - The menu-bar needs a basic refactor to reduce cpu usage, it's constantly
   recalculating each update.
 - There ain't much automated testing in here at all, soon a TUI snapshot tester
   is going to be developed, which should bring up coverage from about 0% as it
   stands (yikes!). 
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
   occur only once across the whole target area (`DynVal::FULL`). Gradients on
   angles which are repetitive (`DynVal::fixed(..)`) work good, however the way
   the angle is interpreted will likely change to account for cell dimensions.
   Gradients on right-angles (0, 90, 180, 270 degrees) are stable.
 - All the fancy color types (gradients, patterns) will likely get refactored
   and lumped together into a common Trait whereby entirely new classes of
   colors can be created by developers. 

## Performance Considerations <!-- NOTE duplicate in README.md:242 -->

TL;DR - Elements should provide drawing updates only when actually necessary,
also things may be slightly laggy while in debug mode if deeply nested
containers are used. Also currently time-base gradients trigger constant
recalculation and will substantially increase cpu usage.

The current design of the drawing system favours graphical flexibility over
performance. Each element must implement the `Drawing` function which in turn
returns `DrawUpdate`'s which can list of individual cell updates. For each
redraw, container elements will then reposition all the draw information of
their respective sub-elements. All this reprocessing which takes place in
container elements is computationally inefficient as it occurs with each redraw
cycle where changes occur. The inefficiency introduced by the current design may
lead to slightly laggy interfaces but primarily only when compiled in debug mode
and if deeply nested containers are used AND if those elements are providing
continuous drawing updates. Use of parallel computation with rayon has been
implemented to help mitigate these inefficiencies. In conclusion, to minimize
the computational burden at render time, Elements should track their own state
and only respond with drawing updates (in `fn drawing(..)`)  when their are
actual state changes which warrant redrawing.

The time based gradient design currently will cause ample CPU recalculations,
this will hopefully be improved with time (GPU), but just be aware that if you
want to use those sweet sweet time gradient colors it will cost you. If you
choose not to use time gradients you will be unaffected by these CPU drains.  
