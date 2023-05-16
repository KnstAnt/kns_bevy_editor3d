use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
//use bevy_mod_picking::PickingEvent;
use rfd::{MessageButtons, MessageLevel};

use crate::{if_err_return, objects::ObjectType, picking::PickingEvent};

use super::{FileState, SelectState};


pub fn process_right_panel (
    mut commands: Commands,
  //  file_state: Res<FileState>,
    mut select_state: ResMut<SelectState>,
    mut contexts: EguiContexts,
    transform_query: Query<&Transform, With<ObjectType>>,
    root_query: Query<Entity, Without<Parent>>,
    object_query: Query<&ObjectType>,
    children_query: Query<&Children, With<ObjectType>>,
    mut picking_writer: EventWriter<PickingEvent>,
) {
    let ctx = contexts.ctx_mut();

    //let mut assets_path = file_state.assets_path.clone();

    egui::SidePanel::right("right_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.collapsing("Selected entity", |ui| { 
                    if let Some(selected_entity) = select_state.entity {
                        if let Some(mut entity_commands) = commands.get_entity(selected_entity) {                
                            let transform = if let Ok(transform) = transform_query.get(selected_entity) {
                                *transform
                            } else {
                                Transform::IDENTITY
                            };

                            let mut translation = transform.translation;
                            let mut rotation = transform.rotation.to_euler(bevy::prelude::EulerRot::XYZ);
                            let mut scale = transform.scale;

                            egui::Grid::new("transform:").show(ui, |ui| {
                                ui.label("trans");
                                ui.add(egui::DragValue::new(&mut translation.x).speed(0.1));
                                ui.add(egui::DragValue::new(&mut translation.y).speed(0.1));
                                ui.add(egui::DragValue::new(&mut translation.z).speed(0.1));  
                                ui.end_row();   

                                ui.label("rot");
                                ui.add(egui::DragValue::new(&mut rotation.0).speed(0.1));
                                ui.add(egui::DragValue::new(&mut rotation.1).speed(0.1));
                                ui.add(egui::DragValue::new(&mut rotation.2).speed(0.1));
                                ui.end_row();
                                
                                ui.label("scale");
                                ui.add(egui::DragValue::new(&mut scale.x).speed(0.1));                          
                                ui.add(egui::DragValue::new(&mut scale.y).speed(0.1));
                                ui.add(egui::DragValue::new(&mut scale.z).speed(0.1));
                                ui.end_row();
                            });

                            let transform = Transform::from_translation(translation)
                                                        .with_rotation(Quat::from_euler(
                                                            bevy::prelude::EulerRot::XYZ,
                                                            rotation.0,
                                                            rotation.1,
                                                            rotation.2)
                                                        )
                                                        .with_scale(scale);

                            entity_commands.insert(transform);
                        }
                    }
                });

                ui.collapsing("Objects tree:", |mut ui| { 
                    for entity in root_query.iter() {
                        show_node (
                            &mut ui,
                            &entity,
                       //     &mut select_state,
                            &object_query,
                            &children_query,
                            &mut picking_writer,
                        );
                    }
                });                 
            });
        }); 

}


fn show_node (
    ui: &mut egui::Ui,
    entity: &Entity,
 //   select_state: &mut ResMut<SelectState>,
    object_query: &Query<&ObjectType>,
    children_query: &Query<&Children, With<ObjectType>>,
    picking_writer: &mut EventWriter<PickingEvent>,
) {
    if let Ok(object_type) = object_query.get(*entity) {
        ui.collapsing("entity_{entity.index()}", |ui| {       

            ui.vertical(|ui| {

                let name = match object_type {
                    ObjectType::Empty => "Empty",
                    ObjectType::Scene(_path) => "GLTF Scene",
                    ObjectType::Mesh(_path) => "GLTF Mesh",
                    ObjectType::Ron(_path) => "Ron",
                    ObjectType::Collider(_) => "Collider",
                };

                if ui.button(name).clicked() {
                    picking_writer.send(PickingEvent{entity: *entity});

                //    select_state.entity = Some(*entity);
                }

                if let Ok(children) = children_query.get(*entity) {
                    for child in children.iter() {  
            //            if object_query.contains(*child) {
                            show_node (
                                ui,
                                child,
                             //   select_state,
                                object_query,
                                children_query,
                                picking_writer,
                            );
            //           }
                    }
                }
            });
        });
    }
}