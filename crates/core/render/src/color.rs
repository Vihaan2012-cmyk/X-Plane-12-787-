#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
    pub const BG: Color = Color::rgb(0x12, 0x12, 0x12);
    pub const PANEL: Color = Color::rgb(0x18, 0x18, 0x18);
    pub const PANEL2: Color = Color::rgb(0x1f, 0x1f, 0x1f);
    pub const LINE: Color = Color::rgb(0x33, 0x33, 0x33);
    pub const DIM: Color = Color::rgb(0x8f, 0x8f, 0x8f);
    pub const FAINT: Color = Color::rgb(0x55, 0x55, 0x55);
    pub const BAD: Color = Color::rgb(0xe0, 0x55, 0x55);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub const fn hex(v: u32) -> Color {
        Color::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }

    pub const fn with_alpha(self, a: u8) -> Color {
        Color { a, ..self }
    }

    pub fn lerp(self, o: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t + 0.5) as u8;
        Color {
            r: m(self.r, o.r),
            g: m(self.g, o.g),
            b: m(self.b, o.b),
            a: m(self.a, o.a),
        }
    }
}
