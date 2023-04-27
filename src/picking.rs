use bevy::prelude::*;
use bevy_mod_picking::*;
use bevy_mod_raycast::{RaycastSource, RaycastSystem};
use bevy_transform_gizmo::*;

use crate::filesystem::ObjectType;
use crate::gui::SelectState;

#[derive(Clone, Reflect)]
pub struct ObjectRaycastSet;

pub struct MyPickingPlugin;

impl Plugin for MyPickingPlugin {
    fn build(&self, app: &mut App) {
        app
            //         .add_startup_system(setup)
            .add_plugin(bevy_mod_raycast::DefaultRaycastingPlugin::<ObjectRaycastSet>::default())
            .add_system(
                update_raycast_with_cursor
                    .in_base_set(CoreSet::First)
                    .before(RaycastSystem::BuildRays::<ObjectRaycastSet>),
            )
            .add_system(process_picking_events.in_base_set(CoreSet::PostUpdate));
    }
}
/*
fn setup(
    mut commands: Commands,
    grid_query: Query<Entity, With<Grid>>
) {
//    log::info!("picking_plugin setup");

    for entity in &grid_query {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(PickingBlocker);
        }
    };
} */

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<ObjectRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    //    log::info!("update_raycast_with_cursor");

    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = bevy_mod_raycast::RaycastMethod::Screenspace(cursor_position);
    }
}

fn process_picking_events(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>,
    mut select_state: ResMut<SelectState>,
    parent_query: Query<&Parent>,
    mut object_query: Query<(&mut Selection, &mut Interaction), With<ObjectType>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => {
                //             info!("A selection event happened: {:?}", e);

                match e {
                    SelectionEvent::JustSelected(entity) => {
                        /*                         let mut current = *entity;

                        while let Ok(parent) = parent_query.get(current) {
                            current = parent.get();
                            if object_query.contains(current) {
                                break;
                            }
                        }

                        select_state.entity = Some(current.clone()); */
                    }
                    SelectionEvent::JustDeselected(entity) => {
                        if let Some(state_entity) = select_state.entity {
                            if state_entity == *entity {
                                select_state.entity = None;
                            }
                        }
                    }
                }
            }

            PickingEvent::Hover(e) => {
                //            info!("Egads! A hover event!? {:?}", e);
            }

            PickingEvent::Clicked(entity) => {
                //              info!("Gee Willikers, it's a click! {:?}", entity);

                if object_query.contains(*entity) {
                    select_state.entity = Some(entity.clone());
                    return;
                }

                let mut current = *entity;

                while let Ok(parent) = parent_query.get(current) {
                    current = parent.get();
                    if object_query.contains(current) {
                        break;
                    }
                }

                if let Ok((mut selection, mut interaction)) = object_query.get_mut(current) {
                    *interaction = Interaction::Clicked;
                    selection.set_selected(true);

                    if let Some(mut entity_commands) = commands.get_entity(*entity) {
                        entity_commands
                            .insert(Interaction::None)
                            .insert(Selection::default());
                    }

                    select_state.entity = Some(current.clone());
                }
            }
        }
    }
}
