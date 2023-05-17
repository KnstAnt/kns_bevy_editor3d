use bevy::{log, prelude::*, render::primitives::Aabb, math::vec4};
use bevy_mod_picking::prelude::*;

use crate::if_err_continue;
use rfd::*;

use super::*;

#[derive(Component, Debug)]
pub struct CompositeObjectLabel;

#[derive(Component, Resource, Default)]
pub(crate) struct Resources {
//    pub mesh: Option<Handle<Mesh>>,
    pub material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn setup_spawn_resources(
    mut resources: ResMut<Resources>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    resources.material = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.3, 0.3, 0.3, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    }));
}

const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(0.2, 0.0, 0.0, 0.1), 
        alpha_mode: AlphaMode::Blend,
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(0.0, 0.2, 0.0, 0.2), 
        alpha_mode: AlphaMode::Blend,
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(0.0, 0.0, 0.2, 0.3), 
        alpha_mode: AlphaMode::Blend,
        ..matl.to_owned()
    })),
};

pub(crate) fn process_set_pickable_mesh(
    mut commands: Commands,
    mut reader: EventReader<SetPickableMeshEvent>,
    children_query: Query<&Children>,
    transform_query: Query<&GlobalTransform, With<ObjectType>>,
    mut mesh_query: Query<&Handle<Mesh>, With<Parent>>,
    mut writer: EventWriter<SetPickableMeshWaiterEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    resources: Res<Resources>,
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
            &resources,
        );

        if let Some(mut entity_commands) = commands.get_entity(*entity) {
            info!("process_set_pickable_mesh");

            let mut min: Vec3 = Vec3::ZERO;
            let mut max: Vec3 = Vec3::ZERO;
    
            for aabb in mesh_data.into_iter() {
                min.x = min.x.min(aabb.center.x - aabb.half_extents.x);
                min.y = min.y.min(aabb.center.y - aabb.half_extents.y);
                min.z = min.z.min(aabb.center.z - aabb.half_extents.z);
    
                max.x = max.x.max(aabb.center.x + aabb.half_extents.x);
                max.y = max.y.max(aabb.center.y + aabb.half_extents.y);
                max.z = max.z.max(aabb.center.z + aabb.half_extents.z);
            }

            entity_commands
                .insert(meshes.add(Mesh::from(shape::Box::from_corners(min, max))))
                .insert(resources
                    .material
                    .clone()
                    .expect("process_set_pickable_mesh resources err: no material!"))
                .insert(HIGHLIGHT_TINT.clone())
                .insert(bevy_transform_gizmo::GizmoTransformable);

            /*             entity_commands.insert(
                            OnPointer::<Click>::run_callback(|In(event): In<ListenedEvent<Click>>| -> Bubble {
            //                  info!("Clicked on entity {:?}", entity);
                                Bubble::Up
                            })
                        ); */

            /*entity_commands.with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: resources.mesh.expect("process_set_pickable_mesh resources err: no mesh!"),
                        material: resources.material.expect("process_set_pickable_mesh resources err: no material!"),
                        transform: Transform::from_translation(label_pos),
                        ..default()
                    })
                    .insert(bevy_transform_gizmo::GizmoTransformable)
                    .insert(CompositeObjectLabel);
            }); */
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
    resources: &Res<Resources>,
) -> Vec<Aabb> {
    let mut res = Vec::new();

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            let mut entity_commands = if_none_continue!(commands.get_entity(*child));

            if let Ok(handle) = mesh_query.get(*child) {
                entity_commands.insert(bevy_transform_gizmo::GizmoTransformable);

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
                resources,
            ));
        }
    }

    res
}
