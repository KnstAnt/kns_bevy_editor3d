use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::fs;
use bevy::asset::{HandleId, LoadState};
use bevy::ecs::system::EntityCommand;
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

const TO_FLOAT: f32 = 10000.0;
const FROM_FLOAT: f32 = 0.0001;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Resource, Component, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct ColliderData {
    pub friction: u32,
    pub collision_group_self: u32,
    pub collision_group_filter: u32,
    pub solver_group_self: u32,
    pub solver_group_filter: u32,
}

impl ColliderData {
    fn new(
        friction: f32,
        collision_group_self: u32,
        collision_group_filter: u32,
        solver_group_self: u32,
        solver_group_filter: u32,
    ) -> Self {
        Self {
            friction: (friction*TO_FLOAT) as u32,
            collision_group_self,
            collision_group_filter,
            solver_group_self,
            solver_group_filter,
        }
    }

    fn friction(&self) -> f32 { self.friction as f32 * TO_FLOAT }
}

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

            ColliderType::Ball(radius) => Some(Collider::ball(*radius as f32 * FROM_FLOAT)),

            ColliderType::Cuboid((hx, hy, hz)) => 
                Some(Collider::cuboid(
                    *hx as f32 * FROM_FLOAT, 
                    *hy as f32 * FROM_FLOAT, 
                    *hz as f32 * FROM_FLOAT, 
                )),

/*             ColliderType::Capsule((a, b, radius)) => 
                Some(Collider::capsule(
                    *a as f32 * FROM_FLOAT, 
                    *b as f32 * FROM_FLOAT,  
                    *radius as f32 * FROM_FLOAT,
                )), */

            ColliderType::Cylinder((hh, radius)) => 
                Some(Collider::cylinder(
                    *hh as f32 * FROM_FLOAT, 
                    *radius as f32 * FROM_FLOAT, 
                )),

            ColliderType::Cone((hh, radius)) => 
                Some(Collider::cone(
                    *hh as f32 * FROM_FLOAT, 
                    *radius as f32 * FROM_FLOAT, 
                )),
        };

        if let Some(collider) = collider {
            commands
                .entity(*entity)
                .insert(collider)
                .insert(*transform);
        }
    }
}

pub fn add_collider_from_mesh(
    commands: &mut Commands,
    entity: &Entity,
    mesh: &Handle<Mesh>,
    meshes: &Res<Assets<Mesh>>,
    data: &ColliderData,
) {
    log::info!("add_collider_from_mesh");

    let mut entity_commands = crate::if_none_return!(commands.get_entity(*entity));

    if let Some(mesh) = meshes.get(&mesh) {

            if mesh.count_vertices() <= 0 {
                return;
            }

            if let Some(mut collider) = bevy_rapier3d::geometry::Collider::from_bevy_mesh(
                mesh,
                &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh,
            ) {              
                entity_commands
                    .insert(collider)
                    .insert(bevy_rapier3d::prelude::RigidBody::Fixed)
                    .insert(bevy_rapier3d::prelude::Friction::coefficient(data.friction()))
/*                     .insert(bevy_rapier3d::prelude::CollisionGroups::new(
                        data.collision_group_self.try_into().unwrap(),
                        data.collision_group_filter.try_into().unwrap(),
                    ))
                    .insert(SolverGroups::new(
                        data.solver_group_self,
                        data.solver_group_filter,
                    )) */
                    ;

                log::info!("add_collider_from_mesh ok");
            }
        }
}



