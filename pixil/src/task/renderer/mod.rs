use std::cell::LazyCell;
use std::collections::HashMap;
use std::sync::Mutex;
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::wgpu::{RenderBundleEncoder, TextureView};
use crate::render_set::RenderSet;

static mut DYNAMIC_OBJECTS:RenderSet<RenderBundleEncoder>= RenderSet::new() ;

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