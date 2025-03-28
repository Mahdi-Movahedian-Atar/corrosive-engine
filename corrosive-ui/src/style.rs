/*#[derive(Default, Copy, Clone, Debug)]
pub enum Display {
    Sticky,
    Flex,
    Grid,
    Block,
    #[default]
    None,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum PositionType {
    Relative,
    #[default]
    Absolute,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum OverflowType {
    Visible,
    #[default]
    Hidden,
}
#[derive(Default, Copy, Clone, Debug)]
pub struct Overflow {
    pub x: OverflowType,
    pub y: OverflowType,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum Val {
    #[default]
    Px(u16),
    Per(u16),
    PerW(u16),
    PerH(u16),
}
#[derive(Default, Copy, Clone, Debug)]
pub enum Justify {
    Start,
    End,
    #[default]Center,
}
#[derive(Default, Copy, Clone, Debug)]
pub struct UiRect {
    pub left: Val,
    pub right: Val,
    pub top: Val,
    pub bottom: Val,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}
#[derive(Default, Copy, Clone, Debug)]
pub struct Style {
    pub display: Display,
    pub position_type: PositionType,
    pub overflow: Overflow,
    pub left: Val,
    pub right: Val,
    pub top: Val,
    pub bottom: Val,
    pub width: Val,
    pub height: Val,
    pub min_width: Val,
    pub min_height: Val,
    pub max_width: Val,
    pub max_height: Val,
    pub align_items: Justify,
    pub justify_items: Justify,
    pub align_self: Justify,
    pub justify_self: Justify,
    pub align_content: Justify,
    pub justify_content: Justify,
    pub margin: UiRect,
    pub padding: UiRect,
    pub border: UiRect,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Val,
    pub row_gap: Val,
    pub column_gap: Val,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_template_rows: Vec<RepeatedGridTrack>,
    pub grid_template_columns: Vec<RepeatedGridTrack>,
    pub grid_auto_rows: Vec<GridTrack>,
    pub grid_auto_columns: Vec<GridTrack>,
    pub grid_row: GridPlacement,
    pub grid_column: GridPlacement,
}*/
use corrosive_ecs_renderer_backend::color::Color;

#[derive(Copy, Clone, Debug)]
pub enum Val {
    Px(u32),
    Per(f32),
    PerW(f32),
    PerH(f32),
}
impl Default for Val {
    fn default() -> Self {
        Val::Per(1.0)
    }
}
#[derive(Default, Copy, Clone, Debug)]
pub enum Display {
    #[default]
    Block,
    Flex,
    Grid,
    Sticky,
    None,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum Overflow {
    #[default]
    Hidden,
    Visible,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum PositionType {
    #[default]
    Relative,
    Absolute,
    Floating(Val, Val),
}
#[derive(Default, Copy, Clone, Debug)]
pub enum Placement {
    #[default]
    Start,
    Center,
    End,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum Wrap {
    #[default]
    NoWrap,
    Warp,
}
#[derive(Default, Copy, Clone, Debug)]
pub struct Style<'a> {
    pub z_index: u32,
    pub min_width: Val,
    pub max_width: Val,
    pub width: Val,
    pub min_height: Val,
    pub max_height: Val,
    pub height: Val,
    pub margin_l: Val,
    pub margin_t: Val,
    pub margin_r: Val,
    pub margin_b: Val,
    pub border_l: Val,
    pub border_t: Val,
    pub border_r: Val,
    pub border_b: Val,
    pub corner_lt: Val,
    pub corner_rt: Val,
    pub corner_rb: Val,
    pub corner_lb: Val,
    pub display: Display,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub position_type_x: PositionType,
    pub position_type_y: PositionType,
    pub position_x: Placement,
    pub position_y: Placement,
    pub wrap: Wrap,
    pub basis: u16,
    pub background_color: Color<'a>,
    pub border_color_l: Color<'a>,
    pub border_color_t: Color<'a>,
    pub border_color_r: Color<'a>,
    pub border_color_b: Color<'a>,
}
