# Widgets
 - button
 - checkbox 
 - textbox
   - with scollbar(s)
 - textbox for numbers (with up/down arrows)
 - label - (simple text)
 - title (megafonts)
 - dropdown list
 - toggle (yes -> no) 
 - radio buttons      
    - optionally replace the radio characters
 - list box 
   - allow for multiline entries (ex. wimp layer) 
   - with scrollbar
 - generalized label decorators 
 - right click menus

## Planned Features
 - widget: color selector
 - widget: table (see ratatui)
 - button: visualize button being clicked
 - TGIF
 - widget: vim-style textbox
   - with two scrollbars the mode can be placed in 
     the decorations area!
 - widget: date selector
 - feature: hover comments
 - progress bar
    - optionally with an embedded word
    - gradients
 - slider bars / track bars
   ██████████████████━━━━━━━━━━━━━━   ╋   ╋ 1
                                      ┃   ┃
   ━━━━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━   ╋   ╋ 2
                                      ┃   ┃
   ██████████╋━━━━━━━━━━━━━━━━━━━━━  ▶╋   ╋ 3
                                      ┃  ▶┃
   ━━━━━━━━━━╋██████████████╋━━━━━━   ╋   ╋ 4
                                      ┃   ┃
   ══════════╪═════════════════════   ╋   ╋ 5
                                      ┃   ┃
   ══════════╪██████████████╪══════   ╋   ╋ 6 
                                      ┃   ┃
   ══════════╪▓▓▓▓▓▓▓▓▓▓▓▓▓▓╪══════   ╋   ╋ 7 

   ━━━━━━━━━● 
      ●━━━━━━━━━━━━━━━━━━━━● 

## Known Bugs
 - Scrollbar: when dragging scrollbar with mouse, will drag good for a bit then close to
   the end it just moves all the way to the maximum
 - when a right click menu is opened from within a scrollable pane, once the
   mouse hovers over the scrollable pane the scrollbar elements temporarily
   disappear. This is likely do to the rcm ctx being sent into the the
   scrollable pane and then into the scrollbar drawing function which screws
   things up. 
