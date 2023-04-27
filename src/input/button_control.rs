use std::collections::HashMap;

use crate::{if_none_continue, if_none_return};
use super::InputEvent;
use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::*;
use bevy::input::mouse::*;
use core::fmt::Debug;
use core::hash::Hash;
use std::cmp::Eq;


#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum Notify {
    #[default]
    None,
    OnPress,
    OnActive,
    OnRelease,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct KeyState {
    pub pressed: bool,
    pub time: u64,
}

impl KeyState {
/*     pub fn new(
    ) -> Self {
        Self {
            pressed: false,
            time: 0,
        }
    } */

    pub fn change_state( &mut self, state: ButtonState ) {
        match state {
            ButtonState::Pressed => self.set_pressed(),
            ButtonState::Released => self.set_released(),             
        }
    }

    pub fn process(&mut self, delta_time: f64) {
        if self.pressed {
            self.add_delta_time(delta_time);
        }
    }

/*     pub fn change_state(
        &mut self, 
        state: ButtonState
    )  -> Option<f64> {
        match state {
            ButtonState::Pressed => { 
                self.set_pressed(); 

                if self.notify == Notify::OnPress {
                    return Some(0.);
                } 
            },
            ButtonState::Released => { 
                self.set_released(); 

                if self.notify == Notify::OnRelease {
                    return Some(self.get_delta_time());
                }
            },             
        }

        None
    }

    pub fn process(&mut self, delta_time: f64) -> Option<f64> {
        if self.pressed {
            self.add_delta_time(delta_time);

            if self.notify == Notify::OnActive {
                return Some(self.get_delta_time());
            }
        }

        None
    } */

    fn set_pressed(&mut self) {
        self.clear_delta_time();
        self.pressed = true;
    }

    fn set_released(&mut self) {
        self.pressed = false;
    }

    fn clear_delta_time(&mut self) {
        self.time = 0;
    }

    pub fn get_delta_time(&self) -> f64 {
        (self.time as f64) / 1000000. 
    }

    fn add_delta_time(&mut self, delta_time: f64) {
        self.time += (delta_time * 1000000.) as u64;
    } 
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum InputAction {
    Key(KeyCode),
    Mouse(MouseButton),
    Scroll,
}

impl From<KeyCode> for InputAction {
    fn from(key_code: KeyCode) -> Self {
        InputAction::Key(key_code)
    }
}

impl From<MouseButton> for InputAction {
    fn from(mouse_button: MouseButton) -> Self {
        InputAction::Mouse(mouse_button)
    }
}

#[derive(Component, Resource, Debug)]
pub struct ButtonControl<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,// + TryFrom<u16> + TryInto<u16>,
{
//    keys: HashMap<T, Vec<InputAction>>,
    keys: HashMap<T, Vec<InputAction>>,
    notifies: HashMap<T, Notify>,
    states: HashMap<InputAction, KeyState>,
    actions: HashMap<InputAction, Vec<T>>,
}

impl<T> Default for ButtonControl<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,// + TryFrom<u16> + TryInto<u16>,
{
    fn default() -> Self {
        ButtonControl::new()
    }
}

impl<T> ButtonControl<T>
where
    T: 'static + Send + Sync + Default + Debug + Eq + Hash + Clone,// + TryFrom<u16> + TryInto<u16>,
{
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            notifies: HashMap::new(),
            states: HashMap::new(),
            actions: HashMap::new(), 
        }
    }

    pub fn add_key(&mut self, name: T, key: KeyCode, notify: Notify) {
        self.add_action(name, InputAction::from(key), notify);
    }

    pub fn add_mouse(&mut self, name: T, button: MouseButton, notify: Notify) {
        //        log::info!("add_mouse {:?}", button);

        self.add_action(name, InputAction::from(button), notify);
    }

    pub fn add_action(&mut self, name: T, action: InputAction, notify: Notify) {
        //        log::info!("add_action {:?}", action);
         self.add_actions(name, vec![action], notify);
    }

    pub fn add_actions(&mut self, name: T, actions: Vec<InputAction>, notify: Notify) {
        //        log::info!("add_action_multy {:?}", action);

        self.keys.insert(name.clone(), actions.clone());

        self.notifies.insert(name.clone(), notify);

        for action in actions {
            if !self.states.contains_key(&action) {
                self.states.insert(action.clone(), KeyState::default());
            }

            if let Some(names) = self.actions.get_mut(&action) {
                if !names.contains(&name) {
                    names.push(name.clone());
                }
            } else {
                self.actions.insert(action, vec![name.clone()]);
            } 
        }

        //self.actions.insert(action, name);
    }

    pub fn remove_action(&mut self, name: T) {
        //        log::info!("remove_action {:?}", name);

        let actions = if_none_return!(self.keys.get_mut(&name)); 
        
        for action in actions {
            let names = if_none_return!(self.actions.get_mut(&action));

            names.retain(|n| n != &name);

            if names.is_empty() {
                self.actions.remove(&action);
                self.states.remove(&action);     
            }  
        }

        self.keys.remove(&name);  

        self.notifies.remove(&name); 
    }

