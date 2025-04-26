# Comp folder

1. To create components, resources, states, and traits the following rules must be applied, or the engine won't detect them.
2. All modules must be Directory modules. Modules files or nested modules within a file won't be detected by the engine.
   Modules must be public.
3. Use `component`, `state`, and `resource` to mark structs and enums to be used by the engine.
4. Use the `trait_bound` attribute macro to mark traits.
5. Use the `trait_for` macro to assign a component to a trait.
6. Implement the `SharedBehavior` trait to a component so they can be used in a hierarchy.

## Example:
```
#[derive(Resource, Default)]
pub struct Renderer2dData {
    pub(crate) data: Option<(Receiver<UnsafeRenderPass>, Sender<()>)>,
}

#[derive(Component)]
pub struct Sprite2D {
    offset: [f32; 2],
    texture: Asset<TextureAsset>,
    bind_group_layout_asset: Asset<BindGroupLayoutAsset>,
    bind_group: BindGroup,
    vertex_buffer: Buffer,
}

#[trait_bound]
pub trait Mesh2D {
    fn draw(&self, render_pass: &mut RenderPass);
    fn update(&self, render_pass: &mut RenderPass);
    fn name<'a>(&self) -> &'a str;
    fn get_bind_group_layout_desc(&self) -> 	&Asset<BindGroupLayoutAsset>;
}

trait_for!(trait Mesh2D => Sprite2D);
```