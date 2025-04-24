use crate::{Axis, InputEvent, InputState, InputTable, MouseState, WheelState};
use corrosive_ecs_core_macro::Component;
use corrosive_ecs_renderer_backend::winit::dpi::PhysicalPosition;
use corrosive_ecs_renderer_backend::winit::event::MouseButton;
use corrosive_ecs_renderer_backend::winit::keyboard::KeyCode;
use std::collections::{HashMap, HashSet};

#[derive(Default, Component)]
pub struct Inputs {
    pub(crate) keys_down: HashSet<KeyCode>,
    pub(crate) keys_hold: HashSet<KeyCode>,
    pub(crate) keys_up: HashSet<KeyCode>,
    pub(crate) mouse_buttons_down: HashSet<MouseButton>,
    pub(crate) mouse_buttons_hold: HashSet<MouseButton>,
    pub(crate) mouse_buttons_up: HashSet<MouseButton>,
    pub(crate) mouse_position: PhysicalPosition<f64>,
    pub(crate) mouse_position_delta: PhysicalPosition<f64>,
    pub(crate) mouse_wheel: f32,
    pub(crate) input_tables: HashMap<String, InputTable>,
    pub(crate) active_table: String,
}
impl Inputs {
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.keys_hold.contains(&key)
    }
    pub fn is_key_up(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }
    pub fn is_mouse_button_down(&self, key: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&key)
    }
    pub fn is_mouse_button_held(&self, key: MouseButton) -> bool {
        self.mouse_buttons_hold.contains(&key)
    }
    pub fn is_mouse_button_up(&self, key: MouseButton) -> bool {
        self.mouse_buttons_up.contains(&key)
    }
    pub fn get_mouse_position(&self) -> Axis {
        Axis {
            x: self.mouse_position.x.clone(),
            y: self.mouse_position.y.clone(),
        }
    }
    pub fn get_mouse_delta(&self) -> Axis {
        Axis {
            x: self.mouse_position_delta.x.clone(),
            y: self.mouse_position_delta.y.clone(),
        }
    }
    pub fn get_mouse_wheel(&self) -> f32 {
        self.mouse_wheel.clone()
    }
    pub fn add_table(&mut self, name: &str, input_table: InputTable) {
        self.input_tables.insert(name.to_string(), input_table);
    }
    pub fn get_table(&self, name: &str) -> Option<&InputTable> {
        self.input_tables.get(name)
    }
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut InputTable> {
        self.input_tables.get_mut(name)
    }
    pub fn set_active_table(&mut self, name: &str) -> bool {
        let name = name.to_string();
        if self.input_tables.contains_key(&name.to_string()) {
            self.active_table = name;
            return true;
        }
        false
    }
    pub fn is_action_triggered(&self, table: &str, action: &str) -> bool {
        if let Some(t) = self.input_tables.get(&table.to_string()) {
            if let Some(t) = t.get_action_combinations(&action.to_string()) {
                for t in t {
                    let mut is_active = true;
                    for t in &t.comb {
                        match t {
                            InputEvent::KeyCode(t) => match t {
                                InputState::Down(t) => {
                                    if self.keys_down.contains(&t) {
                                        continue;
                                    }
                                }
                                InputState::Hold(t) => {
                                    if self.keys_hold.contains(&t) {
                                        continue;
                                    }
                                }
                                InputState::Up(t) => {
                                    if self.keys_up.contains(&t) {
                                        continue;
                                    }
                                }
                            },
                            InputEvent::MouseButton(t) => match t {
                                InputState::Down(t) => {
                                    if self.mouse_buttons_down.contains(&t) {
                                        continue;
                                    }
                                }
                                InputState::Hold(t) => {
                                    if self.mouse_buttons_hold.contains(&t) {
                                        continue;
                                    }
                                }
                                InputState::Up(t) => {
                                    if self.mouse_buttons_up.contains(&t) {
                                        continue;
                                    }
                                }
                            },
                            InputEvent::MouseState(t) => match t {
                                MouseState::MouseUp => {
                                    if self.mouse_position.y > 0f64 {
                                        continue;
                                    }
                                }
                                MouseState::MouseDown => {
                                    if self.mouse_position.y < 0f64 {
                                        continue;
                                    }
                                }
                                MouseState::MouseLeft => {
                                    if self.mouse_position.x < 0f64 {
                                        continue;
                                    }
                                }
                                MouseState::MouseRight => {
                                    if self.mouse_position.x > 0f64 {
                                        continue;
                                    }
                                }
                            },
                            InputEvent::WheelState(t) => match t {
                                WheelState::Up => {
                                    if self.mouse_wheel > 0f32 {
                                        continue;
                                    }
                                }
                                WheelState::Down => {
                                    if self.mouse_wheel < 0f32 {
                                        continue;
                                    }
                                }
                            },
                        }
                        is_active = false;
                        break;
                    }
                    if is_active {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub fn is_action_from_active_table(&self, action: &str) -> bool {
        self.is_action_triggered(self.active_table.as_str(), action)
    }
    pub fn get_mouse_keys(&self) -> HashSet<MouseButton> {
        let mut keys = HashSet::new();
        keys.extend(self.mouse_buttons_down.clone());
        keys.extend(self.mouse_buttons_hold.clone());
        keys.extend(self.mouse_buttons_up.clone());
        keys
    }
    pub fn get_keys(&self) -> HashSet<KeyCode> {
        let mut keys = HashSet::new();
        keys.extend(self.keys_up.clone());
        keys.extend(self.keys_hold.clone());
        keys.extend(self.keys_up.clone());
        keys
    }
    pub fn get_down_keys(&self) -> HashSet<KeyCode> {
        self.keys_down.clone()
    }
    pub fn get_hold_keys(&self) -> HashSet<KeyCode> {
        self.keys_hold.clone()
    }
    pub fn get_up_keys(&self) -> HashSet<KeyCode> {
        self.keys_up.clone()
    }
    pub fn get_mouse_up_keys(&self) -> HashSet<MouseButton> {
        self.mouse_buttons_up.clone()
    }
    pub fn get_mouse_hold_keys(&self) -> HashSet<MouseButton> {
        self.mouse_buttons_hold.clone()
    }
    pub fn get_mouse_down_keys(&self) -> HashSet<MouseButton> {
        self.mouse_buttons_down.clone()
    }
}
