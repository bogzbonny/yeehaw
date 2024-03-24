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
