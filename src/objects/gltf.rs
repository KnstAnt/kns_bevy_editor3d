use bevy::{log, prelude::*};
use bevy_gltf::{GltfMesh, GltfNode};
use bevy_mod_picking::prelude::{OnPointer, Click, ListenedEvent, Bubble};

use crate::if_none_return;
use rfd::*;

use super::*;



pub fn process_add_gltf_scene(
    mut commands: Commands,
    mut reader: EventReader<AddGltfSceneEvent>,
    mut writer: EventWriter<SetPickableMeshEvent>,
) {
    for AddGltfSceneEvent {
        entity,
        handle,
        collider,
        transform,
    } in reader.iter()
    {
        log::info!("process_add_gltf_scene");

        if let Some(mut entity_commands) = commands.get_entity(*entity) {
            entity_commands.insert(SceneBundle {
                scene: handle.to_owned(),
                transform: *transform,
                ..default()
            })
            /* .insert(
                OnPointer::<Click>::run_callback(|In(event): In<ListenedEvent<Click>>| {
    //                info!("Clicked on entity {:?}", entity);
                    Bubble::Up
                }),
            ) */;

            writer.send(SetPickableMeshEvent { entity: *entity });
        }
    }
}

pub fn process_add_gltf_mesh(
    mut commands: Commands,
    mut reader: EventReader<AddGltfMeshEvent>,
    //    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    for AddGltfMeshEvent {
        entity,
        handle,
        collider,
        transform,
    } in reader.iter()
    {
        log::info!("process_add_gltf_mesh");

        let mut entity_commands = if_none_return!(commands.get_entity(*entity));

        if let Some(primitive) = gltf_meshes.get(&handle) {
            if let Some(primitive) = primitive.primitives.first() {


                if let Some(material) = primitive.material.clone() {
                    entity_commands.insert((PbrBundle {
                        mesh: primitive.mesh.clone(),
                        transform: *transform,
                        material,
                        ..default()
                    },));
                } else {
                    entity_commands.insert((PbrBundle {
                        mesh: primitive.mesh.clone(),
                        transform: *transform,
                        ..default()
                    },));
                };


                if let Some(data) = collider {
                    log::info!("process_add_gltf_mesh collider");

                    collider::add_collider_from_mesh(
                        &mut commands,
                        entity,
                        &primitive.mesh,
                        &meshes,
                        &data,
                    );
                }
            } else {
                rfd::MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title("Add gltf mesh error")
                    .set_description("Primitive not loaded!Try a few seconds later.")
                    .set_buttons(MessageButtons::Ok)
                    .show();
            };
        } else {
            rfd::MessageDialog::new()
                .set_level(MessageLevel::Error)
                .set_title("Add gltf mesh error")
                .set_description("Gltf_mesh not loaded!\nTry a few seconds later.")
                .set_buttons(MessageButtons::Ok)
                .show();
        };
    }
}
