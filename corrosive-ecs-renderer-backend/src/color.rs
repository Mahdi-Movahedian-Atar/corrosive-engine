use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub enum Color {
    RGBA(f32, f32, f32, f32),
    RGB(f32, f32, f32),
    HSVA(f32, f32, f32, f32),
    HSV(f32, f32, f32),
    GRAY(f32),
    OklabA(f32, f32, f32, f32),
    Oklab(f32, f32, f32),
    Hex(&'static str),
}
impl Default for Color {
    fn default() -> Self {
        Color::RGBA(1f32, 1f32, 1f32, 1f32)
    }
}

impl Color {
    pub fn from_hex(color: &str) -> Self {
        let mut hex_str = color;
        if hex_str.starts_with('#') {
            hex_str = &hex_str[1..];
        }
        let len = hex_str.len();
        match len {
            6 => Color::RGB(
                u8::from_str_radix(&hex_str[0..2], 16).unwrap_or(0) as f32 / 255.0,
                u8::from_str_radix(&hex_str[2..4], 16).unwrap_or(0) as f32 / 255.0,
                u8::from_str_radix(&hex_str[4..6], 16).unwrap_or(0) as f32 / 255.0,
            ),
            8 => Color::RGBA(
                u8::from_str_radix(&hex_str[0..2], 16).unwrap_or(0) as f32 / 255.0,
                u8::from_str_radix(&hex_str[2..4], 16).unwrap_or(0) as f32 / 255.0,
                u8::from_str_radix(&hex_str[4..6], 16).unwrap_or(0) as f32 / 255.0,
                u8::from_str_radix(&hex_str[6..8], 16).unwrap_or(0) as f32 / 255.0,
            ),
            _ => Color::RGB(0.0, 0.0, 0.0),
        }
    }
    pub fn to_rgba(&self) -> Self {
        match self {
            Color::RGBA(r, g, b, a) => Color::RGBA(*r, *g, *b, *a),
            Color::RGB(r, g, b) => Color::RGBA(*r, *g, *b, 1.0),
            Color::HSVA(h, s, v, a) => {
                let (r, g, b) = hsv_to_rgb(*h, *s, *v);
                Color::RGBA(r, g, b, *a)
            }
            Color::HSV(h, s, v) => {
                let (r, g, b) = hsv_to_rgb(*h, *s, *v);
                Color::RGBA(r, g, b, 1.0)
            }
            Color::GRAY(g) => Color::RGBA(*g, *g, *g, 1.0),
            Color::OklabA(l, a_ok, b_ok, alpha) => {
                let (r_lin, g_lin, b_lin) = oklab_to_linear_rgb(*l, *a_ok, *b_ok);
                let r = linear_to_srgb(r_lin);
                let g = linear_to_srgb(g_lin);
                let b = linear_to_srgb(b_lin);
                Color::RGBA(
                    r.clamp(0.0, 1.0),
                    g.clamp(0.0, 1.0),
                    b.clamp(0.0, 1.0),
                    *alpha,
                )
            }
            Color::Oklab(l, a_ok, b_ok) => {
                let (r_lin, g_lin, b_lin) = oklab_to_linear_rgb(*l, *a_ok, *b_ok);
                let r = linear_to_srgb(r_lin);
                let g = linear_to_srgb(g_lin);
                let b = linear_to_srgb(b_lin);
                Color::RGBA(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0)
            }
            Color::Hex(hex_str) => {
                let (r, g, b, a) = parse_hex(hex_str);
                Color::RGBA(r, g, b, a)
            }
        }
    }

    pub fn to_rgb(&self) -> Self {
        match self.to_rgba() {
            Color::RGBA(r, g, b, _) => Color::RGB(r, g, b),
            _ => self.to_rgba().to_rgba(),
        }
    }

    pub fn to_hsva(&self) -> Self {
        match self.to_rgba() {
            Color::RGBA(r, g, b, a) => {
                let (h, s, v) = rgb_to_hsv(r, g, b);
                Color::HSVA(h, s, v, a)
            }
            _ => self.to_rgba().to_hsva(),
        }
    }

    pub fn to_hsv(&self) -> Self {
        match self.to_rgba() {
            Color::RGBA(r, g, b, _) => {
                let (h, s, v) = rgb_to_hsv(r, g, b);
                Color::HSV(h, s, v)
            }
            _ => self.to_rgba().to_hsv(),
        }
    }

    pub fn to_gray(&self) -> Self {
        match self.to_rgba() {
            Color::RGBA(r, g, b, _) => {
                let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                Color::GRAY(gray.clamp(0.0, 1.0))
            }
            _ => self.to_rgba().to_gray(),
        }
    }

    pub fn to_oklaba(&self) -> Self {
        match self.to_rgba() {
            Color::RGBA(r, g, b, aa) => {
                let r_lin = srgb_to_linear(r);
                let g_lin = srgb_to_linear(g);
                let b_lin = srgb_to_linear(b);

                let x = 0.4124564 * r_lin + 0.3575761 * g_lin + 0.1804375 * b_lin;
                let y = 0.2126729 * r_lin + 0.7151522 * g_lin + 0.0721750 * b_lin;
                let z = 0.0193339 * r_lin + 0.1191920 * g_lin + 0.9503041 * b_lin;

                let lms_l = 0.819022437996703 * x + 0.3619062600528904 * y - 0.1288737815209879 * z;
                let lms_m =
                    0.032983653932388535 * x + 0.9292868615863434 * y + 0.03614466635064236 * z;
                let lms_s =
                    0.0481771893596242 * x + 0.2642395317527308 * y + 0.6335478284694309 * z;

                let lms_linear_l = lms_l.powf(1.0 / 3.0);
                let lms_linear_m = lms_m.powf(1.0 / 3.0);
                let lms_linear_s = lms_s.powf(1.0 / 3.0);

                let l = 0.2104542553 * lms_linear_l + 0.7936177749 * lms_linear_m
                    - 0.0040720468 * lms_linear_s;
                let a = 1.9779984951 * lms_linear_l - 2.4285922050 * lms_linear_m
                    + 0.4505937099 * lms_linear_s;
                let b = 0.0259040371 * lms_linear_l + 0.7827717662 * lms_linear_m
                    - 0.8086757660 * lms_linear_s;

                Color::OklabA(l, a, b, aa)
            }
            _ => self.to_rgba().to_oklaba(),
        }
    }

    pub fn to_oklab(&self) -> Self {
        match self.to_oklaba() {
            Color::OklabA(l, a, b, _) => Color::Oklab(l, a, b),
            _ => self.to_rgba().to_oklab(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Color::RGBA(r, g, b, a) => {
                let arr: [f32; 4] = [*r, *g, *b, *a];
                let bytes: &[u8] = bytemuck::cast_slice(&arr);
                bytes.to_vec()
            }
            _ => self.to_rgba().to_bytes(),
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        match self {
            Color::RGBA(r, g, b, a) => [*r, *g, *b, *a],
            _ => self.to_rgba().to_array(),
        }
    }
    pub fn to_array_u8(&self) -> [u8; 4] {
        match self {
            Color::RGBA(r, g, b, a) => [
                (r.clamp(0.0, 1.0) * 255.0).round() as u8,
                (g.clamp(0.0, 1.0) * 255.0).round() as u8,
                (b.clamp(0.0, 1.0) * 255.0).round() as u8,
                (a.clamp(0.0, 1.0) * 255.0).round() as u8,
            ],
            _ => self.to_rgba().to_array_u8(),
        }
    }
    pub fn mix(&self, other: &Color, amount: f32) -> Color {
        match self {
            Color::RGBA(r1, g1, b1, a1) => match other {
                Color::RGBA(r2, g2, b2, a2) => Color::RGBA(
                    r1 * (amount - 1.0) + r2 * amount,
                    g1 * (amount - 1.0) + g2 * amount,
                    b1 * (amount - 1.0) + b2 * amount,
                    a1 * (amount - 1.0) + a2 * amount,
                ),
                _ => self.mix(&other.to_rgba(), amount),
            },
            /*Color::RGB(r1, g1, b1) => {
                match other {
                    Color::RGB(r2, g2, b2) => {
                        Color::RGB(r1 * (amount-1.0) + r2 * amount,g1 * (amount-1.0) + g2 * amount,b1 * (amount-1.0) + b2 * amount)

                    }
                    _=> {
                        self.mix(&other.to_rgb(), amount)
                    }
                }
            }
            Color::HSVA(r1, g1, b1, a1) => {
                todo!()
            }
            Color::HSV(_, _, _) => {todo!()}
            Color::GRAY(_) => {todo!()}
            Color::OklabA(_, _, _, _) => {Color::default()}
            Color::Oklab(_, _, _) => {Color::default()}
            Color::Hex(_) => {Color::default()}*/
            _ => self.to_rgba().mix(&other, amount),
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = match (h / 60.0).floor() as i32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    (r_prime + m, g_prime + m, b_prime + m)
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    let s = if max == 0.0 { 0.0 } else { delta / max };

    (h, s, max)
}

fn oklab_to_linear_rgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let lms_l = l + 0.3963377774 * a + 0.2158037573 * b;
    let lms_m = l - 0.1055613458 * a - 0.0638541728 * b;
    let lms_s = l - 0.0894841775 * a - 1.2914855480 * b;

    let lms_linear_l = lms_l.powf(3.0);
    let lms_linear_m = lms_m.powf(3.0);
    let lms_linear_s = lms_s.powf(3.0);

    let x = 1.2270138511 * lms_linear_l - 0.5577989807 * lms_linear_m + 0.2812561490 * lms_linear_s;
    let y =
        -0.0405801784 * lms_linear_l + 1.1122568696 * lms_linear_m - 0.0716766787 * lms_linear_s;
    let z =
        -0.0763812845 * lms_linear_l - 0.4214819784 * lms_linear_m + 1.5861632204 * lms_linear_s;

    let r_lin = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let g_lin = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    let b_lin = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

    (r_lin, g_lin, b_lin)
}

fn linear_to_srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn parse_hex(hex: &str) -> (f32, f32, f32, f32) {
    let hex = hex.trim_start_matches('#');
    let len = hex.len();
    let hex_chars: Vec<char> = hex.chars().collect();

    let expanded: Vec<char> = if len == 3 || len == 4 {
        hex_chars.iter().flat_map(|&c| [c, c]).collect()
    } else {
        hex_chars
    };

    let components = match expanded.len() {
        6 => {
            let r = u8::from_str_radix(&expanded[0..2].iter().collect::<String>(), 16).unwrap_or(0);
            let g = u8::from_str_radix(&expanded[2..4].iter().collect::<String>(), 16).unwrap_or(0);
            let b = u8::from_str_radix(&expanded[4..6].iter().collect::<String>(), 16).unwrap_or(0);
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&expanded[0..2].iter().collect::<String>(), 16).unwrap_or(0);
            let g = u8::from_str_radix(&expanded[2..4].iter().collect::<String>(), 16).unwrap_or(0);
            let b = u8::from_str_radix(&expanded[4..6].iter().collect::<String>(), 16).unwrap_or(0);
            let a = u8::from_str_radix(&expanded[6..8].iter().collect::<String>(), 16).unwrap_or(0);
            (r, g, b, a)
        }
        _ => panic!("Invalid hex string length"),
    };

    let r = components.0 as f32 / 255.0;
    let g = components.1 as f32 / 255.0;
    let b = components.2 as f32 / 255.0;
    let a = components.3 as f32 / 255.0;

    (r, g, b, a)
}
