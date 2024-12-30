
🚧 this doc is under construction 🚧  


 - The DrawRegion is only provided during drawing and not included within the
   context throughout in order to make it easier to send events/ call functions
   on an element from another event. If we required sending the DrawRegion into 
   the element for these functions, it would make life difficult for the
   developer having to calculate the draw region for an Element from an
   arbitrary other element


## Event Routing

 -  Events which are not "captured" keep getting routed to other elements.

```
                                                                                             
   KEYBOARD EVENT                                                                            
                                                                                             
    TUI                                                                                      
    - relays events to elements                                                               
    - responsible for the top element                                                        
                                                                                             
                                                                                             
                                                                                             
                 ┌────────────────────────────────────────┐                                  
                 │ Element                                │◀──────────────┐                  
                 │  - Receives Events                     │               │                 
                 │  - Provides drawing details on request │               │                     
                 │                                        │               │                  
                 │                   ┌──────────────────┐ │               │                   
                 │                   │                  ├─┼─ Sub-Elements─┘                  
                 │                   │                  │ │                                      
                 │                   │  Sub-Element     ├─┼─ Sub-Elements' Contexts
                 │                   │  Organizer       │ │   
                 │                   │                  ├─┼─  
                 │                   │                  │ │   
                 │                   │  EventPrioritizer├─┼─                          
                 │                   └──────────────────┘ │                                  
                 └────────────────────────────────────────┘                                  
                                                                                             
```

WITH GREAT POWER COMES GREAT RESPONSIBILITY 

THINKING
 - partially autonomous element model. (suzerainty?)
   - the local loc/visibility is controlled by the element
     - this is not the abs location, only the location within the immediate
       context.
   - this introduces a bit of confusion with regards to mouse event positions. 
     - mouse position events are local (upper right is 0, 0) 
     - could create a new position type and send that in with the crossterm mouse
       event 
