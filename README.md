# bevy_pen_tool
A Bevy Engine plugin for making 2D paths, smooth animations, meshes and roads with Bezier curves.















## Controls

| Icon | Keys | Description |
| --- | --- | --- |
| ![scale_up](https://user-images.githubusercontent.com/6177048/137652103-02a4b95b-61de-491a-92da-3ef74cf97498.png) | Left Control + Mousescroll up | Scale UI and curves up |
| ![scale_down](https://user-images.githubusercontent.com/6177048/137652111-6d3e13aa-bca9-40d5-9a06-222cad9c23bf.png) | Left Control + Mousescroll down | Scale UI and curves down |
| ![spawn_curve](https://user-images.githubusercontent.com/6177048/137652140-605744c5-e9a0-4c8d-ad8a-1c47dcb7db7c.png) | Left Shift + Click | Spawn curve |
| ![group](https://user-images.githubusercontent.com/6177048/137652145-adb487b7-c45d-4aa5-8a20-ddd45829dc2d.png) | Left Control + G | Group selected curves |
| ![latch](https://user-images.githubusercontent.com/6177048/137652149-a604ed8d-83bb-4d2d-973c-05658c12ae6b.png) | Left Shift + Left Control + Drag |   Latch a curve to another curve by dragging a free anchor close to another free anchor |
| ![unlatch](https://user-images.githubusercontent.com/6177048/137652201-3a6880c3-c149-4ff1-bc27-b8132bf52fc2.png) | Space + Drag | Unlatch anchors that were latched together. |
| ![hide_anchors](https://user-images.githubusercontent.com/6177048/137652205-d915eb15-88ea-45da-92a0-3d3680a56ea1.png) | H | Hide anchors and control points |
| ![save](https://user-images.githubusercontent.com/6177048/137652208-a7d843b7-6adc-414b-b0d7-126afd4f809f.png)  | Left Control + S | Save set of existing individual curves (does not currently preserve groups or latches) |
| ![load](https://user-images.githubusercontent.com/6177048/137652246-69c1309e-2486-496c-acbc-852a255476d2.png) | Left Control + L | Load set of saved curves (does not currently preserve groups or latches)|
| ![hide_ctrls](https://user-images.githubusercontent.com/6177048/137652249-81669e44-42b8-4775-afe5-071c248713ef.png) | Left Control + Left Shift + H | Hide the control points |
| ![lut](https://user-images.githubusercontent.com/6177048/137652254-f62c0d1b-d323-4ec6-b51f-c86b3f21f390.png) | Left Shift + T | Compute look-up table (linearizes animations) |
| ![sound](https://user-images.githubusercontent.com/6177048/137652277-c43ace61-723b-409b-b48b-5521238c5e4d.png) | None | Toggle sound |
| ![bin](https://user-images.githubusercontent.com/6177048/137652281-a461da81-bbd0-4728-a80f-7bb19849a149.png) | Select curves or group + Delete | Delete curves or group |
| ![road](https://user-images.githubusercontent.com/6177048/137652369-0bd832a9-9c03-42a3-9dc6-b840f45c86dd.png) | None | Spawn road on curve group |
| ![mesh](https://user-images.githubusercontent.com/6177048/137652366-ffc53243-0df9-4e84-a0ab-3985c3c59302.png) | None | Spawn mesh inside curve group |
| ![heli](https://user-images.githubusercontent.com/6177048/137652364-67eedf2b-8283-43b0-a2e6-e80e97f5cb89.png) | None | Spawn animated car on curve group |








## Setup
Clone the repo, copy and paste the crate called "bevy_pen_tool_plugin" from the "crates" folder in the repo to the directory for your project, and add "bevy_pen_tool_plugin" as a local dependency in Cargo.toml. Refer to main.rs and Cargo.toml for futher details.

## How to
A typical sequence of actions using the plugin would be as follows:
1. Spawn curves
2. Latch them together
3. Group the latched curves (cannot be ungrouped)
4. Move anchors and control points to desired position
5. Compute the look-up table
6. Save

A user can save and load the data structure for a group of Bezier curves -- called Group in the code -- in JSON format. The default directory for saving groups is "./saved/groups/", and the file extension is a custom one: ".group". Meshes can be saved in well-known ".obj" format, and their default save directory is "./saved/meshes". The one save button prompts a file dialog window for each data structure that can be saved in the current session.

There are two important parameters to tweak and they are both located in a Resource called "Globals"
1. group_lut_num_points: the number of elements in the generated look-up table (more yields smoother animations/meshes)
2. road_width: the width of the road meshes
See main.rs to see how to modify these parameters.



## Notes
bevy_pen_tool, in its current form,
- attemps to follow Bevy's latest release
- does not work with a Perspective Camera (only Orthographic)
- cannot save multiple groups at once, only a single one
- deletes everything on the canvas before loading a group of Bezier curves



## TODO
- saving multiple groups
- ability to move whole group
- ruler tool
- no guarantees, but maybe a 3D version



![bevy_pen_tool](https://user-images.githubusercontent.com/6177048/133936336-c9bc8a18-a54e-4fc6-a068-bf765d833d49.gif)

