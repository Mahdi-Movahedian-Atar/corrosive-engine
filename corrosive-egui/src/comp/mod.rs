use corrosive_ecs_core_macro::Resource;
use egui::{Context, TexturesDelta};
use std::cell::LazyCell;

#[derive(Resource)]
pub struct EguiObject {
    pub(crate) state: Option<(
        egui_winit::State,
        Box<dyn FnMut(&Context) + Send + Sync + 'static>,
    )>,
    pub(crate) renderer: Option<egui_wgpu::Renderer>,
    pub(crate) textures: TexturesDelta,
    pub(crate) input: bool,
}
impl Default for EguiObject {
    fn default() -> Self {
        EguiObject {
            state: None,
            renderer: None,
            textures: TexturesDelta::default(),
            input: true,
        }
    }
}
impl EguiObject {
    pub fn set_ui(&mut self, ui: impl FnMut(&Context) + Send + Sync + 'static) {
        if let Some((_, t)) = &mut self.state {
            *t = Box::new(ui);
        }
    }
    pub fn disable_input(&mut self) {
        self.input = false;
    }
    pub fn enable_input(&mut self) {
        self.input = true;
    }
}
