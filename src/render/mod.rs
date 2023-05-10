use bevy::{log, prelude::*};
use bevy_prototype_debug_lines::DebugLines;

use crate::gui::{SelectState, RenderState};

use crate::objects::{CompositeObjectLabel, ObjectType};


//mod navmesh;
mod physics;


pub struct RenderPlugin ;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems((
                process_drow_hierarchy.run_if(resource_exists::<RenderState>().and_then(
                    |state: Res<RenderState>| state.hierarchy_enabled,
                )),
            ));
    }
}  

fn process_drow_hierarchy(
    label_query: Query<(&Parent, &GlobalTransform), With<CompositeObjectLabel>>,   
    children_query: Query<&Children>,
    transform_query: Query<&GlobalTransform, (With<Handle<Mesh>>, With<Parent>, Without<CompositeObjectLabel>)>,
    object_query: Query<&ObjectType>,
    mut lines: ResMut<DebugLines>,
) {
    for (root, label_transform) in label_query.iter() {
    //    log::info!("process_drow_hierarchy");

        let label_pos = label_transform.translation();

        let mesh_data = get_pos_childs_with_mesh(
            root.get(),
            &children_query,
            &transform_query,
            &object_query,
        );

        for child_pos in mesh_data.into_iter() {
            lines.line_colored(child_pos, label_pos, 0.0, Color::GREEN);
        }
    }
}

fn get_pos_childs_with_mesh(
    entity: Entity,
    children_query: &Query<&Children>,
    transform_query: &Query<&GlobalTransform, (With<Handle<Mesh>>, With<Parent>, Without<CompositeObjectLabel>)>,
    object_query: &Query<&ObjectType>,
) -> Vec<Vec3> {
    let mut res = Vec::new();

    if let Ok(children) = children_query.get(entity) {
 //       log::info!("get_pos_childs_with_mesh children ok");

        for child in children.iter() {

            if let Ok(transform) = transform_query.get(*child) {
    //            log::info!("get_pos_childs_with_mesh child ok");
                res.push(transform.translation());
            }

            if object_query.contains(*child) {
     //           log::info!("get_pos_childs_with_mesh child ObjectType");
                continue;
            } 

            res.append(
                &mut get_pos_childs_with_mesh( *child, children_query, transform_query, object_query)
            );
        }
    }

    res
}