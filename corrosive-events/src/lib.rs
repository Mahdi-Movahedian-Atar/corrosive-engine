use corrosive_ecs_core_macro::corrosive_engine_builder;
use corrosive_ecs_renderer_backend::winit::event::MouseButton;
use corrosive_ecs_renderer_backend::winit::keyboard::KeyCode;
use std::collections::hash_set::Iter;
use std::collections::{HashMap, HashSet};
use std::iter::Map;

pub use corrosive_ecs_renderer_backend::winit::event;

pub mod comp;
pub mod task;

corrosive_engine_builder!(
    setup "start_events",
    sync_update "update_events"
);

pub struct Axis {
    pub x: f64,
    pub y: f64,
}
impl Axis {
    pub fn into_tuple(self) -> (f64, f64) {
        (self.x, self.y)
    }
}
#[derive(Eq, Hash, Ord, PartialEq, Clone, PartialOrd)]
pub enum InputState<T> {
    Down(T),
    Hold(T),
    Up(T),
}
#[derive(Eq, Hash, Ord, PartialEq, Clone, PartialOrd)]
pub enum MouseState {
    MouseUp,
    MouseDown,
    MouseLeft,
    MouseRight,
}
#[derive(Eq, Hash, Ord, PartialEq, Clone, PartialOrd)]
pub enum WheelState {
    Up,
    Down,
}
#[derive(Eq, Hash, Ord, PartialEq, Clone, PartialOrd)]
pub enum InputEvent {
    KeyCode(InputState<KeyCode>),
    MouseButton(InputState<MouseButton>),
    MouseState(MouseState),
    WheelState(WheelState),
}
#[derive(Eq, Hash, PartialEq, Clone)]
pub struct InputCombination {
    comb: Vec<InputEvent>,
}
impl InputCombination {
    pub fn get_all_combinations<'a>(&self) -> Vec<Vec<InputEvent>> {
        let mut combos: Vec<InputEvent> = Vec::new();
        self.comb
            .iter()
            .map(|x| {
                combos.push(x.clone());
                combos.sort();
                combos.clone()
            })
            .collect()
    }
}
impl InputCombination {
    pub fn new(comb: &[InputEvent]) -> Self {
        let mut comb = InputCombination {
            comb: Vec::from(comb),
        };
        comb.comb.sort();
        comb
    }
    pub fn add(&mut self, input_event: InputEvent) {
        self.comb.push(input_event);
        self.comb.sort();
    }
    pub fn remove_by_index(&mut self, index: usize) {
        self.comb.remove(index);
        self.comb.sort();
    }
    pub fn remove(&mut self, input_event: InputEvent) {
        while let Some(t) = self.comb.iter().position(|x| x == &input_event) {
            self.comb.remove(t);
        }
        self.comb.sort();
    }
    pub fn get(&self) -> &Vec<InputEvent> {
        &self.comb
    }
}
pub struct InputTable {
    input_combination: HashMap<String, Vec<InputCombination>>,
}
impl InputTable {
    pub fn add_action(&mut self, name: &str, input_combination: InputCombination) {
        if let Some(t) = self.input_combination.get_mut(name) {
            t.push(input_combination);
        } else {
            self.input_combination
                .insert(name.to_string(), vec![input_combination]);
        }
    }
    pub fn get_action_combinations(&self, name: &str) -> Option<&Vec<InputCombination>> {
        self.input_combination.get(name)
    }
    pub fn get_action_combinations_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut Vec<InputCombination>> {
        self.input_combination.get_mut(name)
    }
}
