use bevy::{prelude::*, utils::HashMap};
use std::path::PathBuf;

use crate::filesystem::*;


#[derive(Resource, Component)]
pub struct MyEditorState {
    pub selected_object: Option<ObjectType>,
    pub objects: HashMap<String, ObjectType>,
}

impl Default for MyEditorState {
    fn default() -> Self {
        Self {
            selected_object: None,
            objects: HashMap::new(),
        }
    }
}

impl MyEditorState {
    pub fn get_selected_object_name(&self) -> String {
        if let Some(selected_object) = self.selected_object.clone() {
            return match selected_object {
                ObjectType::Scene(path) => {
                    get_name(&Some(path))
                }
                ObjectType::Mesh(path) => {
                    get_name(&Some(path))
                }
                ObjectType::Ron(path) => {
                    get_name(&Some(path))
                }
                ObjectType::Empty => "empty".to_string(),
            };
        }

        "-".to_string()
    }
}

#[derive(Default, Resource, Component)]
pub struct SelectState {
    pub set_child: bool,
    pub entity: Option<Entity>,
}

#[derive(Resource, Component)]
pub struct FileState {
    pub qnt_loading_ogjects: usize,
    pub assets_path: PathBuf,    
    pub current_file_path: Option<PathBuf>,
    pub load_handle: Option<Handle<Ron>>,
}

impl Default for FileState {
    fn default() -> Self {
        let mut assets_path = std::env::current_exe().expect("Failed to get assets path");
        assets_path.pop();
        assets_path.push("assets");

        Self {
            qnt_loading_ogjects: 0,
            current_file_path: None,
            assets_path,
            load_handle: None,
        }
    }
}

impl FileState {
    pub fn get_file_name(&self) -> String {
        get_name(&self.current_file_path)
    }
}

fn get_name(path: &Option<PathBuf>) -> String {
    if let Some(path) = path.clone() {
        if let Some(file_name) = path.file_name() {
            if let Some(file_name) = file_name.to_str() {
                return file_name.to_string();
            }
        }
    }

    "-".to_string()
}

#[derive(Resource, Component)]
pub struct RenderState {
    pub empty_enabled: bool,
    pub collider_enabled: bool,
    pub navmesh_enabled: bool,
    pub hierarchy_enabled: bool,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            empty_enabled: false,
            collider_enabled: false,
            navmesh_enabled: false,
            hierarchy_enabled: true, 
        }
    }
}
