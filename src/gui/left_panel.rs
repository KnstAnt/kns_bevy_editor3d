use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rfd::{MessageButtons, MessageLevel};

use super::{FileState, MyEditorState, SelectState};
use crate::editor::ClearLevelEvent;
use crate::objects::{LoadObjectEvent, ObjectType, ColliderType};

pub fn process_left_panel(
    mut editor_state: ResMut<MyEditorState>,
    mut file_state: ResMut<FileState>,
    mut select_state: ResMut<SelectState>,
    mut contexts: EguiContexts,
    mut load_object_writer: EventWriter<LoadObjectEvent>,
    mut clear_writer: EventWriter<ClearLevelEvent>,
) {
    let ctx = contexts.ctx_mut();

    let mut assets_path = file_state.assets_path.clone();

    egui::SidePanel::left("left_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                //if file_state.qnt_loading_ogjects != 0 {
                ui.heading(
                    "objects in loading queue: ".to_string()
                        + &file_state.qnt_loading_ogjects.to_string(),
                );
                //}

                if ui.button("Clear all").clicked() {
                    if rfd::MessageDialog::new()
                        .set_level(MessageLevel::Warning)
                        .set_title("Clear data")
                        .set_description("Do you want to clear all data?")
                        .set_buttons(MessageButtons::YesNo)
                        .show()
                    {
                        clear_writer.send(ClearLevelEvent);
                    }
                }

                ui.horizontal(|ui| {
                    ui.heading(
                        "Selected object: ".to_string() + &editor_state.get_selected_object_name(),
                    );

                    if ui.button("clear").clicked() {
                        editor_state.selected_object = None;
                    }
                });

                ui.horizontal(|ui| {
                    let entity_index = if let Some(entity) = select_state.entity {
                        entity.index().to_string()
                    } else {
                        "-".to_string()
                    };

                    ui.heading("Selected entity: ".to_string() + &entity_index);

                    if ui.button("clear").clicked() {
                        select_state.entity = None;
                        select_state.set_child = false;
                    }
                });

                ui.checkbox(&mut select_state.set_child, "make a child".to_string());

                if ui.button("Load ron file").clicked() {
                    //              current_path.push("map");

                    info!("{:?}", assets_path.as_path());
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(assets_path.as_path())
                        .add_filter("ron", &["ron"])
                        .pick_file()
                    {
                        let rel_path = path
                            .display()
                            .to_string()
                            .replace(&file_state.assets_path.display().to_string(), "");

                        editor_state
                            .objects
                            .insert(rel_path, ObjectType::Ron(path.clone()));
                        load_object_writer.send(LoadObjectEvent {
                            object: Some(ObjectType::Ron(path.clone())),
                        });
                    }
                }

                if ui.button("Load gltf scene").clicked() {
                    //              current_path.push("map");

                    info!("{:?}", assets_path.as_path());
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(assets_path.as_path())
                        .add_filter("gltf", &["gltf"])
                        .pick_file()
                    {
                        //                    state.selected_object = Some(ObjectType::Scene(path));
                        let rel_path = path
                            .display()
                            .to_string()
                            .replace(&file_state.assets_path.display().to_string(), "");

                        editor_state
                            .objects
                            .insert("gltf_scene:".to_string() + &rel_path, ObjectType::Scene((path.clone(), None)));

                        load_object_writer.send(LoadObjectEvent {
                            object: Some(ObjectType::Scene((path.clone(), None))),
                        });

                        /*                    if let Some(scene_name) = path.file_name() {
                            if let Some(scene_name) = scene_name.to_str() {
                                state.objects.insert(scene_name.to_string(), ObjectType::Scene(path.clone()));

                                world.send_event(LoadObjectEvent{object: Some(ObjectType::Scene(path.clone()))});
                            }
                        } */
                    }
                }

                if ui.button("Load gltf mesh").clicked() {
                    //               current_path.push("objects");

                    info!("{:?}", assets_path.as_path());

                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(assets_path.as_path())
                        .add_filter("gltf", &["gltf"])
                        .pick_file()
                    {
                        let rel_path = path
                            .display()
                            .to_string()
                            .replace(&file_state.assets_path.display().to_string(), "");

                        editor_state
                            .objects
                            .insert("gltf_mesh:".to_string() + &rel_path, ObjectType::Mesh((path.clone(), None)));

                        load_object_writer.send(LoadObjectEvent {
                            object: Some(ObjectType::Mesh((path.clone(), None))),
                        });

                        /*if let Some(scene_name) = path.file_name() {
                            if let Some(scene_name) = scene_name.to_str() {
                                state.objects.insert(scene_name.to_string(), ObjectType::Mesh(path.clone()));

                                world.send_event(LoadObjectEvent{object: Some(ObjectType::Mesh(path.clone()))});
                            }
                        } */
                    }
                }
            });


            ui.separator();
            ui.heading("Objects:");
            
            ui.collapsing("Auto make collider", |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut select_state.generate_collider, "generate collider".to_string());
                });
            });

            
            let objects = editor_state.objects.clone();
            egui::ScrollArea::vertical()
                //       .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for (name, object_type) in objects.iter() {
                            ui.horizontal(|ui| {
                                if ui.button(name).clicked() {
                                    editor_state.selected_object = Some(object_type.clone());
                                }

                                if ui.button(" - ").clicked() {
                                    editor_state.objects.remove(name);
                                }
                            });
                        }
                    });
                });

            ui.collapsing("Add collider", |ui| {
                ui.vertical(|ui| {

/*                     ui.collapsing("primitive", |ui| {
                        ui.vertical(|ui| { */
                            ui.horizontal(|ui| {
                                ui.label("Ball, rad:");
                                
                                let mut radius = 1.;
                                ui.add(egui::DragValue::new(&mut radius).speed(0.1));

                                if ui.button("add").clicked() {
                                    editor_state.selected_object = Some(ObjectType::Collider(ColliderType::Ball( (radius*10000.) as u32) ));
                                }                                
                            });

                            ui.horizontal(|ui| {
                                ui.label("Cuboid, xyz:");
                                
                                let mut x = 1.;
                                ui.add(egui::DragValue::new(&mut x).speed(0.1));
                                let mut y = 1.;
                                ui.add(egui::DragValue::new(&mut y).speed(0.1));
                                let mut z = 1.;
                                ui.add(egui::DragValue::new(&mut z).speed(0.1));

                                if ui.button("add").clicked() {
                                    editor_state.selected_object = Some(ObjectType::Collider(ColliderType::Cuboid(( (x*5000.) as u32, (y*5000.) as u32, (z*5000.) as u32,)) ) );
                                }                                
                            }); 

                            ui.horizontal(|ui| {
                                ui.label("Cylinder, h, rad:");
                                
                                let mut height = 1.;
                                ui.add(egui::DragValue::new(&mut height).speed(0.1));
                                let mut radius = 1.;
                                ui.add(egui::DragValue::new(&mut radius).speed(0.1));
                                
                                if ui.button("add").clicked() {
                                    editor_state.selected_object = Some(ObjectType::Collider(ColliderType::Cylinder(( (height*5000.) as u32, (radius*10000.) as u32,)) ) );
                                }                                
                            });

                            ui.horizontal(|ui| {
                                ui.label("Cone, h, rad:");
                                
                                let mut height = 1.;
                                ui.add(egui::DragValue::new(&mut height).speed(0.1));
                                let mut radius = 1.;
                                ui.add(egui::DragValue::new(&mut radius).speed(0.1));
                                
                                if ui.button("add").clicked() {
                                    editor_state.selected_object = Some(ObjectType::Collider(ColliderType::Cone(( (height*5000.) as u32, (radius*10000.) as u32,)) ) );
                                }                                
                            }); 
 
                    });
                });
            });
 //       }); 
}
