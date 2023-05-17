use bevy::window::PrimaryWindow;
use bevy::{ecs::component::Component, input::mouse::MouseMotion};
use bevy::{prelude::*, log};
use bevy_mod_raycast::{ RaycastSource, RaycastSystem, };
use bevy_transform_gizmo::PickingBlocker;

use crate::input::*;
use crate::picking::ObjectRaycastSet;
use crate::{if_err_return, if_none_return};


enum CameraEventState {
    Start,
    Stop,
}

struct CameraZoomEvent {
    pub value: f32,
}

struct CameraMoveEvent {
    pub state: CameraEventState,
}

struct CameraRotateEvent {
    pub state: CameraEventState,
}

 struct CameraDirEvent {
    pub origin: Vec3,
    pub dir: Vec3,
} 

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
enum Actions {
    #[default]
    None,

    CameraZoom,
    CameraMoveStart,
    CameraMoveStop,
    CameraRotateStart,
    CameraRotateStop,
}

#[derive(Clone, Reflect)]
pub struct CameraMoveRaycastSet;


#[derive(Component, Debug)]
pub struct MyCamera;


#[derive(Debug, Resource)]
pub struct CameraConfig {
    pub up_fixed: bool,
    pub can_rotate: bool,
    pub rotate_speed_x: f32,
    pub rotate_speed_y: f32,
    pub screen_dist_x: f32,
    pub screen_dist_y: f32,
    pub scroll_speed: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            up_fixed: false,
            can_rotate: true,
            rotate_speed_x: 0.5*std::f32::consts::PI/180.,
            rotate_speed_y: 0.1*std::f32::consts::PI/180.,
            screen_dist_x: 50.,
            screen_dist_y: 50.,
            scroll_speed: 0.05,
            pitch_min: -1.5,
            pitch_max: -0.10,
        }
    }
}

#[derive(Debug, Resource)]
pub struct CameraState {
    pub dist: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub target: Vec3,
    pub mouse_ray: Vec3,
    pub camera_pos: Vec3,
    pub cursor_pos: Vec2,
    move_start_pos: Option<Vec3>,
    rotate_start_ray: Option<Vec3>,
}

impl CameraState {
    pub fn new(
        dist: f32,
        pitch: f32,
        yaw: f32,
        target: Vec3,
    ) -> Self {
        let mut res = Self::default();

        res.dist = dist;
        res.pitch = -pitch * std::f32::consts::PI / 180.;
        res.yaw = yaw * std::f32::consts::PI / 180.;
        res.target = target;

        res
    }
}

impl Default for CameraState {
        fn default() -> Self {
        Self {
            dist: 100.0,
            pitch: -1.,
            yaw: 0.,
            mouse_ray: Vec3::new(0., -1., 0.),
            camera_pos: Vec3::new(0., 0., 0.),
            cursor_pos: Vec2::new(0., 0.),
            target: Vec3::ZERO,
            move_start_pos: None,
            rotate_start_ray: None,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut App) {
        app
            .add_plugin(InputPlugin::<Actions>::default())
            .add_event::<CameraZoomEvent>()
            .add_event::<CameraMoveEvent>()
            .add_event::<CameraRotateEvent>()
            .add_event::<CameraDirEvent>()
            .add_plugin(bevy_mod_raycast::DefaultRaycastingPlugin::<CameraMoveRaycastSet>::default())
            .add_plugin(bevy_mod_raycast::DefaultRaycastingPlugin::<ObjectRaycastSet>::default())
            .add_startup_system(setup)
            .add_system(
                update_raycast_with_cursor
                    .in_base_set(CoreSet::First)
                    .before(RaycastSystem::BuildRays::<CameraMoveRaycastSet>)
                    .before(RaycastSystem::BuildRays::<ObjectRaycastSet>),
            ) 
             .add_systems((
                process_input_events.before(process_zoom_event),
                process_zoom_event.before(process_move_event),
                process_move_event.before(process_cursor_move_event),
                process_rotate_event.before(process_cursor_change_event),                            
                process_cursor_move_event.before(process_camera),   
                process_cursor_change_event.before(process_camera),   
                process_camera.run_if(resource_exists::<CameraState>().and_then(
                    |state: Res<CameraState>| state.is_changed(),
                )),
            )) 
            ;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
 //   log::info!("CameraPlugin setup");
    let mut button_control = ButtonControl::<Actions>::new();

    button_control.add_action(Actions::CameraZoom, InputAction::Scroll, Notify::OnActive);

    button_control.add_mouse(Actions::CameraMoveStart, MouseButton::Right, Notify::OnPress);
    button_control.add_mouse(Actions::CameraMoveStop, MouseButton::Right, Notify::OnRelease);

    button_control.add_key(Actions::CameraRotateStart, KeyCode::LControl, Notify::OnPress);    
    button_control.add_key(Actions::CameraRotateStop, KeyCode::LControl, Notify::OnRelease);


    commands.insert_resource(button_control);

    commands.insert_resource(bevy_mod_raycast::DefaultPluginState::<CameraMoveRaycastSet>::default());
    commands.insert_resource(bevy_mod_raycast::DefaultPluginState::<ObjectRaycastSet>::default());

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100000., subdivisions: 0 })),
        material: materials.add(Color::rgba(0., 0., 0., 0.).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default() 
    })
    .insert(bevy_mod_raycast::RaycastMesh::<CameraMoveRaycastSet>::default())
    .insert(bevy_mod_raycast::RaycastMesh::<ObjectRaycastSet>::default())
 //   .insert(PickingBlocker)
    ; 
}