/*     pub fn get_key_state(&self, name: T) -> Option<&KeyState> {
        self.states.get(&name)
    } */

    pub fn process_input(
        &mut self,
        delta_time: f64,
        writer: &mut EventWriter<InputEvent<T>>,
    ) {
        self.states.iter_mut().for_each( |(_, state)| {
            state.process(delta_time);
        });

        for (name, actions) in self.keys.iter() {
            let notify = if_none_continue!(self.notifies.get(&name));

            if *notify != Notify::OnActive {
                continue;
            }

            for action in actions.iter() {
                let state = if_none_continue!(self.states.get(&action));

                if !state.pressed {
                    break;
                }

                if action != actions.last().unwrap() {
                    continue;
                } 

                writer.send( InputEvent::<T> { name: name.clone(), value: state.get_delta_time() } );
            }
        }
    }

    fn process_action(
        &mut self,    
        current_action: InputAction,
        current_state: KeyState,
        writer: &mut EventWriter<InputEvent<T>>,
    ) {
            let names = if_none_return!(self.actions.get(&current_action));

            for name in names {
                if let Some(actions) = self.keys.get_mut(&name) {

                    if &current_action != actions.last().expect("button_control process_action err - no actions") {
                        continue;
                    }

                    for action in actions.iter() {
                        let state = if_none_continue!(self.states.get_mut(&action));                        

                        if action == actions.last().expect("button_control process_action err - no actions") {   
                            let notify = self.notifies.get(&name).expect("button_control process_action err - no notify");

                            if current_state.pressed {
                                if *notify == Notify::OnPress {
                                    writer.send( InputEvent::<T> { name: name.clone(), value: 0. } );
                                } 
                            } else {                     
                                if *notify == Notify::OnRelease {
                                    writer.send( InputEvent::<T> { name: name.clone(), value: state.get_delta_time() } );
                                }       
                            }

                            break;
                        }

                        if !state.pressed {
                            break;
                        }
                    }
                }
            }
    }

    pub fn process_keyboard(
        &mut self,  
        reader: &mut EventReader<KeyboardInput>,
        writer: &mut EventWriter<InputEvent<T>>,
    ) {
        for event in reader.iter() {  
            let key_code = if_none_continue!(event.key_code);

            let current_action = InputAction::from(key_code);

            {
                let current_state = if_none_continue!(self.states.get_mut(&current_action));
                current_state.change_state(event.state);
            }

            let current_state = if_none_continue!(self.states.get(&current_action));
            self.process_action(
                current_action,
                *current_state,
                writer,
            );
        }
    }
    
    pub fn process_mouse_button(
        &mut self,   
        reader: &mut EventReader<MouseButtonInput>,
        writer: &mut EventWriter<InputEvent<T>>,
    ) {
        for event in reader.iter() {  
            let current_action = InputAction::from(event.button);

            {
                let current_state = if_none_continue!(self.states.get_mut(&current_action));
                current_state.change_state(event.state);
            }

            let current_state = if_none_continue!(self.states.get(&current_action));
            self.process_action(
                current_action,
                *current_state,
                writer,
            );
        }
    }
    
    pub fn process_mouse_wheel(
        &mut self,
        reader: &mut EventReader<MouseWheel>,
        writer: &mut EventWriter<InputEvent<T>>,
    ) {
        for event in reader.iter() {  

            let names = if_none_continue!(self.actions.get_mut(&InputAction::Scroll));

            for name in names {
                let actions = if_none_continue!(self.keys.get(&name));

                for action in actions {                     

                    if action != actions.last().expect("button_control process_mouse_wheel err - no actions") {
                        let state = if_none_continue!(self.states.get(&action));  

                        if !state.pressed {
                            break;
                        }
                        continue;
                    } 
    
                    writer.send( InputEvent::<T> { name: name.clone(), value: event.y as f64 } );
                }
            }
        }
    } 
}
