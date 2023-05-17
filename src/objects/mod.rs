use std::hash::Hash;
use std::path::PathBuf;
use bevy::asset::{HandleId, LoadState};
use bevy::prelude::*;
use bevy_gltf::{GltfMesh, GltfNode};
use bevy_mod_picking::PickableBundle;
use bevy_transform_gizmo::GizmoTransformable;
use ::serde::{Serialize, Deserialize, de::DeserializeSeed};
use bevy::log;
use bevy::{
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        TypeUuid,
    },
};
use crate::gui::{FileState, SelectState};
use crate::picking::{ObjectRaycastSet, PickingEvent};
use crate::{if_none_return, if_none_continue, if_err_return};

use self::collider::ColliderData;
pub use self::collider::{ColliderType, CreateColliderEvent, ColliderPlugin};
use self::gltf::{process_add_gltf_scene, process_add_gltf_mesh};
pub use self::ron::*;
pub use self::spawn::CompositeObjectLabel;
use self::spawn::*;


mod ron;
mod gltf;
mod spawn;
mod collider;

#[derive(Clone)]
pub struct AddObjectEvent {
    pub entity: Option<Entity>,
    pub object: Option<ObjectType>,
    pub collider: Option<ColliderData>,
    pub transform: Option<Transform>,
    pub selected: bool,
}

#[derive(Clone)]
pub struct AddGltfSceneEvent {
    pub entity: Entity,
    pub handle: Handle<Scene>,
    pub collider: Option<ColliderData>,
    pub transform: Transform,
}

#[derive(Clone)]
pub struct SetPickableMeshEvent {
    pub entity: Entity,
}
#[derive(Clone)]
pub struct SetPickableMeshWaiterEvent {
    pub entity: Entity,
}

#[derive(Clone)]
pub struct AddGltfMeshEvent {
    pub entity: Entity,
    pub handle: Handle<GltfMesh>,
    pub collider: Option<ColliderData>,
    pub transform: Transform,
}

pub struct LoadObjectEvent {
    pub object: Option<ObjectType>,   
}

#[derive(Default, Debug, Resource, Component)]
struct LoadedObjects {
    pub handles: Vec<HandleId>,   
}


#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Resource, Component, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub enum ObjectType {
    #[default]
    Empty,
    Scene((PathBuf, Option<ColliderData>)),
    Mesh((PathBuf, Option<ColliderData>)),
    Ron(PathBuf),
    Collider(ColliderType),
}

pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {

        app
            .insert_resource(LoadedObjects::default())
            .insert_resource(Resources::default())
            .add_event::<LoadObjectEvent>()   
            .add_event::<AddObjectEvent>()     
            .add_event::<AddGltfSceneEvent>()    
            .add_event::<SetPickableMeshEvent>()  
            .add_event::<SetPickableMeshWaiterEvent>()  
            .add_event::<AddGltfMeshEvent>()    
            .add_plugin(RonPlugin)    
            .add_plugin(ColliderPlugin)  
            .add_startup_system(setup_spawn_resources)  
            .add_system(process_load_object.before(check_load_objects_complete))
            .add_system(check_load_objects_complete.after(process_load_object))
            .add_systems((
                check_load_ron.before(process_load_ron),               
                process_load_ron.after(check_load_ron),
            ))
            .add_systems((                
                process_add_object,
                process_add_gltf_scene.after(process_add_object),
                process_add_gltf_mesh.after(process_add_object),
                process_set_pickable_mesh.after(process_add_gltf_scene),
                await_set_pickable_mesh.after(process_set_pickable_mesh),
            ))
            ;
    }
}  

fn process_load_object(
    mut reader: EventReader<LoadObjectEvent>,
    asset_server: Res<AssetServer>,
    mut load_data: ResMut<LoadedObjects>,
) {
    for LoadObjectEvent { object } in reader.iter() {
        let object = if_none_continue!(object);

        match object.clone() {
            ObjectType::Scene((path, _collider)) => { 
                let handle: Handle<Scene> = asset_server.load(path.display().to_string() + "#Scene0");
                load_data.handles.push(handle.id());
            },   
            ObjectType::Mesh((path, _collider)) => {                 
                let handle: Handle<GltfMesh> = asset_server.load(path.display().to_string() + "#Mesh0"); 
                load_data.handles.push(handle.id());
            },
            ObjectType::Ron(path) => {                 
                let handle: Handle<Ron> = asset_server.load(path.display().to_string());  
                load_data.handles.push(handle.id());
            },

            ObjectType::Collider(_) => continue,

            ObjectType::Empty => continue,

        };

        log::info!("process_load_object {:?}, {}", object, load_data.handles.len());
    }
}

