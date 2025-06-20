use crate::comp::Inputs;
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::WindowOptions;
use corrosive_ecs_renderer_backend::winit::dpi::PhysicalPosition;
use corrosive_ecs_renderer_backend::winit::event::{
    ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
};
use corrosive_ecs_renderer_backend::winit::event_loop::ActiveEventLoop;
use corrosive_ecs_renderer_backend::winit::keyboard::{
    KeyCode, ModifiersState, NativeKeyCode, PhysicalKey,
};
use corrosive_ecs_renderer_backend::winit::window::WindowId;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Default)]
struct InputStorage {
    keys_down: HashSet<KeyCode>,
    keys_hold: HashSet<KeyCode>,
    keys_up: HashSet<KeyCode>,
    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_hold: HashSet<MouseButton>,
    mouse_buttons_up: HashSet<MouseButton>,
    mouse_position: PhysicalPosition<f64>,
    mouse_wheel: f32,
}

pub static INPUT_STORAGE: LazyLock<Mutex<InputStorage>> =
    LazyLock::new(|| Mutex::new(Default::default()));

#[task]
pub fn start_events(window_options: Res<WindowOptions>) {
    window_options
        .f_write()
        .func
        .push(Arc::new(|app, event_loop, window_id, event| match event {
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let mut lock = INPUT_STORAGE.lock().unwrap();
                match event.state {
                    ElementState::Pressed => match event.physical_key {
                        PhysicalKey::Code(c) => {
                            lock.keys_hold.insert(c);
                            lock.keys_down.insert(c);
                        }
                        _ => {}
                    },
                    ElementState::Released => match event.physical_key {
                        PhysicalKey::Code(c) => {
                            lock.keys_hold.remove(&c);
                            lock.keys_up.insert(c);
                        }
                        _ => {}
                    },
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                INPUT_STORAGE.lock().unwrap().mouse_position = *position;
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                let scroll_amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => &(pos.y as f32),
                };
                INPUT_STORAGE.lock().unwrap().mouse_wheel += scroll_amount;
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let mut lock = INPUT_STORAGE.lock().unwrap();
                match state {
                    ElementState::Pressed => {
                        lock.mouse_buttons_hold.insert(button.clone());
                        lock.mouse_buttons_down.insert(button.clone());
                    }
                    ElementState::Released => {
                        lock.mouse_buttons_hold.remove(button);
                        lock.mouse_buttons_up.insert(button.clone());
                    }
                }
            }
            _ => {}
        }))
}

#[task]
pub fn update_events(input_res: Res<Inputs>) {
    let mut lock = INPUT_STORAGE.lock().unwrap();
    let mut input_res = input_res.f_write();

    input_res.mouse_position_delta = PhysicalPosition {
        x: input_res.mouse_position.x - lock.mouse_position.x,
        y: input_res.mouse_position.y - lock.mouse_position.y,
    };
    input_res.keys_down = lock.keys_down.clone();
    input_res.keys_hold = lock.keys_hold.clone();
    input_res.keys_up = lock.keys_up.clone();
    input_res.mouse_buttons_down = lock.mouse_buttons_down.clone();
    input_res.mouse_buttons_hold = lock.mouse_buttons_hold.clone();
    input_res.mouse_buttons_up = lock.mouse_buttons_up.clone();
    input_res.mouse_position = lock.mouse_position.clone();
    input_res.mouse_wheel = lock.mouse_wheel;
    lock.keys_up = HashSet::new();
    lock.keys_down = HashSet::new();
    lock.mouse_buttons_up = HashSet::new();
    lock.mouse_buttons_down = HashSet::new();
    lock.mouse_wheel = 0f32;
}
