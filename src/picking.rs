use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, PointerEvent};
use bevy_mod_picking::selection::{Deselect, PickSelection, Select};
use bevy_mod_picking::*;
use bevy_mod_raycast::{RaycastSource, RaycastSystem};

use crate::gui::SelectState;
use crate::objects::ObjectType;

#[derive(Clone, Reflect)]
pub struct ObjectRaycastSet;

pub struct MyPickingPlugin;

pub struct PickingEvent {
    pub entity: Entity,
}

impl Plugin for MyPickingPlugin {
    fn build(&self, app: &mut App) {
        app
            //         .add_startup_system(setup)
            .add_plugins(DefaultPickingPlugins)
            .add_event::<PickingEvent>()
            .add_event::<PointerSelectEventWaiter>()
            .add_system(
                update_raycast_with_cursor
                    .in_base_set(CoreSet::First)
                    .before(RaycastSystem::BuildRays::<ObjectRaycastSet>),
            )
            .add_systems((
                await_select_event.before(process_select_event),
                process_select_event,
                process_deselect_event,
                process_click_event,
                process_picking_events,                
            ));
    }
}

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

fn process_select_event(
    mut commands: Commands,
    mut events: EventReader<PointerEvent<Select>>,
    mut select_state: ResMut<SelectState>,
    parent_query: Query<&Parent>,
    object_query: Query<Option<&PickSelection>, With<ObjectType>>,
    mut pick_query: Query<&mut PickSelection, Without<ObjectType>>,
    mut selections: EventWriter<PointerSelectEventWaiter>,
    mut deselections: EventWriter<PointerEvent<Deselect>>,
) {
    for PointerEvent::<Select> {
        pointer_id,
        pointer_location,
        target,
        event,
    } in events.iter()
    {
        info!("select! {:?}", target);

        if object_query.contains(*target) {
            select_state.entity = Some(target.clone());
            return;
        }

        let mut current = *target;

        while let Ok(parent) = parent_query.get(current) {
            current = parent.get();
            if object_query.contains(current) {
                break;
            }
        }

        if current != *target {
            deselections.send(PointerEvent::new(
                *pointer_id,
                pointer_location.to_owned(),
                *target,
                Deselect,
            ));

            selections.send( PointerSelectEventWaiter { event: PointerEvent::new(
                *pointer_id,
                pointer_location.to_owned(),
                current,
                Select,
            ) } );

            //          select_state.entity = Some(current.clone());
            // entity_commands.insert(PickSelection {is_selected: true});
        }

        info!("select end {:?} {:?}", target, current);

        select_state.entity = Some(current.clone());
    }
}

struct PointerSelectEventWaiter {
    event: PointerEvent<Select>,
}
fn await_select_event(
    mut reader: EventReader<PointerSelectEventWaiter>,
    mut writer: EventWriter<PointerEvent<Select>>,
) {
    for PointerSelectEventWaiter { event } in reader.iter() {
        writer.send( event.to_owned() );
    }
}
fn process_deselect_event(
    mut events: EventReader<PointerEvent<Deselect>>,
    mut select_state: ResMut<SelectState>,
) {
    for PointerEvent::<Deselect> {
        pointer_id,
        pointer_location,
        target,
        event,
    } in events.iter()
    {
        info!("deselect! {:?}", target);

        if let Some(state_entity) = select_state.entity {
            //          if state_entity == *target {
            select_state.entity = None;
            //          }
        }
    }
}

fn process_click_event(
    mut commands: Commands,
    mut events: EventReader<PointerEvent<Click>>,
    mut select_state: ResMut<SelectState>,
    parent_query: Query<&Parent>,
    mut object_query: Query<Option<&mut PickSelection>, With<ObjectType>>,
    mut pick_query: Query<&mut PickSelection, Without<ObjectType>>,
    mut selections: EventWriter<PointerEvent<Select>>,
    mut deselections: EventWriter<PointerEvent<Deselect>>,
) {
    for PointerEvent::<Click> {
        pointer_id,
        pointer_location,
        target,
        event,
    } in events.iter()
    {
        info!("click! {:?}", target);

        /*         if object_query.contains(*target) {
                    select_state.entity = Some(target.clone());

                    info!("click  1");

                    return;
                }

                let mut current = *target;

                while let Ok(parent) = parent_query.get(current) {
                    current = parent.get();
                    if object_query.contains(current) {
                        info!("click  2");
                        break;
                    }
                }

                if current != *target {
                    info!("click  3");

                    if let Some(mut entity_commands) = commands.get_entity(*target) {
                        info!("click  4");
                     //   entity_commands.insert(PickSelection::default());
                        deselections.send(PointerEvent::new(
                            *pointer_id,
                            pointer_location.to_owned(),
                            *target,
                            Deselect,
                        ));
                    }

        /*             for mut pick_selection in pick_query.iter_mut() {
                        if pick_selection.is_selected {
                            pick_selection.is_selected = false;
                        }
                    } */

                    if let Some(mut entity_commands) = commands.get_entity(current) {
                        info!("click  5");
                        selections.send(PointerEvent::new(
                            *pointer_id,
                            pointer_location.to_owned(),
                            current,
                            Select,
                        ));

              //          select_state.entity = Some(current.clone());
                       // entity_commands.insert(PickSelection {is_selected: true});
                    }
                }

                info!("click end {:?} {:?}", target, current);

          //      select_state.entity = Some(current.clone()); */
    }
}

fn process_picking_events(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>,
    mut select_state: ResMut<SelectState>,
    mut object_query: Query<&mut PickSelection, With<ObjectType>>,
) {
    if let Some(PickingEvent { entity }) = events.iter().last() {
        info!("picking! entity: {:?}", entity);

        if let Some(mut entity_commands) = commands.get_entity(*entity) {
            for mut pick_selection in object_query.iter_mut() {
                if pick_selection.is_selected {
                    pick_selection.is_selected = false;
                }
            }

            entity_commands.insert(PickSelection { is_selected: true });
            select_state.entity = Some(*entity);
        }
    }
}

/*
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
                            .insert(Selection::default())
                            ;
                    }

                    select_state.entity = Some(current.clone());
                }
            }
        }
    }
}
 */
