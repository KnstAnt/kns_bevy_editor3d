use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::fs;
use bevy::asset::{HandleId, LoadState};
use bevy::prelude::*;
use bevy_gltf::{GltfMesh, GltfNode};
use ::serde::{Serialize, Deserialize, de::DeserializeSeed};
use bevy::utils::HashMap;
use bevy::log;
use bevy::{
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        TypeUuid,
    },
};

use self::loader::RonLoader;
use self::spawn::{process_add_ron, process_spawn_ron};
use self::sawer::process_save_ron;

use super::Object;


mod loader;
mod sawer;
mod spawn;


pub struct LoadRonEvent {
    pub path: Option<PathBuf>,
}

pub struct SaveRonEvent {
    pub path: Option<PathBuf>,    
    pub root: Option<Entity>, 
}

#[derive(Clone)]
pub struct SpawnRonEvent {  
    pub handle: Handle<Ron>,   
}

#[derive(Clone)]
pub struct AddRonEvent {
    pub entity: Entity,     
    pub handle: Handle<Ron>,   
    pub transform: Transform,
}


#[derive(Default, Debug, Clone, PartialEq, Resource, Reflect, FromReflect, Serialize, Deserialize, TypeUuid)]
#[uuid = "05232afa-11b7-42ba-9217-de0f6f0fe88d"]
#[reflect(Serialize, Deserialize)]
pub struct Ron {
    pub objects: HashMap<usize, Object>,
    pub nodes: HashMap<usize, RonNode>,
}

/* #[derive(Default, Debug, Clone, PartialEq, Resource, Reflect, FromReflect, Serialize, Deserialize, TypeUuid)]
#[uuid = "9e33ad75-d8ff-4412-ada1-cfafffc3b394"]
#[reflect(Serialize, Deserialize)] */
#[derive(Default, Debug, Clone, PartialEq, Reflect, FromReflect, Serialize, Deserialize)]
pub struct RonNode {
    pub transform: [f32; 16],
    pub object: usize,
    pub has_parent: bool,
    pub childrens: Vec<usize>,
}

impl Eq for RonNode {
}

impl Hash for RonNode {
        fn hash<H: Hasher>(&self, state: &mut H) {
        for v in self.transform {
            ((v*1000.) as i64).hash(state);
        }

        self.object.hash(state);
    }    
}

pub struct RonPlugin;

impl Plugin for RonPlugin {
    fn build(&self, app: &mut App) {

        app
            .add_asset::<Ron>()
            .add_asset_loader(RonLoader)
            .add_event::<LoadRonEvent>() 
            .add_event::<SaveRonEvent>()   
            .add_event::<AddRonEvent>()           
            .add_event::<SpawnRonEvent>()           
            .add_systems((
                process_save_ron,
                process_spawn_ron,
                process_add_ron
            ))
            ;
    }
}  

