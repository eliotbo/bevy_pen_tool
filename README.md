# bevy_pen_tool
A Bevy Engine plugin for making 2D paths, smooth animations, meshes and roads with Bezier curves.



## Controls

| Icon | Keys | Description |
| --- | --- | --- |
| ![scale_up](https://user-images.githubusercontent.com/6177048/134087619-89ea602c-dca0-478e-8653-0dba7a50d1d5.png) | Left Control + Mousescroll up | Scale UI and curves up |
| ![scale_down](https://user-images.githubusercontent.com/6177048/134087639-8434a081-270b-49c2-a220-7eb196621c94.png) | Left Control + Mousescroll down | Scale UI and curves down |
| ![spawn](https://user-images.githubusercontent.com/6177048/133933744-aafdf2cd-9c56-4310-8704-4baa73e376b6.png) | Left Shift + Click | Spawn curve |
| ![group](https://user-images.githubusercontent.com/6177048/133933726-dd9394b8-7742-491f-88a3-43d4a06a2967.png) | Left Control + G | Group selected curves |
| ![latch](https://user-images.githubusercontent.com/6177048/133933734-41806eb3-d507-4aa9-88ec-915f60bd1dbf.png) | Left Shift + Left Control + Drag |   Latch a curve to another curve by dragging a free anchor close to another free anchor |
| ![unlatch](https://user-images.githubusercontent.com/6177048/133933752-9f935b91-c8a1-4682-98e7-7e86459dcdea.png) | Space + Drag | Unlatch anchors that were latched together. |
|![hide_anchors](https://user-images.githubusercontent.com/6177048/133933733-fd83ac0c-aadc-4028-a1fd-68c0028a8b60.png) | H | Hide anchors and control points |
|  ![save](https://user-images.githubusercontent.com/6177048/133933741-591d12c7-b7b2-4479-8f39-3da4d7a3f293.png) | Left Control + S | Save set of existing individual curves (does not currently preserve groups or latches) |
| ![load](https://user-images.githubusercontent.com/6177048/133933736-6bed8165-fe08-4401-9bb1-e580d2f3e31a.png) | Left Control + L | Load set of saved curves (does not currently preserve groups or latches)|
| ![hidectrl](https://user-images.githubusercontent.com/6177048/136477042-37ec4d17-4c6c-4959-a7b8-6bde042b5401.png) | Left Control + Left Shift + H | Hide the control points |
| ![compute_lut](https://user-images.githubusercontent.com/6177048/136477061-96c02668-e44f-4e54-a92b-3f7ccd98dc6f.png) | Left Shift + T | Compute look-up table (linearizes animations) |
| ![toggle_sound](https://user-images.githubusercontent.com/6177048/133933748-4769bd96-f6c6-4863-9de5-e283f614b6f4.png) | None | Toggle sound |
| ![bin](https://user-images.githubusercontent.com/6177048/137649706-ddac2065-3992-4f8d-b9fe-6bbf7e3cb351.png) | Select curves or group + Delete | Delete curves or group |

## Setup
Have a look at main.rs to find out how to setup the plugin.

## How to
The order of actions should be the following:
1. Spawn curves
2. Latch them together
3. Group the latched curves (cannot be ungrouped)
4. Move anchors and control points to desired position
5. Compute the look-up table
6. Save

A user can save and load the data structure for a group of Bezier curves -- called Group in the code -- in JSON format. The default directory for saving groups is "./saved/groups/", and the file extension is a custom one: ".group". Meshes can be saved in well-known ".obj" format, and their default save directory is "./saved/meshes". The one save button prompts a file dialog window for each data structure that can be saved in the current session.



## Notes
bevy_pen_tool, in its current form,
- attemps to follow Bevy's latest release
- does not work with a Perspective Camera (only Orthographic)
- cannot save multiple groups at once, only a single one
- deletes everything on the canvas before loading a group



## TODO
- saving multiple groups
- ability to move whole group
- no guarantees, but maybe a 3D version



![bevy_pen_tool](https://user-images.githubusercontent.com/6177048/133936336-c9bc8a18-a54e-4fc6-a068-bf765d833d49.gif)

