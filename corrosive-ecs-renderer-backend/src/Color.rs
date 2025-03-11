#[derive(Debug, Clone, Copy)]
pub enum Color<'a> {
    RGBA(f32, f32, f32, f32),
    RGB(f32, f32, f32),
    HSVA(f32, f32, f32, f32),
    HSV(f32, f32, f32),
    GRAY(f32),
    OklabA(f32, f32, f32, f32),
    Oklab(f32, f32, f32),
    Hex(&'a str),
}
impl Default for Color<'_> {
    fn default() -> Self {
        Color::RGBA(1.0, 1.0, 1.0, 1.0)
    }
}
/*impl Color<'_>{
    pub fn to_rgba(self) -> self {
        match &self {
            Color::RGBA(_, _, _, _) => {self}
            Color::RGB(r, g, b) => {Color::RGBA(*r, *g, *b, 1.0)}
            Color::HSVA(_, _, _, _) => {}
            Color::HSV(_, _, _) => {}
            Color::GRAY(_) => {}
            Color::OklabA(_, _, _, _) => {}
            Color::Oklab(_, _, _) => {}
            Color::Hex(_) => {}
        }
    }
}*/
