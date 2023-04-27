use bevy::{log, prelude::*, render::primitives::Aabb};
use bevy_gltf::{GltfMesh, GltfNode};
use bevy_mod_picking::*;
use bevy_prototype_debug_lines::DebugLines;
use bevy_transform_gizmo::{GizmoPickSource, GizmoTransformable};

use crate::{gui::SelectState, if_err_continue, if_err_return, if_none_return};
use rfd::*;

use crate::picking::*;

use super::*;

#[derive(Component, Debug)]
pub struct CompositeObjectLabel;

pub fn process_add_object(
    mut commands: Commands,
    mut state: ResMut<SelectState>,
    mut reader: EventReader<AddObjectEvent>,
    asset_server: Res<AssetServer>,
    mut gltf_scene_writer: EventWriter<AddGltfSceneEvent>,
    mut gltf_mesh_writer: EventWriter<AddGltfMeshEvent>,
    mut ron_writer: EventWriter<AddRonEvent>,
) {
    for AddObjectEvent {
        entity,
        object,
        transform,
        selected,
    } in reader.iter()
    {
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
                .insert(bevy_mod_raycast::RaycastMesh::<ObjectRaycastSet>::default())
                .insert(PickableBundle::default())
                .insert(GizmoTransformable);

            match object.clone() {
                ObjectType::Scene(path) => {
                    gltf_scene_writer.send(AddGltfSceneEvent {
                        entity,
                        handle: asset_server.load(path.display().to_string() + "#Scene0"),
                        transform,
                    });
                }

                ObjectType::Mesh(path) => {
                    gltf_mesh_writer.send(AddGltfMeshEvent {
                        entity,
                        handle: asset_server.load(path.display().to_string() + "#Mesh0"),
                        transform,
                    });
                }

                ObjectType::Ron(path) => {
                    ron_writer.send(AddRonEvent {
                        entity,
                        handle: asset_server.load(path.display().to_string()),
                        transform,
                    });
                }

                ObjectType::Empty => {
                    entity_commands.insert(SpatialBundle {
                        transform,
                        ..Default::default()
                    });
                }
            };
        }

        if *selected {
            commands.entity(entity).insert(PickableBundle {
                interaction: Interaction::Clicked,
                ..default()
            });

            state.entity = Some(entity);
        }
    }
}

pub fn process_add_gltf_scene(
    mut commands: Commands,
    mut reader: EventReader<AddGltfSceneEvent>,
    mut writer: EventWriter<SetPickableMeshEvent>,
) {
    for AddGltfSceneEvent {
        entity,
        handle,
        transform,
    } in reader.iter()
    {
        log::info!("process_add_gltf_scene");

        if let Some(mut entity_commands) = commands.get_entity(*entity) {
            entity_commands.insert(SceneBundle {
                scene: handle.to_owned(),
                transform: *transform,
                ..default()
            });

            writer.send(SetPickableMeshEvent { entity: *entity });
        }
    }
}

pub fn process_set_pickable_mesh(
    mut commands: Commands,
    mut reader: EventReader<SetPickableMeshEvent>,
    children_query: Query<&Children>,
    transform_query: Query<&GlobalTransform, With<ObjectType>>,
    mut mesh_query: Query<&Handle<Mesh>, With<Parent>>,
    mut writer: EventWriter<SetPickableMeshWaiterEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for SetPickableMeshEvent { entity } in reader.iter() {
        if !children_query.contains(*entity) {
            writer.send(SetPickableMeshWaiterEvent { entity: *entity });
            continue;
        }

        let global_transform = if_err_continue!(transform_query.get(*entity));
        let root_pos = global_transform.translation();

        let mesh_data = get_childs_with_mesh(
            root_pos,
            &mut commands,
            *entity,
            &children_query,
            &mut mesh_query,
            &meshes,
        );

        let max_y = mesh_data
            .iter()
            .map(|aabb| aabb.max().y)
            .collect::<Vec<f32>>()
            .into_iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0f32);

        /*             let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);

        for aabb in mesh_data.iter() {
            let mesh_max = aabb.max();
            let mesh_min = aabb.min();

            max.x = max.x.max(mesh_max.x);
            max.y = max.y.max(mesh_max.y);
            max.z = max.z.max(mesh_max.z);

            min.x = min.x.min(mesh_min.x);
            min.y = min.y.min(mesh_min.y);
            min.z = min.z.min(mesh_min.z);
        } */

        let label_pos = Vec3::new(0., max_y + 1., 0.);

        if let Some(mut entity_commands) = commands.get_entity(*entity) {
            entity_commands.with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::UVSphere {
                            radius: 1.,
                            sectors: 8,
                            stacks: 8,
                        })),
                        material: materials.add(StandardMaterial {
                            base_color: Color::RED,
                            emissive: Color::rgba_linear(100.0, 0.0, 0.0, 0.0),
                            ..default()
                        }),
                        transform: Transform::from_translation(label_pos),
                        ..default()
                    })
                    .insert(bevy_mod_raycast::RaycastMesh::<ObjectRaycastSet>::default())
                    .insert(PickableBundle::default())
                    .insert(CompositeObjectLabel);
            });
        }
    }
}

pub fn await_set_pickable_mesh(
    mut reader: EventReader<SetPickableMeshWaiterEvent>,
    mut writer: EventWriter<SetPickableMeshEvent>,
) {
    for SetPickableMeshWaiterEvent { entity } in reader.iter() {
        writer.send(SetPickableMeshEvent { entity: *entity });
    }
}

fn get_childs_with_mesh(
    root: Vec3,
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<&Handle<Mesh>, With<Parent>>,
    meshes: &ResMut<Assets<Mesh>>,
) -> Vec<Aabb> {
    let mut res = Vec::new();

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if let Ok(handle) = mesh_query.get(*child) {
                if let Some(mut entity_commands) = commands.get_entity(*child) {
                    entity_commands
                        .insert(bevy_mod_raycast::RaycastMesh::<ObjectRaycastSet>::default())
                        .insert(PickableBundle::default());
                }

                if let Some(mesh) = meshes.get(handle) {
                    if let Some(aabb) = mesh.compute_aabb() {
                        res.push(aabb.clone());
                    }
                }
            }

            res.append(&mut get_childs_with_mesh(
                root,
                commands,
                *child,
                children_query,
                mesh_query,
                meshes,
            ));
        }
    }

    res
}

pub fn process_add_gltf_mesh(
    mut commands: Commands,
    mut reader: EventReader<AddGltfMeshEvent>,
    //    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
) {
    for AddGltfMeshEvent {
        entity,
        handle,
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
