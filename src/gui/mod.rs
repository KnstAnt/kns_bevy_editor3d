use bevy::{log, prelude::*, window::PrimaryWindow};
use bevy_egui::*;
use rfd::*;
use std::fmt::Debug;

use crate::editor::ClearLevelEvent;
use crate::objects::{LoadRonEvent, SaveRonEvent};
use crate::input::*;

use self::left_panel::process_left_panel;
pub use self::my_state::*;
use self::right_panel::process_right_panel;


mod my_state;
mod left_panel;
mod right_panel;
mod select_panel;

#[derive(PartialEq, Eq)]
enum UiScaleType {
    Increment,
    Decrement,
    Toggle,
}

struct UiScaleEvent {
    pub action: UiScaleType,
}

#[derive(PartialEq, Eq)]
pub enum UiEventType {
    File,
    Gltf,
    Rapier,
}

pub struct UiPanelEvent {
    action: UiEventType,
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
enum Actions {
    #[default]
    None,

    ScaleInc,
    ScaleDec,
    ScaleToggle,
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyEditorState>()
            .init_resource::<FileState>()
            .init_resource::<SelectState>()
            .init_resource::<RenderState>()
            .add_event::<UiScaleEvent>()
            .add_event::<UiPanelEvent>()
            .add_plugin(InputPlugin::<Actions>::default())
            .add_startup_system(setup_input)
            .add_startup_system(configure_visuals_system)
            .add_systems((
                process_input_events,
                update_ui_scale_factor_system.after(process_input_events),
                process_up_panel,
                process_left_panel.after(process_up_panel),
                process_right_panel.after(process_up_panel),
            ));
    }
}

fn setup_input(mut commands: Commands) {
    //    log::info!("WindowPlugin setup_input");
    let mut button_control = ButtonControl::<Actions>::new();

    button_control.add_actions(
        Actions::ScaleInc,
        vec![
            InputAction::from(KeyCode::LControl),
            InputAction::from(KeyCode::PageUp),
        ],
        Notify::OnPress,
    );
    button_control.add_actions(
        Actions::ScaleDec,
        vec![
            InputAction::from(KeyCode::LControl),
            InputAction::from(KeyCode::PageDown),
        ],
        Notify::OnPress,
    );
    button_control.add_actions(
        Actions::ScaleToggle,
        vec![
            InputAction::from(KeyCode::LControl),
            InputAction::from(KeyCode::Back),
        ],
        Notify::OnPress,
    );

    commands.insert_resource(button_control);
}

