use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::log;
use bevy::reflect::serde::ReflectSerializer;

use crate::objects::{RonNode, Ron};
use crate::gui::FileState;

use crate::{if_err_return, if_none_continue};

use super::SaveRonEvent;
use super::ObjectType;


pub fn process_save_ron (
    mut state: ResMut<FileState>,
    mut reader: EventReader<SaveRonEvent>,
    object_query: Query<(Entity, &Transform, &ObjectType)>,
    children_query: Query<&Children, With<ObjectType>>,
    parent_query: Query<&Parent, With<ObjectType>>,
) {
    for SaveRonEvent {path, root } in reader.iter() {

        let path_buf = if_none_continue!(path);

        log::info!("process_save_ron");

        let mut objects = HashMap::new();
        let mut nodes = HashMap::new();

        if let Some(root) = root {
            process_save_node (
                root,
                &object_query,
                &children_query,
                &parent_query,
                &mut objects,
                &mut nodes,
            );
        } else {  
            state.current_file_path = path.clone(); 
              
            for (_, _, object_type) in object_query.iter() {
                if !objects.contains_key(object_type) {
                    objects.insert(object_type.clone(), objects.len());
                }
            }

            for (entity, transform, object_type) in object_query.iter() {
                let node = RonNode {
                    transform: transform.compute_matrix().to_cols_array(),
                    object: *objects.get(object_type).expect("Failed to create node"),
                    has_parent: parent_query.contains(entity),
                    childrens: Vec::new(),
                };

                nodes.insert(entity, (node, nodes.len()));
            }
        }
        
        let mut ron = Ron {
            objects: objects.into_iter().map(|(v, k)| (k, v)).collect(),
            nodes: nodes.iter().map(|(_, (v, k))| (*k, v.clone())).collect(),
        };

        for (entity,  (_, key) ) in &nodes {
            if let Ok(children) = children_query.get(*entity) { 
                let node = ron.nodes.get_mut(key).expect("Failed process create add childs");
                for child in children.iter() {  
                    if object_query.contains(*child) {
                        node.childrens.push(nodes.get(child).expect("Failed to add child to node").1);
                    }
                }
            }
        }

        let mut registry = bevy::reflect::TypeRegistryInternal::new();
        registry.register::<Ron>();
        
        let serializer = ReflectSerializer::new(&ron, &registry);
        let serialized: String = ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize level");

        let assets_path = state.assets_path.to_str().expect("serialize error: can't replace exe path").replace("\\","\\\\");
        let serialized = serialized.replace(&assets_path, "__path_to_assets__"); 

        std::fs::write(path_buf, serialized).unwrap_or_else(|e| error!("Failed to write level {}", e));
    }
}

fn process_save_node (
    entity: &Entity,
    object_query: &Query<(Entity, &Transform, &ObjectType)>,
    children_query: &Query<&Children, With<ObjectType>>,
    parent_query: &Query<&Parent, With<ObjectType>>,
    objects: &mut HashMap<ObjectType, usize>,
    nodes: &mut HashMap<Entity, (RonNode, usize)>,
) {
    let (entity, transform, object_type) = if_err_return!(object_query.get(*entity));

    if !objects.contains_key(object_type) {
        objects.insert(object_type.clone(), objects.len());
    }

    let node = RonNode {
        transform: transform.compute_matrix().to_cols_array(),
        object: *objects.get(object_type).expect("Failed to create node"),
        has_parent: parent_query.contains(entity),
        childrens: Vec::new(),
    };

    nodes.insert(entity, (node, nodes.len()));

    if let Ok(children) = children_query.get(entity) { 
        for child in children.iter() {  
            if object_query.contains(*child) {
                process_save_node (
                    child,
                    object_query,
                    children_query,
                    parent_query,
                    objects,
                    nodes,
                );
            }
        }
    }
}