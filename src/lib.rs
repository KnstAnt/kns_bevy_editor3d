mod editor;
mod picking;
mod input;
mod gui;
mod camera;
mod my_macro;
mod objects;
mod render;

use bevy::app::App;
use bevy::app::Plugin;  
use bevy_mod_picking::*;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::*;
use bevy_transform_gizmo::TransformGizmoPlugin;
use bevy_debug_grid::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::editor::MyEditorPlugin;

pub struct AplicationPlugin;

impl Plugin for AplicationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WorldInspectorPlugin::new())
//            .add_plugin(DebugCursorPickingPlugin)
        //    .add_plugin(DebugEventsPickingPlugin)
            .add_plugin(TransformGizmoPlugin::default())
            .add_plugin(DebugLinesPlugin::with_depth_test(true))
            .add_plugin(DebugGridPlugin::with_floor_grid())
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_plugin(MyEditorPlugin);
    }
}
