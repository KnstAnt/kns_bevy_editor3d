use std:: marker::PhantomData;

use bevy::input::mouse::*;
use bevy::input::keyboard::*;
use bevy::prelude::*;
use core::fmt::Debug;
use core::hash::Hash;
use std::cmp::Eq;

pub use self::button_control::*;
pub mod button_control;

pub struct InputEvent<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,
{
    pub name: T,
    pub value: f64,
}


pub struct InputPlugin<T>(pub PhantomData<T>)
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone;

impl<T> Plugin for InputPlugin<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ButtonControl<T>>()      
            .add_event::<InputEvent<T>>()      
            .add_systems((
                            process_input::<T>,
                            process_keyboard_event::<T>, 
                            process_mouse_button_event::<T>,
                            process_mouse_wheel_event::<T>,
                        ));
    }
}

impl<T> Default for InputPlugin<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,
{
    fn default() -> Self {
        InputPlugin(PhantomData::<T>)
    }
}

fn process_input<T>(
    time: Res<Time>,
    mut button_control: ResMut<ButtonControl<T>>,
    mut writer: EventWriter<InputEvent<T>>,
) where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,
{
//    log::info!("process_input");
    button_control.process_input(time.delta_seconds_f64(), &mut writer);
}

fn process_keyboard_event<T>(
    mut button_control: ResMut<ButtonControl<T>>,
    mut reader: EventReader<KeyboardInput>,
    mut writer: EventWriter<InputEvent<T>>,
) where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,
{
//    log::info!("process_keyboard_event");
    button_control.process_keyboard(&mut reader, &mut writer);
}

fn process_mouse_button_event<T>(
    mut button_control: ResMut<ButtonControl<T>>,
    mut reader: EventReader<MouseButtonInput>,
    mut writer: EventWriter<InputEvent<T>>,
) where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,
{
  //  log::info!("process_mouse_button_event");
    button_control.process_mouse_button(&mut reader, &mut writer);
}

fn process_mouse_wheel_event<T>(
    mut button_control: ResMut<ButtonControl<T>>,
    mut reader: EventReader<MouseWheel>,
    mut writer: EventWriter<InputEvent<T>>,
) where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,
{
//    log::info!("process_mouse_wheel_event");
    button_control.process_mouse_wheel(&mut reader, &mut writer);
}




