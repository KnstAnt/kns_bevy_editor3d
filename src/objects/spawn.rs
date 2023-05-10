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
//                    .insert(bevy_mod_raycast::RaycastMesh::<ObjectRaycastSet>::default())
//                    .insert(PickableBundle::default())
                    .insert(bevy_transform_gizmo::GizmoTransformable)
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
                        .insert(bevy_transform_gizmo::GizmoTransformable)
//                        .insert(bevy_mod_raycast::RaycastMesh::<ObjectRaycastSet>::default())
//                        .insert(PickableBundle::default())
                    ;
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
