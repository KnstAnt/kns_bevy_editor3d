use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::gui::RenderState;


pub struct RapierCreateEvent {
    pub parent: Option<Entity>,   
}

pub fn process_physics_render(
    render_state: Res<RenderState>,
    mut debug_render_context: ResMut<DebugRenderContext>
) {
    debug_render_context.enabled = render_state.collider_enabled;
}
