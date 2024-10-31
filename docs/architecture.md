```
                                                                                             
   KEYBOARD EVENT                                                                            
                                                                                             
    CUI                                                                                      
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
                 │                   │  Organizer       │ │   ├─Visibility                    
                 │                   │                  ├─┼─  ├─Location          
                 │                   │                  │ │   └─Priority          
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

 - DynLocation change hooks?
   - elements could setup interdependancies around scaling
   - how to deal with inf.recurrsion?? NOT AN ISSUE
     - if two elements want to have a common edge.
       - eg. element 1 shifts, triggering element 2 which shifts, which triggers
         element 1 which doesn't change as it's already in the correct position. 
 - DONT DO Move DynLocationSet back to the responses?
   - would need to first create widget builders which actually need the location
