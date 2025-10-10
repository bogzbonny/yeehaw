Theme Manager - it would be awesome to be able to just get the colors for each
element from a provided theme manager
 - could replicate a new trait similar to Into<Color/Style> but where the
   element KIND is actually fed in. regular colors could be provided but a 
   single theme could just be passed around manually. 
 - Alternatively maybe the theme could just be added into the Context (in
   Rc<RefCell<>> like the hat). Furthermore the color could simply be taken 
   from the theme in the default 'new' functions, then special colors could
   be applied using with decorators. 
    -  note even though the theme is in a context, an element could replace
       the theme in a new context if there was going to be a different
       sub-theme in a particular grouping of sub-elements
 - Theme should use a map of of strings for the names of theme, so that its
   fully extensible and future proof. 
 - night day themes could be accomplished by just reseting all the theme
   colors (or swapping out the entire theme object?) 
    - DONT DO Note we could also make a new Color kind which is "Color from theme for
      "button"" for example. This way the Color could be modifed at the theme
      level (day/night switch for example) and the colors would automatically
      refresh everywhere

