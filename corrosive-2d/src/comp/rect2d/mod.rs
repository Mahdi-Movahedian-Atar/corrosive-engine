use crate::comp::Mesh2D;
use corrosive_asset_manager::asset_server::Asset;
use corrosive_ecs_core::trait_for;
use corrosive_ecs_core_macro::Component;
use corrosive_ecs_renderer_backend::assets::BindGroupLayoutAsset;
use corrosive_ecs_renderer_backend::helper::RenderPass;

#[derive(Component)]
pub struct Rect2D {
    offset: [f32; 2],
}
trait_for!(trait Mesh2D => Rect2D);
impl Mesh2D for Rect2D {
    fn draw(&self) {
        todo!()
    }

    fn update(&self, render_pass: &mut RenderPass) {
        todo!()
    }

    fn name<'a>(&self) -> &'a str {
        todo!()
    }

    fn get_bind_group_layout_desc(&self) -> Asset<BindGroupLayoutAsset> {
        todo!()
    }
}
