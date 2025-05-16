use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::wgpu::{RenderBundleEncoder, TextureView};

static mut DYNAMIC_OBJECTS: Vec<RenderBundleEncoder<'static>> = Vec::new();
static mut DYNAMIC_OBJECTS_INDEX: Vec<u64> = Vec::new();

struct RenderPixilNode{

}
impl RenderNode for RenderPixilNode{
    fn name(&self) -> &str {
        "RenderPixilNode"
    }

    fn execute(&self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, view: &TextureView, depth_view: &TextureView) {
        todo!()
    }
}

#[task]
pub fn start_pixil_renderer(){
}