fn check_load_objects_complete(
    mut state: ResMut<FileState>,
    mut load_data: ResMut<LoadedObjects>,
    asset_server: Res<AssetServer>,
    ron_assets: Res<Assets<Ron>>,
    mut writer: EventWriter<LoadObjectEvent>,
) {
    load_data.handles = load_data
        .handles
        .iter()
        .filter(|&&handle| {
            return match asset_server.get_load_state(handle) {                
                LoadState::NotLoaded => true,
                LoadState::Loading => true,
                LoadState::Loaded => {                    
                    let handle = asset_server.get_handle(handle);

                    if let Some(ron) = ron_assets.get(&handle) {

                        log::info!("check_load_objects_complete ron ok");

                        for (_, object) in ron.objects.iter() {
                            writer.send(LoadObjectEvent{object: Some(object.clone())});
                        }
                    }

                    false
                },
                LoadState::Failed => false,
                LoadState::Unloaded => true, 
            };
        } )
        .map(|handle| *handle)
        .collect::<Vec<HandleId>>();

    state.qnt_loading_ogjects = load_data.handles.len();
}


pub fn process_add_object(
    mut commands: Commands,
    mut state: ResMut<SelectState>,
    mut reader: EventReader<AddObjectEvent>,
    asset_server: Res<AssetServer>,
    mut gltf_scene_writer: EventWriter<AddGltfSceneEvent>,
    mut gltf_mesh_writer: EventWriter<AddGltfMeshEvent>,
    mut ron_writer: EventWriter<AddRonEvent>,
    mut collider_writer: EventWriter<CreateColliderEvent>,
    mut picking_writer: EventWriter<PickingEvent>,
) {
    for AddObjectEvent {
        entity,
        object,
        collider,
        transform,
        selected,
    } in reader.iter() {
        log::info!("process_add_object");

        let object = if_none_return!(object.clone());

        let transform = if let Some(transform) = transform {
            transform.clone()
        } else {
            Transform::IDENTITY
        };

        let entity = if_none_return!(entity.clone());

        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands
                .insert(object.clone())
                .insert(GizmoTransformable);

            match object.clone() {
                ObjectType::Scene((path, collider)) => {
                    gltf_scene_writer.send(AddGltfSceneEvent {
                        entity,
                        collider: collider.clone(),
                        handle: asset_server.load(path.display().to_string() + "#Scene0"),
                        transform,
                    });
                },

                ObjectType::Mesh((path, collider)) => {
                    gltf_mesh_writer.send(AddGltfMeshEvent {
                        entity,
                        collider: collider.clone(),
                        handle: asset_server.load(path.display().to_string() + "#Mesh0"),
                        transform,
                    });
                },

                ObjectType::Ron(path) => {
                    ron_writer.send(AddRonEvent {
                        entity,
                        handle: asset_server.load(path.display().to_string()),
                        transform,
                    });
                },

                ObjectType::Collider(collider) => {
                    collider_writer.send(CreateColliderEvent {
                        entity,
                        collider,
                        transform,
                    });
                },
                
                ObjectType::Empty => {
                    entity_commands.insert(SpatialBundle {
                        transform,
                        ..Default::default()
                    });
                },
            };
        }

        if *selected {
            picking_writer.send(PickingEvent { entity });

/*             commands.entity(entity).insert(PickableBundle {
                interaction: Interaction::Clicked,
                ..default()
            });

            state.entity = Some(entity); */
        }
    }
}

fn process_load_ron (
    mut reader: EventReader<LoadRonEvent>,
    mut state: ResMut<FileState>,
    asset_server: Res<AssetServer>,
) {
    let LoadRonEvent {path} = if_none_return!(reader.iter().last());

    log::info!("process_load_ron");   

    state.load_handle = Some(asset_server.load(path.clone().expect("process_load_ron path err").display().to_string()));  
    state.current_file_path = path.clone();
}

fn check_load_ron (
    mut state: ResMut<FileState>,
    mut writer: EventWriter<SpawnRonEvent>,
) {  
    if state.qnt_loading_ogjects != 0 {
        return;
    }

    let handle = if_none_return!(state.load_handle.clone());

    state.load_handle = None;    

    writer.send(SpawnRonEvent { handle });
}