fn process_input_events(
    mut reader: EventReader<InputEvent<Actions>>,
    mut scale_writer: EventWriter<UiScaleEvent>,
) {
    // log::info!("process_input_events Start");

    for InputEvent::<Actions> { name, value } in reader.iter() {
        match name {
            Actions::ScaleInc => {
   //                           log::info!("WindowPlugin process_input_events Increment");
                scale_writer.send(UiScaleEvent {
                    action: UiScaleType::Increment,
                });
            }

            Actions::ScaleDec => {
   //                      log::info!("WindowPlugin process_input_events Decrement");
                scale_writer.send(UiScaleEvent {
                    action: UiScaleType::Decrement,
                });
            }

            Actions::ScaleToggle => {
  //                       log::info!("WindowPlugin process_input_events Toggle");
                scale_writer.send(UiScaleEvent {
                    action: UiScaleType::Toggle,
                });
            }

            Actions::None => (),
        };
    }
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn update_ui_scale_factor_system(
    mut reader: EventReader<UiScaleEvent>,
    mut toggle_factor: Local<Option<bool>>,
    mut scale_factor: Local<Option<f64>>,
    mut egui_settings: ResMut<EguiSettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    for UiScaleEvent { action } in reader.iter() {

        *scale_factor = Some(
            match action {
                UiScaleType::Increment => 1.1,
                UiScaleType::Decrement => 0.9,
                UiScaleType::Toggle => 1.,
            } * scale_factor.unwrap_or(1.),
        );

        let mut value = scale_factor.unwrap();

        if *action == UiScaleType::Toggle {
            *toggle_factor = Some(!toggle_factor.unwrap_or(true));

            if let Ok(window) = windows.get_single() {
                if toggle_factor.unwrap() {
                    value = value / window.scale_factor();
                }
            }
        };

/*         log::info!(
            "scale_factor {:?}, toggle_factor {:?}, value {:?}",
            scale_factor.unwrap(),
            toggle_factor.unwrap(),
            value
        );
 */
        egui_settings.scale_factor = value;
    }
}


fn process_up_panel(
    mut editor_state: ResMut<MyEditorState>,
    mut render_state: ResMut<RenderState>,
    file_state: Res<FileState>,
    mut contexts: EguiContexts,
//    mut is_quit_open: Local<Option<bool>>,
    mut panel_writer: EventWriter<UiPanelEvent>,
    mut load_writer: EventWriter<LoadRonEvent>,
    mut save_writer: EventWriter<SaveRonEvent>,
    mut clear_writer: EventWriter<ClearLevelEvent>,
) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("top_panel")
    .show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
/*             if ui.button("File").clicked() {
                panel_writer.send( UiPanelEvent { action: UiEventType::File } );
            } */

            egui::menu::menu_button(ui, "File", |ui| {

                let mut assets_path = file_state.assets_path.clone();

                ui.vertical(|ui| {
                    if file_state.current_file_path.is_some() {
                        ui.horizontal(|ui| {
                            ui.label("Current file: ");
                            ui.text_edit_singleline(&mut file_state.get_file_name());
                        });

                        if ui.button("New file").clicked() {
                            if rfd::MessageDialog::new()
                                .set_level(MessageLevel::Warning)
                                .set_title("New file")
                                .set_description("Do you want to clear all data and create new file?")
                                .set_buttons(MessageButtons::YesNo)
                                .show()
                            {
                                clear_writer.send(ClearLevelEvent);
                            }
                        }

                        if ui
                            .button("Save file to ".to_string() + &file_state.get_file_name())
                            .clicked()
                        {
                            if rfd::MessageDialog::new()
                                .set_level(MessageLevel::Warning)
                                .set_title("Save file")
                                .set_description("Do you want save data to file?")
                                .set_buttons(MessageButtons::YesNo)
                                .show()
                            {
                                save_writer.send(SaveRonEvent {
                                    path: file_state.current_file_path.clone(),
                                    root: None,
                                });
                            }
                        }
                    } 

                    if ui.button("Save file as..").clicked() {
                  //      assets_path.push("levels");

                        info!("{:?}", assets_path.as_path());

                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(assets_path.as_path())
                            .add_filter(".ron", &["ron"])
                            .save_file()
                        {
                            save_writer.send(SaveRonEvent { path: Some(path), root: None });
                        }
                    }
    
                    if ui.button("Load file").clicked() {
                   //     assets_path.push("levels");

                        info!("{:?}", assets_path.as_path());

                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(assets_path.as_path())
                            .add_filter(".ron", &["ron"])
                            .pick_file() {
                            clear_writer.send(ClearLevelEvent);
                            load_writer.send( LoadRonEvent { path: Some(path) });
                        }
                    }


                    if ui.button("Quit").clicked() {
                        if rfd::MessageDialog::new()
                        .set_level(MessageLevel::Warning)
                        .set_title("Quit")
                        .set_description("Do you want to quit?")
                        .set_buttons(MessageButtons::YesNo)
                        .show() {
                            std::process::exit(0);
                        }
                    }                   

/*                  if ui.button("Quit").clicked() {
                        *is_quit_open = Some(true);
                    }

                    if is_quit_open.unwrap_or(false) {
                        egui::Window::new("Do you want to quit?")
                            .collapsible(false)
                            .resizable(false)
                            .default_pos(default_pos)
                            .show(ctx, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("Cancel").clicked() {
                                        *is_quit_open = Some(false);
                                    }

                                    if ui.button("Yes!").clicked() {
                                        std::process::exit(0);
                                    }
                                });
                            });
                    }  */
                });
            }); 

            if ui.button("Edit").clicked() {
                panel_writer.send( UiPanelEvent { action: UiEventType::Gltf } );
            }

            egui::menu::menu_button(ui, "Render", |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut render_state.hierarchy_enabled, "hierarchy");
   //                 ui.checkbox(&mut render_state.meshes_enabled, "Meshes");
   //                 ui.checkbox(&mut render_state.empty_enabled, "Empty nodes");
   //                 ui.checkbox(&mut render_state.collider_enabled, "Colliders");
                });
            }); 
        });
    });
}
