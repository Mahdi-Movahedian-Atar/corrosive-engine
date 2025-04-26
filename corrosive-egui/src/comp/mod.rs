/*use egui::{Context, TexturesDelta};
use egui_winit::State as EguiState;
use egui_wgpu::Renderer as EguiRenderer;
struct EguiIntegration {
    ctx: Context,
    state: EguiState,
    renderer: EguiRenderer,
    textures: TexturesDelta,
}
impl Default for EguiIntegration {
    fn default() -> Self {
        Self {
            ctx: Context::default(),
            state: EguiState::default(),
            renderer: EguiRenderer::default(),
            textures: TexturesDelta::default(),
        }
    }
}*/
use egui::TexturesDelta;
use corrosive_ecs_core_macro::Resource;

#[derive(Resource,Default)]
pub struct EguiObject{
    pub(crate) state: Option<egui_winit::State>,
    pub(crate) renderer: Option<egui_wgpu::Renderer>,
    pub(crate) textures: TexturesDelta
}