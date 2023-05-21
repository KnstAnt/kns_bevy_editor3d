use bevy::asset::{HandleId, LoadState};
use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use bevy_gltf::{GltfMesh, GltfNode};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use ::serde::{de::DeserializeSeed, Deserialize, Serialize};
use bevy::log;
use bevy::reflect::{
    serde::{ReflectSerializer, UntypedReflectDeserializer},
    TypeUuid,
};
use bevy::utils::HashMap;
//use bevy_picking_rapier::RapierPickTarget;

const TO_FLOAT: f32 = 10000.0;
const FROM_FLOAT: f32 = 0.0001;

#[derive(
    Default,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Resource,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
#[reflect(Resource, Serialize, Deserialize)]
pub enum ColliderType {
    #[default]
    FromBevyMesh,

    Ball(u32), // radius

    Cuboid((u32, u32, u32)), // half-extents x 3

    //   Capsule((u32, u32, u32)), // a, b, radius
    Cylinder((u32, u32)), // half-height, radius

    Cone((u32, u32)), // half-height, radius
}

impl ToString for ColliderType {
    fn to_string(&self) -> String {
        return match self {
            ColliderType::FromBevyMesh => "FromBevyMesh".to_string(),
            ColliderType::Ball(_) => "ColliderBall".to_string(),
            ColliderType::Cuboid(_) => "ColliderCuboid".to_string(),
            //         ColliderType::Capsule(_) => "ColliderCapsule".to_string(),
            ColliderType::Cylinder(_) => "ColliderCylinder".to_string(),
            ColliderType::Cone(_) => "ColliderCone".to_string(),
        };
    }
}

#[derive(
    Default,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Resource,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct ColliderData {
    pub friction: u32,
    pub fixed: bool,
    pub collision_group_self: u32,
    pub collision_group_filter: u32,
    pub solver_group_self: u32,
    pub solver_group_filter: u32,
}

impl ColliderData {
    pub fn new(
        friction: f32,
        fixed: bool,
        collision_group_self: u32,
        collision_group_filter: u32,
        solver_group_self: u32,
        solver_group_filter: u32,
    ) -> Self {
        Self {
            friction: (friction*TO_FLOAT) as u32,
            fixed,
            collision_group_self,
            collision_group_filter,
            solver_group_self,
            solver_group_filter,
        }
    }
    pub fn get_friction(&self) -> f32 {
        self.friction as f32 * FROM_FLOAT
    }
}

#[derive(
    Default,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Resource,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct Collider {
    pub collider_type: ColliderType,
    pub collider_data: ColliderData,
}

/*  impl Collider {
/*     pub fn new(
        collider_type: ColliderType,
        friction: f32,
        collision_group_self: u32,
        collision_group_filter: u32,
        solver_group_self: u32,
        solver_group_filter: u32,
    ) -> Self {
        Self {
            collider_type,
            friction: (friction*TO_FLOAT) as u32,
            collision_group_self,
            collision_group_filter,
            solver_group_self,
            solver_group_filter,
        }
    } */

    fn set_friction(&self, value: f32) { self.collider_data.friction = (value * TO_FLOAT) as u32; }
    fn get_friction(&self) -> f32 { self.collider_data.friction as f32 * FROM_FLOAT }
}  */

pub struct CreateColliderEvent {
    pub entity: Entity,
    pub collider: self::Collider,
    pub transform: Option<Transform>,
}

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateColliderEvent>()
            .add_systems((process_create_collider,));
    }
}

fn process_create_collider(
    mut commands: Commands,
    mut reader: EventReader<CreateColliderEvent>,
    mesh_query: Query<&Handle<Mesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    for CreateColliderEvent {
        entity,
        collider,
        transform,
    } in reader.iter()
    {
        if let Some(rapier_collider) = match collider.collider_type {
            ColliderType::FromBevyMesh => {
                if let Ok(mesh) = mesh_query.get(*entity) {
                    if let Some(mesh) = meshes.get(mesh) {
                        add_collider_from_mesh(
                            &mut commands,
                            &entity,
                            &mesh,
                            &collider.collider_data,
                        );
                    }
                }

                return;
            }, 

            ColliderType::Ball(radius) => Some(bevy_rapier3d::prelude::Collider::ball(
                radius as f32 * FROM_FLOAT,
            )),

            ColliderType::Cuboid((hx, hy, hz)) => Some(bevy_rapier3d::prelude::Collider::cuboid(
                hx as f32 * FROM_FLOAT,
                hy as f32 * FROM_FLOAT,
                hz as f32 * FROM_FLOAT,
            )),

            /*             ColliderType::Capsule((a, b, radius)) =>
            Some(Collider::capsule(
                *a as f32 * FROM_FLOAT,
                *b as f32 * FROM_FLOAT,
                *radius as f32 * FROM_FLOAT,
            )), */
            ColliderType::Cylinder((hh, radius)) => {
                Some(bevy_rapier3d::prelude::Collider::cylinder(
                    hh as f32 * FROM_FLOAT,
                    radius as f32 * FROM_FLOAT,
                ))
            }

            ColliderType::Cone((hh, radius)) => Some(bevy_rapier3d::prelude::Collider::cone(
                hh as f32 * FROM_FLOAT,
                radius as f32 * FROM_FLOAT,
            )),
        } {
            if let Some(mut entity_commands) = commands.get_entity(*entity) {
                entity_commands.insert(rapier_collider);

                if let Some(transform) = transform {
                    entity_commands.insert(*transform);
                }

                aply_collider_data(&mut commands, &entity, &collider.collider_data);
            } 
        }
    }
}

fn aply_collider_data(commands: &mut Commands, entity: &Entity, collider_data: &ColliderData) {
    log::info!("aply_collider_data");

    let mut entity_commands = crate::if_none_return!(commands.get_entity(*entity));

    entity_commands.insert(bevy_rapier3d::prelude::Friction::coefficient(
        collider_data.get_friction(),
    ));

    if collider_data.fixed {
        entity_commands.insert(bevy_rapier3d::prelude::RigidBody::Fixed);
    }

/*     entity_commands
        .insert(bevy_rapier3d::prelude::CollisionGroups::new(
            data.collision_group_self.try_into().unwrap(),
            data.collision_group_filter.try_into().unwrap(),
        ))
        .insert(bevy_rapier3d::prelude::SolverGroups::new(
            data.solver_group_self,
            data.solver_group_filter,
        )); */
}

pub fn add_collider_from_mesh(
    commands: &mut Commands,
    entity: &Entity,
    mesh: &Mesh,
    collider_data: &ColliderData,
) {
    log::info!("add_collider_from_mesh");

    let mut entity_commands = crate::if_none_return!(commands.get_entity(*entity));

    if mesh.count_vertices() <= 0 {
        return;
    }

    if let Some(collider) = bevy_rapier3d::prelude::Collider::from_bevy_mesh(
        mesh,
        &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh,
    ) {
        entity_commands.insert(collider);

        log::info!("add_collider_from_mesh ok");
    } else {
        return;
    }

    aply_collider_data(commands, entity, collider_data);
}
