use bevy::{prelude::*, log};
use bevy_transform_gizmo::{GizmoPickSource, GizmoTransformable};

use crate::camera::*;

use crate::objects::*;
use crate::gui::{WindowPlugin, MyEditorState, SelectState, FileState};
use crate::input::InputEvent;
use crate::input::Notify;
use crate::input::{InputPlugin, ButtonControl};
use crate::picking::*;
use crate::render::RenderPlugin;

pub struct ClearLevelEvent;


#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
enum Actions {
    #[default]
    None,
    ObjectAdd,
    ObjectRemove,
}

pub enum InputObjectType {
    Add,
    Delete,
}

pub struct InputObjectEvent {
    pub action: InputObjectType,
}

pub struct MyEditorPlugin ;

impl Plugin for MyEditorPlugin {
    fn build(&self, app: &mut App) {

        app
            .insert_resource(CameraState::new(
                5.,
                45.,
                0.,
                Vec3::new(0., 0., 0.)))
            .insert_resource(CameraConfig::default())
            .add_plugin(InputPlugin::<Actions>::default())
            .add_plugin(MyPickingPlugin)
            .add_plugin(CameraPlugin)              
            .add_plugin(ObjectPlugin)
            .add_plugin(RenderPlugin)
            .add_plugin(WindowPlugin)
            .add_event::<ClearLevelEvent>()
            .add_event::<InputObjectEvent>()    
            .add_startup_system(setup)
            .add_startup_system(setup_input)
            .add_system(process_input_events)
            .add_system(process_clear_level)
            .add_system(process_input)
            ;
    }
}

fn setup(
    mut commands: Commands,
    state: Res<CameraState>,
) {
//    log::info!("picking_plugin setup");

    let rotation = Quat::from_axis_angle(Vec3::Y, state.yaw)
                    * Quat::from_axis_angle(Vec3::X, state.pitch);        

    let translation = state.target + Transform::from_rotation(rotation).back() * state.dist;

    commands
        .spawn((Camera3dBundle {
                transform: Transform::from_translation(translation).with_rotation(rotation),
                ..Default::default()
            },
            bevy_mod_raycast::RaycastSource::<CameraMoveRaycastSet>::new_transform_empty(),
            bevy_mod_raycast::RaycastSource::<ObjectRaycastSet>::new_transform_empty(),
            bevy_transform_gizmo::GizmoPickSource::default(),
            MyCamera,
        )); 

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
}

fn setup_input(mut commands: Commands) {
    //    log::info!("setup_input");
    
        let mut button_control = ButtonControl::<Actions>::new();
    
        button_control.add_key(Actions::ObjectAdd, KeyCode::Insert, Notify::OnPress);    
        button_control.add_key(Actions::ObjectRemove, KeyCode::Delete, Notify::OnPress);
    
        commands.insert_resource(button_control);
}

fn process_input_events(
    mut reader: EventReader<InputEvent<Actions>>,
    mut object_writer: EventWriter<InputObjectEvent>,
) {
    for InputEvent::<Actions>{ name, value } in reader.iter() {
        match name {

            Actions::ObjectAdd => {
                object_writer.send(InputObjectEvent {action: InputObjectType::Add});
            },
                        
            Actions::ObjectRemove => {
                object_writer.send(InputObjectEvent {action: InputObjectType::Delete});
            },

            _ => (),
        };
    }
}

fn process_clear_level (
    mut commands: Commands,
    mut select_state: ResMut<SelectState>,
    mut file_state: ResMut<FileState>,
    mut reader: EventReader<ClearLevelEvent>,
    obj_query: Query<Entity, (With<ObjectType>, Without<Parent>)>,
) {
    if reader.is_empty() {
        return;
    }

    reader.clear();

    file_state.current_file_path = None; 

    select_state.entity = None;

    log::info!("process_clear_level");

    for entity in obj_query.iter() {
        if let Some(entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn_recursive();  
        } 
    }
}

fn process_input (
    mut commands: Commands,
    mut select_state: ResMut<SelectState>,
    mut editor_state: ResMut<MyEditorState>,
    camera_state: Res<CameraState>,
    pick_query: Query<&bevy_mod_raycast::RaycastSource<ObjectRaycastSet>>,
    transform_query: Query<&GlobalTransform>,
    mut reader: EventReader<InputObjectEvent>,
    mut add_writer: EventWriter<AddObjectEvent>,
) {
    for InputObjectEvent {action} in reader.into_iter() {
        log::info!("process_input");

        match action {
            InputObjectType::Add => {

                let mut transform = None;

                for pick_source in &pick_query {
                    log::info!("process_input iter");
                    if let Some((_entity, intersection)) = pick_source.get_nearest_intersection() {
                        log::info!("process_input transform ok");
                        transform = Some(Transform::from_translation(intersection.position()));
                        break;
                    }                            
                }

                if transform.is_none() {
                    transform = Some(Transform::from_translation(camera_state.camera_pos + camera_state.mouse_ray * camera_state.dist));
                }

                let entity = commands.spawn_empty().id();

                if select_state.set_child {
                    if let Some(parent_entity) = select_state.entity {
                        if let Ok(global_transform) = transform_query.get(parent_entity) {
                            transform = Some( GlobalTransform::from(transform.unwrap()).reparented_to(global_transform));
                        }

                        if let Some(mut entity_commands) = commands.get_entity(parent_entity) {
                            entity_commands.add_child(entity);
                        } 
                    }               
                }

                let collider = if select_state.generate_collider {
                    None
                } else {
                    None
                };

                add_writer.send(AddObjectEvent {
                    entity: Some(entity),
                    object: editor_state.selected_object.clone(),
                    collider,
                    transform,
                    selected: true,
                });
            },

            InputObjectType::Delete => {
                if let Some(entity) = select_state.entity {
                    if let Some(entity_commands) = commands.get_entity(entity) {
                        entity_commands.despawn_recursive();
                    } 

                    select_state.entity = None;
                };
            },
        }
    } 
}


 