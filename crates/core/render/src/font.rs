use crate::canvas::{Canvas, Cap, Join, Stroke};
use crate::color::Color;
use crate::raster::FillRule;
use std::sync::OnceLock;

pub trait OutlineFace: Send + Sync {
    fn advance_em(&self, c: char, bold: bool) -> f32;
    fn kern_em(&self, a: char, b: char, bold: bool) -> f32;
    fn contours_em(&self, c: char, bold: bool) -> Vec<Vec<(f32, f32)>>;
    fn em_per_size(&self) -> f32;
}

static FACE: OnceLock<Box<dyn OutlineFace>> = OnceLock::new();

pub fn install_face(face: Box<dyn OutlineFace>) -> bool {
    FACE.set(face).is_ok()
}

fn face() -> Option<&'static dyn OutlineFace> {
    FACE.get().map(|b| b.as_ref())
}

const BOLD_WEIGHT: f32 = 1.3;

fn outline_width(f: &dyn OutlineFace, text: &str, size: f32, bold: bool) -> f32 {
    let em = size * f.em_per_size();
    let mut w = 0.0;
    let mut prev: Option<char> = None;
    for c in text.chars() {
        if let Some(p) = prev {
            w += f.kern_em(p, c, bold) * em;
        }
        w += f.advance_em(c, bold) * em;
        prev = Some(c);
    }
    w
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub color: Color,
    pub anchor: Anchor,
    pub weight: f32,
}

impl TextStyle {
    pub fn new(size: f32, color: Color) -> TextStyle {
        TextStyle {
            size,
            color,
            anchor: Anchor::Start,
            weight: 1.0,
        }
    }

    pub fn anchor(mut self, anchor: Anchor) -> TextStyle {
        self.anchor = anchor;
        self
    }

    pub fn weight(mut self, weight: f32) -> TextStyle {
        self.weight = weight;
        self
    }
}

const GLYPHS: [&str; 95] = [
    "",
    "20 24|26 26",
    "10 11|30 31",
    "12 16|32 36|03 43|05 45",
    "41 30 10 01 12 34 45 36 16 05|20 26",
    "01 11 10 00 01|45 35 36 46 45|40 06",
    "46 02 01 10 20 31 32 04 05 16 26 44",
    "20 21",
    "30 11 15 36",
    "10 31 35 16",
    "20 24|01 43|41 03",
    "21 25|03 43",
    "25 26 17",
    "13 33",
    "26 26",
    "40 06",
    "10 01 05 16 36 45 41 30 10|14 32",
    "11 30 36|16 46",
    "01 10 30 41 42 06 46",
    "01 10 30 41 42 33 13|33 44 45 36 16 05",
    "30 03 43|30 36",
    "40 10 02 32 43 45 36 16 05",
    "40 20 01 05 16 36 45 43 32 12 03",
    "00 40 16",
    "10 01 02 13 33 42 41 30 10|13 04 05 16 36 45 44 33",
    "10 01 02 13 33 42 41 30 10|42 45 36 16 05",
    "22 22|26 26",
    "22 22|26 27 18",
    "40 03 46",
    "03 43|05 45",
    "00 43 06",
    "01 10 30 41 42 23 24|26 26",
    "46 16 05 01 10 30 41 44 35 25 14 12 22 42",
    "06 20 46|13 33",
    "00 06|00 30 41 42 33 03|33 44 45 36 06",
    "41 30 10 01 05 16 36 45",
    "00 06 26 45 41 20 00",
    "40 00 06 46|03 33",
    "40 00 06|03 33",
    "41 30 10 01 05 16 36 45 43 23",
    "00 06|40 46|03 43",
    "10 30|20 26|16 36",
    "40 45 36 16 05",
    "00 06|40 03|13 46",
    "00 06 46",
    "06 00 22 40 46",
    "06 00 46 40",
    "10 01 05 16 36 45 41 30 10",
    "06 00 30 41 42 33 03",
    "10 01 05 16 36 45 41 30 10|23 46",
    "06 00 30 41 42 33 03|23 46",
    "41 30 10 01 12 34 45 36 16 05",
    "00 40|20 26",
    "00 05 16 36 45 40",
    "00 26 40",
    "00 06 22 46 40",
    "00 46|40 06",
    "00 23 40|23 26",
    "00 40 06 46",
    "30 10 16 36",
    "00 46",
    "10 30 36 16",
    "02 20 42",
    "07 47",
    "10 21",
    "12 32 43 46|43 24 05 16 36 46",
    "00 06|02 32 43 45 36 06",
    "42 12 03 05 16 46",
    "40 46|42 12 03 05 16 46",
    "03 43 32 12 03 05 16 46",
    "40 20 11 16|02 32",
    "42 12 03 05 16 46|42 47 38 18",
    "00 06|02 32 43 46",
    "12 22 26|16 36|20 21",
    "32 37 28 18 07|30 31",
    "00 06|42 04|14 46",
    "10 20 26|16 36",
    "02 06|03 12 22 26|23 32 42 46",
    "02 06|03 12 32 43 46",
    "12 03 05 16 36 45 43 32 12",
    "02 08|02 32 43 45 36 06",
    "42 48|42 12 03 05 16 46",
    "02 06|03 12 32 41",
    "42 12 03 14 34 45 36 06",
    "20 25 36 46|02 42",
    "02 05 16 36 46|42 46",
    "02 26 42",
    "02 16 22 36 42",
    "02 46|42 06",
    "02 05 16 36 46|42 47 38 18",
    "02 42 06 46",
    "30 20 21 22 13 24 25 26 36",
    "20 27",
    "10 20 21 22 33 24 25 26 16",
    "03 12 22 33 43",
];

