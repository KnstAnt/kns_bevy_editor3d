use bevy::{log, prelude::*};
use crate::{if_none_continue, if_none_return};
use crate::objects::{SpawnRonEvent, AddObjectEvent, ObjectType, SetPickableMeshEvent};
use super::{Ron, AddRonEvent, RonNode};

pub fn process_spawn_ron (
    mut commands: Commands,
    mut reader: EventReader<SpawnRonEvent>,
    ron_assets: Res<Assets<Ron>>,
    mut writer: EventWriter<AddObjectEvent>,
) {
 //   log::info!("process_spawn_ron");

    let SpawnRonEvent { handle } = if_none_return!(reader.iter().last());

    let ron = if_none_return!(ron_assets.get(&handle));

    for (_, node) in ron.nodes.iter() {
        if node.has_parent {
            continue;
        }

        process_spawn_node(    
            &mut commands,
            None,
            node,
            &ron,
            &mut writer,
        );
    }
}

pub fn process_add_ron (
    mut commands: Commands,
    mut reader: EventReader<AddRonEvent>,
    ron_assets: Res<Assets<Ron>>,
    mut add_obj_writer: EventWriter<AddObjectEvent>,
    mut set_pickable_writer: EventWriter<SetPickableMeshEvent>,
) {  
    for AddRonEvent {
        entity,
        handle,
        transform,
    } in reader.iter() {

    // log::info!("process_add_ron");

        let ron = if_none_continue!(ron_assets.get(handle));

        add_obj_writer.send( AddObjectEvent{ 
            entity: Some(*entity), 
            object: Some(ObjectType::Empty), 
            transform: Some(*transform),
            selected: true,
        } );

        for (_, node) in ron.nodes.iter() {
            if node.has_parent {
                continue;
            }

            process_spawn_node(    
                &mut commands,
                Some(*entity),
                node,
                &ron,
                &mut add_obj_writer,
            );
        }

        set_pickable_writer.send(SetPickableMeshEvent { entity: *entity });
    }
}

fn process_spawn_node(
    commands: &mut Commands,
    parent: Option<Entity>,
    node: &RonNode,
    ron: &Ron,
    writer: &mut EventWriter<AddObjectEvent>,
) { 
    let entity = commands.spawn_empty().id();

    if let Some(parent) = parent {
        commands.entity(parent).add_child(entity);
    }

    writer.send( AddObjectEvent{ 
        entity: Some(entity), 
        object: Some(ron.objects.get(&node.object).expect("process_load_ron err: failed create the node").clone()), 
        transform: Some(Transform::from_matrix(Mat4::from_cols_array(&node.transform))),
        selected: false,
    } );

    for node_id in node.childrens.iter() { 
        process_spawn_node(
            commands, 
            Some(entity), 
            ron.nodes.get(&node_id).expect("process_load_ron err: failed create the node"), 
            &ron,
            writer,
        );
    }        
}
