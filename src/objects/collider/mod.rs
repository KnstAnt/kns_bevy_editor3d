use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::fs;
use bevy::asset::{HandleId, LoadState};
use bevy::prelude::*;
use bevy_gltf::{GltfMesh, GltfNode};
use bevy_rapier3d::prelude::Collider;
use ::serde::{Serialize, Deserialize, de::DeserializeSeed};
use bevy::utils::HashMap;
use bevy::log;
use bevy::{
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        TypeUuid,
    },
};
//use bevy_picking_rapier::RapierPickTarget;


#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Resource, Component, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub enum ColliderType {
    #[default]
    Empty,   

    Ball(u32), // radius

    Cuboid((u32, u32, u32)), // half-extents x 3

 //   Capsule((u32, u32, u32)), // a, b, radius

    Cylinder((u32, u32)), // half-height, radius

    Cone((u32, u32)), // half-height, radius

    Terrain,

    Computed,    
}

impl ToString for ColliderType {
    fn to_string(&self) -> String {
        return match self {
            ColliderType::Empty => "ColliderEmpty".to_string(),
            ColliderType::Ball(_) => "ColliderBall".to_string(),
            ColliderType::Cuboid(_) => "ColliderCuboid".to_string(),
   //         ColliderType::Capsule(_) => "ColliderCapsule".to_string(),
            ColliderType::Cylinder(_) => "ColliderCylinder".to_string(),
            ColliderType::Cone(_) => "ColliderCone".to_string(),
            ColliderType::Terrain => "ColliderTerrain".to_string(),
            ColliderType::Computed => "ColliderComputed".to_string(),
        }
    }
}

pub struct CreateColliderEvent {
    pub entity: Entity,
    pub collider: ColliderType,
    pub transform: Transform,
}


pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {

        app      
            .add_event::<CreateColliderEvent>()           
            .add_systems((
                process_create_collider,
            ))
            ;
    }
}  

fn process_create_collider(
    mut commands: Commands,
    mut reader: EventReader<CreateColliderEvent>,
    meshes: Res<Assets<Mesh>>,
) {
    for CreateColliderEvent {
        entity,
        collider,
        transform,
    } in reader.iter() {

        let collider = match collider {
            ColliderType::Empty => continue,

            ColliderType::Ball(radius) => Some(Collider::ball(*radius as f32 *0.0001)),

            ColliderType::Cuboid((hx, hy, hz)) => 
                Some(Collider::cuboid(
                    *hx as f32 *0.0001, 
                    *hy as f32 *0.0001, 
                    *hz as f32 *0.0001, 
                )),

/*             ColliderType::Capsule((a, b, radius)) => 
                Some(Collider::capsule(
                    *a as f32 *0.0001, 
                    *b as f32 *0.0001,  
                    *radius as f32 *0.0001,
                )), */

            ColliderType::Cylinder((hh, radius)) => 
                Some(Collider::cylinder(
                    *hh as f32 *0.0001, 
                    *radius as f32 *0.0001, 
                )),

            ColliderType::Cone((hh, radius)) => 
                Some(Collider::cone(
                    *hh as f32 *0.0001, 
                    *radius as f32 *0.0001, 
                )),

            ColliderType::Terrain => None,//create_terrain(&mut commands, entity, &meshes),

            ColliderType::Computed => None,//create_computed(&mut commands, entity, &meshes),
        };

        if let Some(collider) = collider {
            commands
                .entity(*entity)
                .insert(collider)
//                .insert(RapierPickTarget::default()) // <- Needed for the rapier picking backend
                .insert(*transform);
        }
    }
}

/* fn create_terrain(
    commands: &mut Commands,
    entity: &Entity,
    meshes: &Res<Assets<Mesh>>,
) -> Option<Collider> {
    todo
}

fn create_computed(
    commands: &mut Commands,
    entity: &Entity,
    meshes: &Res<Assets<Mesh>>,
) -> Option<Collider> {
    Collider::from_bevy_mesh(mesh: &Mesh, &ComputedColliderShape::TriMesh)
} */