const UNIT_ADVANCE: f32 = 5.0;
const UNIT_DESCENT: f32 = 8.0;
const UNIT_BASELINE: f32 = 6.0;

pub fn glyph_strokes(c: char) -> Vec<Vec<(f32, f32)>> {
    let idx = if (' '..='~').contains(&c) {
        c as usize - 32
    } else {
        '?' as usize - 32
    };
    let mut out = Vec::new();
    for stroke in GLYPHS[idx].split('|') {
        let mut pts = Vec::new();
        for tok in stroke.split(' ') {
            let b = tok.as_bytes();
            if b.len() == 2 {
                pts.push(((b[0] - b'0') as f32, (b[1] - b'0') as f32));
            }
        }
        if !pts.is_empty() {
            out.push(pts);
        }
    }
    out
}

pub fn measure_text(text: &str, size: f32) -> (f32, f32) {
    if let Some(f) = face() {
        return (outline_width(f, text, size, false), size);
    }
    let n = text.chars().count() as f32;
    let unit = size / UNIT_DESCENT;
    ((n * UNIT_ADVANCE - if n > 0.0 { 1.0 } else { 0.0 }) * unit, size)
}

fn draw_outline(f: &dyn OutlineFace, canvas: &mut Canvas, x: f32, y: f32, text: &str, style: &TextStyle) {
    let bold = style.weight >= BOLD_WEIGHT;
    let em = style.size * f.em_per_size();
    let w = outline_width(f, text, style.size, bold);
    let mut pen = match style.anchor {
        Anchor::Start => x,
        Anchor::Middle => x - w / 2.0,
        Anchor::End => x - w,
    };
    let mut polys: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut prev: Option<char> = None;
    for c in text.chars() {
        if let Some(p) = prev {
            pen += f.kern_em(p, c, bold) * em;
        }
        for contour in f.contours_em(c, bold) {
            polys.push(
                contour
                    .iter()
                    .map(|&(gx, gy)| (pen + gx * em, y - gy * em))
                    .collect(),
            );
        }
        pen += f.advance_em(c, bold) * em;
        prev = Some(c);
    }
    canvas.fill_contours(&polys, FillRule::NonZero, style.color);
}

pub(crate) fn draw_text(canvas: &mut Canvas, x: f32, y: f32, text: &str, style: &TextStyle) {
    if let Some(f) = face() {
        draw_outline(f, canvas, x, y, text, style);
        return;
    }
    let unit = style.size / UNIT_DESCENT;
    let (w, _) = measure_text(text, style.size);
    let x0 = match style.anchor {
        Anchor::Start => x,
        Anchor::Middle => x - w / 2.0,
        Anchor::End => x - w,
    };
    let width = (style.size / 9.0 * style.weight).max(0.9);
    let stroke = Stroke::new(style.color, width)
        .with_cap(Cap::Round)
        .with_join(Join::Round);
    let top = y - UNIT_BASELINE * unit;
    for (i, ch) in text.chars().enumerate() {
        let ox = x0 + i as f32 * UNIT_ADVANCE * unit;
        for pts in glyph_strokes(ch) {
            let mapped: Vec<(f32, f32)> = pts
                .iter()
                .map(|&(gx, gy)| (ox + gx * unit + width * 0.5, top + gy * unit))
                .collect();
            canvas.polyline(&mapped, &stroke);
        }
    }
}
