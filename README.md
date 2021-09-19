# bevy_pen_tool
A Bevy Engine plugin for making 2D paths and smooth animations with Bezier curves

TODO:
- Save groups and generate a high-quality look-up-table for grouped curves
- Mesh-making functionality for building 2D shapes and roads

# Controls

| Icon | Keys | Description |
| --- | --- | --- |
| ![spawn](https://user-images.githubusercontent.com/6177048/133933744-aafdf2cd-9c56-4310-8704-4baa73e376b6.png) | Left Shift + Click | Spawn curve |
| 
![group](https://user-images.githubusercontent.com/6177048/133933726-dd9394b8-7742-491f-88a3-43d4a06a2967.png) | Left Control + G | 
Group selected curves |
|
![select](https://user-images.githubusercontent.com/6177048/133933742-63a11995-ceee-4747-8910-e0210a4fc277.png) | Left Control | Select curves by clicking on either its start anchor or its end anchor |
| ![latch](https://user-images.githubusercontent.com/6177048/133933734-41806eb3-d507-4aa9-88ec-915f60bd1dbf.png) | Left Shift + Left Control |   Latch a curve to another curve by dragging a free anchor close to another free anchor |

| ![unlatch](https://user-images.githubusercontent.com/6177048/133933752-9f935b91-c8a1-4682-98e7-7e86459dcdea.png) | Space | Unlatch anchors that were latched together. |

|![hide_anchors](https://user-images.githubusercontent.com/6177048/133933733-fd83ac0c-aadc-4028-a1fd-68c0028a8b60.png) | H | Hide anchors and control points |
|  ![save](https://user-images.githubusercontent.com/6177048/133933741-591d12c7-b7b2-4479-8f39-3da4d7a3f293.png) | Left Control + S | Save set of existing individual curves (does not currently preserve groups or latches) |
| ![load](https://user-images.githubusercontent.com/6177048/133933736-6bed8165-fe08-4401-9bb1-e580d2f3e31a.png) | Left Control + Left Shift + S | Load set of saved curves (does not currently preserve groups or latches)|

|![undo](https://user-images.githubusercontent.com/6177048/133933750-47820fb4-8e1b-4a57-aa4a-e60fa3bee66c.png) | Left Control + Z | Delete the curve that was spawned last (does not keep track of anchor and control point movement) |

| ![redo](https://user-images.githubusercontent.com/6177048/133933739-a72e308d-c2d7-4ecc-a9cc-daf0b19fa0d6.png) | 
Left Control + Left Shift + Z | Respawn the curve that was deleted last with "undo" |

| 
![toggle_sound](https://user-images.githubusercontent.com/6177048/133933748-4769bd96-f6c6-4863-9de5-e283f614b6f4.png) | None | Toggle sound |






![bevy_pen_tool](https://user-images.githubusercontent.com/6177048/133935702-ac72da4f-4470-4c8d-ab14-ab9724f1b201.gif)




