# bevy_pen_tool
A Bevy Engine plugin for making 2D paths and smooth animations with Bezier curves

TODO:
- Mesh-making functionality for building 2D shapes and roads

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
| ![load](https://user-images.githubusercontent.com/6177048/133933736-6bed8165-fe08-4401-9bb1-e580d2f3e31a.png) | Left Control + Left Shift + S | Load set of saved curves (does not currently preserve groups or latches)|
| ![hidectrl](https://user-images.githubusercontent.com/6177048/136477042-37ec4d17-4c6c-4959-a7b8-6bde042b5401.png) | Left Control + Left Shift + H | Hide the control points |
| ![compute_lut](https://user-images.githubusercontent.com/6177048/136477061-96c02668-e44f-4e54-a92b-3f7ccd98dc6f.png) | Left Shift + T | Compute look-up table (linearizes animations) |

| ![toggle_sound](https://user-images.githubusercontent.com/6177048/133933748-4769bd96-f6c6-4863-9de5-e283f614b6f4.png) | None | Toggle sound |


## How to

1. run main.rs
2. explore and have fun
3. spawn multiple curves
4. compute the look-up tables for each curve by pressing 
    (step 4 can be repeated anytime an anchor or control point is moved)
5. latch the curves together if they are not already latched at spawn
6. moves the anchors and control points to a desired position
    (there is a "hide control points" button for when they overlap with anchors)
7. select the latched curves by clicking and dragging a selection box
8. group the curves and repeat step 4
9. save the look-up table 
10. use the look-up table in your app (see the simple_animation.rs example)


## Notes

- bevy_pen_tool does not work with a Perspective Camera (only Orthographic)
- cannot save multiple groups at once, only a single one
- currently, the plugin only works with bevy version 0.5, rev="615d43b", but this will change rather soon
- pressing load will delete everything on the canvas before loading






![bevy_pen_tool](https://user-images.githubusercontent.com/6177048/133936336-c9bc8a18-a54e-4fc6-a068-bf765d833d49.gif)

