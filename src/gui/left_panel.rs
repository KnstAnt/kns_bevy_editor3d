use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rfd::{MessageButtons, MessageLevel};

use super::{FileState, MyEditorState, SelectState};
use crate::editor::ClearLevelEvent;
use crate::filesystem::{LoadObjectEvent, ObjectType};

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
                            .insert(rel_path, ObjectType::Scene(path.clone()));
                        load_object_writer.send(LoadObjectEvent {
                            object: Some(ObjectType::Scene(path.clone())),
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
                            .insert(rel_path, ObjectType::Mesh(path.clone()));
                        load_object_writer.send(LoadObjectEvent {
                            object: Some(ObjectType::Mesh(path.clone())),
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
                    ui.collapsing("From GLTF terrain", |ui| {});

                    ui.collapsing("From bevy mesh", |ui| {});

                    ui.collapsing("primitive", |ui| {});
                });
            });
        });
}