fn process_input_events(
    mut reader: EventReader<InputEvent<Actions>>,
    mut zoom_writer: EventWriter<CameraZoomEvent>,
    mut move_writer: EventWriter<CameraMoveEvent>,
    mut rotate_writer: EventWriter<CameraRotateEvent>,
) {
    // log::info!("process_input_events Start");
    for InputEvent::<Actions>{ name, value } in reader.iter() {
        match name {
            Actions::CameraZoom => {
   //             log::info!("process_input_events CameraZoom");
                zoom_writer.send(CameraZoomEvent {value: *value as f32});
            },
            
            Actions::CameraMoveStart => {
    //            log::info!("process_input_events CameraMoveStart");
                move_writer.send(CameraMoveEvent {state: CameraEventState::Start});
            },
            
            Actions::CameraMoveStop => {
    //            log::info!("process_input_events CameraMoveStop");
                move_writer.send(CameraMoveEvent {state: CameraEventState::Stop});
            },

            Actions::CameraRotateStart => {
    //            log::info!("process_input_events CameraRotateStart");
                rotate_writer.send(CameraRotateEvent {state: CameraEventState::Start});
            },
            
            Actions::CameraRotateStop => {
    //            log::info!("process_input_events CameraRotateStop");
                rotate_writer.send(CameraRotateEvent {state: CameraEventState::Stop});
            },

            Actions::None => (),
        };
    }
}

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query_cam: Query<&mut RaycastSource<CameraMoveRaycastSet>>,
    mut query_obj: Query<&mut RaycastSource<ObjectRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
//    log::info!("update_raycast_with_cursor");

    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query_cam {        
 //       log::info!("update_raycast_with_cursor camera ok");
        pick_source.cast_method = bevy_mod_raycast::RaycastMethod::Screenspace(cursor_position);
    }

    for mut pick_source in &mut query_obj { 
 //       log::info!("update_raycast_with_cursor object ok");       
        pick_source.cast_method = bevy_mod_raycast::RaycastMethod::Screenspace(cursor_position);
    } 
}

fn process_cursor_move_event(
    mut cursor: EventReader<CursorMoved>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    ground_query: Query<&RaycastSource<CameraMoveRaycastSet>>,
    mut state: ResMut<CameraState>,
    camera_global_query: Query<(&Camera, &GlobalTransform), With<MyCamera>>,
    mut dir_writer: EventWriter<CameraDirEvent>,
) {
    state.cursor_pos = if let Some(cursor_latest) = cursor.iter().last() {
        cursor_latest.position
    } else {
        return;
    };

    let window = if_err_return!(primary_query.get_single());

    let (camera, camera_global_transform) = if_err_return!(camera_global_query.get_single());
    let cam_pos = camera_global_transform.translation();

    let mouse_ray = screen_to_world_dir(
        &state.cursor_pos,
        &Vec2::from([window.width() as f32, window.height() as f32]),
        camera,
        camera_global_transform,
    );

    state.mouse_ray = mouse_ray;

    dir_writer.send(CameraDirEvent{
        origin: cam_pos,
        dir: mouse_ray,
    });

    let move_start_pos = if_none_return!(state.move_start_pos);

 //   log::info!("process_cursor_move_event");

     for pick_source in &ground_query {
  //      log::info!("process_cursor_move_event iter");

        if let Some((_entity, intersection)) = pick_source.get_nearest_intersection() {

  //          log::info!("process_cursor_move_event intersection ok");

            state.target = state.target + move_start_pos - intersection.position();

            break; 
        }                            
    }

 //   log::info!("process_move_event start start_pos: {:?}, target: {:?}", state.move_start_pos, state.target); 

    // log::info!("process_move_event start start_pos: {:?}, current_pos: {:?}, delta_pos: {:?}, target: {:?}", 
    //    state.move_start_pos, current_pos, delta_pos, state.target);
}

