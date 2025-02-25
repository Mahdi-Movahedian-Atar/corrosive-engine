use corrosive_ecs_core::ecs_core::LockedRef;

#[derive(Default)]
struct SimpleBox {
    tl: [u16; 2],
    br: [u16; 2],
    corners: [u16; 4],
    edges: [u16; 4],
    background_color: [f32; 3],
    border_color: [f32; 3],
}

#[derive(Default)]
pub enum LenType {
    PX(u16),
    PER(f32),
    #[default]
    None,
}

#[derive(Default)]
struct UIBox {
    pub name: String,
    width: LenType,
    height: LenType,
    left: LenType,
    right: LenType,
    top: LenType,
    bottom: LenType,
    corners: [LenType; 4],
    edges: [LenType; 4],
    background_color: [f32; 3],
    border_color: [f32; 3],
    visible: bool,
    children: Vec<LockedRef<UIBox>>,
    element: SimpleBox,
}

impl UIBox {
    pub fn new(name: String) -> UIBox {
        UIBox {
            name,
            ..UIBox::default()
        }
    }
}
