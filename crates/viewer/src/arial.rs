use ab_glyph::{Font, FontVec, OutlineCurve, Point};
use render::OutlineFace;
use std::collections::BTreeMap;
use std::sync::Mutex;

type Contours = Vec<Vec<(f32, f32)>>;

const CURVE_STEPS: usize = 6;
const EM_PER_SIZE: f32 = 1.05;

struct Face {
    font: FontVec,
    upem: f32,
}

pub struct Arial {
    regular: Face,
    bold: Option<Face>,
    cache: Mutex<BTreeMap<(char, bool), Contours>>,
}

fn load(path: &str) -> Option<Face> {
    let data = std::fs::read(path).ok()?;
    let font = FontVec::try_from_vec(data).ok()?;
    let upem = font.units_per_em()?;
    Some(Face { font, upem })
}

fn fonts_dir() -> String {
    let root = std::env::var("WINDIR").unwrap_or_else(|_| "C:/Windows".to_string());
    format!("{root}/Fonts")
}

impl Arial {
    pub fn load() -> Option<Arial> {
        let dir = fonts_dir();
        let regular = load(&format!("{dir}/arial.ttf"))?;
        let bold = load(&format!("{dir}/arialbd.ttf"));
        Some(Arial {
            regular,
            bold,
            cache: Mutex::new(BTreeMap::new()),
        })
    }

    fn face(&self, bold: bool) -> &Face {
        match (&self.bold, bold) {
            (Some(b), true) => b,
            _ => &self.regular,
        }
    }
}

fn lerp(a: Point, b: Point, t: f32) -> Point {
    Point {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

fn flatten(curve: &OutlineCurve, out: &mut Vec<Point>) {
    match *curve {
        OutlineCurve::Line(_, p1) => out.push(p1),
        OutlineCurve::Quad(p0, p1, p2) => {
            for k in 1..=CURVE_STEPS {
                let t = k as f32 / CURVE_STEPS as f32;
                out.push(lerp(lerp(p0, p1, t), lerp(p1, p2, t), t));
            }
        }
        OutlineCurve::Cubic(p0, p1, p2, p3) => {
            for k in 1..=CURVE_STEPS {
                let t = k as f32 / CURVE_STEPS as f32;
                let a = lerp(lerp(p0, p1, t), lerp(p1, p2, t), t);
                let b = lerp(lerp(p1, p2, t), lerp(p2, p3, t), t);
                out.push(lerp(a, b, t));
            }
        }
    }
}

fn start_of(curve: &OutlineCurve) -> Point {
    match *curve {
        OutlineCurve::Line(p, _) | OutlineCurve::Quad(p, _, _) | OutlineCurve::Cubic(p, _, _, _) => p,
    }
}

fn same(a: Point, b: Point) -> bool {
    (a.x - b.x).abs() < 1.0e-3 && (a.y - b.y).abs() < 1.0e-3
}

fn contours(face: &Face, c: char) -> Vec<Vec<(f32, f32)>> {
    let id = face.font.glyph_id(c);
    let Some(outline) = face.font.outline(id) else {
        return Vec::new();
    };
    let mut out: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut cur: Vec<Point> = Vec::new();
    for curve in &outline.curves {
        let p0 = start_of(curve);
        if cur.last().is_none_or(|last| !same(*last, p0)) {
            if cur.len() > 2 {
                out.push(cur.iter().map(|p| (p.x / face.upem, p.y / face.upem)).collect());
            }
            cur = vec![p0];
        }
        flatten(curve, &mut cur);
    }
    if cur.len() > 2 {
        out.push(cur.iter().map(|p| (p.x / face.upem, p.y / face.upem)).collect());
    }
    out
}

impl OutlineFace for Arial {
    fn advance_em(&self, c: char, bold: bool) -> f32 {
        let f = self.face(bold);
        f.font.h_advance_unscaled(f.font.glyph_id(c)) / f.upem
    }

    fn kern_em(&self, a: char, b: char, bold: bool) -> f32 {
        let f = self.face(bold);
        f.font.kern_unscaled(f.font.glyph_id(a), f.font.glyph_id(b)) / f.upem
    }

    fn contours_em(&self, c: char, bold: bool) -> Vec<Vec<(f32, f32)>> {
        let key = (c, bold && self.bold.is_some());
        if let Ok(cache) = self.cache.lock()
            && let Some(v) = cache.get(&key)
        {
            return v.clone();
        }
        let v = contours(self.face(bold), c);
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, v.clone());
        }
        v
    }

    fn em_per_size(&self) -> f32 {
        EM_PER_SIZE
    }
}

pub fn install() -> bool {
    match Arial::load() {
        Some(a) => render::install_face(Box::new(a)),
        None => false,
    }
}