fn process_cursor_change_event(
    mut cursor: EventReader<MouseMotion>,
    mut state: ResMut<CameraState>,
    config: Res<CameraConfig>, 
    primary_query: Query<&Window, With<PrimaryWindow>>,
) {
    if !config.can_rotate {
        return;
    }

    let _rotate_start_ray = if_none_return!(state.rotate_start_ray);

    let window = if_err_return!(primary_query.get_single());

    let half_y = window.height() as f32 / 2.;

    for &MouseMotion { delta } in cursor.iter() {
        state.yaw -= delta.x * config.rotate_speed_x;// * ((half_y - state.cursor_pos.y)/half_y);
        
        state.pitch -= delta.y * config.rotate_speed_y;
        state.pitch = state.pitch.clamp(config.pitch_min, config.pitch_max);    
    }
}

fn process_zoom_event(
    mut reader: EventReader<CameraZoomEvent>,      
    config: Res<CameraConfig>,   
    mut state: ResMut<CameraState>,   
) {
    if reader.is_empty() {
        return;
    }

    for &CameraZoomEvent { value } in reader.iter() {
        state.dist -= state.dist * config.scroll_speed * value;        
    }
}

fn process_rotate_event(
    mut reader: EventReader<CameraRotateEvent>,   
    mut state: ResMut<CameraState>,   
) {
 //   log::info!("process_rotate_event");

    for event in reader.iter() {      
        state.rotate_start_ray = match event.state {
            CameraEventState::Start => Some(state.mouse_ray.clone()),
            CameraEventState::Stop => None,
        };
    } 
} 

fn process_move_event(
    mut reader: EventReader<CameraMoveEvent>,
    mut state: ResMut<CameraState>,  
    ground_query: Query<&RaycastSource<CameraMoveRaycastSet>>,
) {
 //       log::info!("process_move_event");

     for event in reader.iter() {
        match event.state {
            CameraEventState::Start => {
 //               log::info!("process_move_event Start");
                for raycast_source  in &ground_query {
 //                   log::info!("process_move_event iter");


                    if let Some((_entity, intersection)) = raycast_source.get_nearest_intersection() {
                        state.move_start_pos = Some(intersection.position());  
      //                  log::info!("process_move_event pos ok {:?}", state.move_start_pos);
                        break; 
                    }    
                }
            },
            CameraEventState::Stop => state.move_start_pos = None,
        };

 //       log::info!("process_move_event res pos {:?}", state.move_start_pos);
    } 
}

fn process_camera(
    mut state: ResMut<CameraState>,
    mut query: Query<&mut Transform, With<MyCamera>>,
) {

    let mut transform = if_err_return!(query.get_single_mut());

    transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw)
                    * Quat::from_axis_angle(Vec3::X, state.pitch);        

    transform.translation = state.target + transform.back() * state.dist;

    state.camera_pos = transform.translation;
}


pub fn screen_to_world_dir(
    mouse_position: &Vec2,
    screen_size: &Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec3 {
    let view = camera_transform.compute_matrix();

    let projection = camera.projection_matrix();
    let cursor_ndc = Vec2::new(
        2.0 * (mouse_position.x / screen_size.x) - 1.0,
        2.0 * (mouse_position.y / screen_size.y) - 1.0,
    );

    let ndc_to_world: Mat4 = view * projection.inverse();
    let world_to_ndc = projection * view;

    let projection = projection.to_cols_array_2d();
    let camera_near = (2.0 * projection[3][2]) / (2.0 * projection[2][2] - 2.0);

    let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * camera_near).z;
    let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

    let dir = cursor_pos_near - camera_transform.translation();

    return dir.normalize();
}