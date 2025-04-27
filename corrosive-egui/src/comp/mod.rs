use egui::TexturesDelta;
use corrosive_ecs_core_macro::Resource;

#[derive(Resource,Default)]
pub struct EguiObject{
    pub(crate) state: Option<egui_winit::State>,
    pub(crate) renderer: Option<egui_wgpu::Renderer>,
    pub(crate) textures: TexturesDelta
}