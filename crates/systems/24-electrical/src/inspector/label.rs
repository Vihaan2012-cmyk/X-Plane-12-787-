use super::topo::{VH, VW};
use render::measure_text;

#[derive(Clone, Copy, Debug)]
pub struct Box2 {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Box2 {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Box2 {
        Box2 { x0, y0, x1, y1 }
    }

    pub fn text(x: f32, baseline: f32, w: f32, size: f32) -> Box2 {
        Box2::new(
            x - 3.5,
            baseline - 0.75 * size - 1.0,
            x + w + 3.5,
            baseline + 0.25 * size + 1.0,
        )
    }

    fn overlaps(&self, o: &Box2) -> bool {
        self.x0 < o.x1 && o.x0 < self.x1 && self.y0 < o.y1 && o.y0 < self.y1
    }

    fn hit_segment(&self, a: (f32, f32), b: (f32, f32)) -> bool {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let mut t0 = 0.0f32;
        let mut t1 = 1.0f32;
        let checks = [
            (-dx, a.0 - self.x0),
            (dx, self.x1 - a.0),
            (-dy, a.1 - self.y0),
            (dy, self.y1 - a.1),
        ];
        for (p, q) in checks {
            if p == 0.0 {
                if q < 0.0 {
                    return false;
                }
            } else {
                let r = q / p;
                if p < 0.0 {
                    if r > t1 {
                        return false;
                    }
                    t0 = t0.max(r);
                } else {
                    if r < t0 {
                        return false;
                    }
                    t1 = t1.min(r);
                }
            }
        }
        t0 <= t1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub x: f32,
    pub y: f32,
}

pub struct Placer {
    blocks: Vec<Box2>,
    segs: Vec<((f32, f32), (f32, f32))>,
}

impl Placer {
    pub fn new() -> Placer {
        Placer {
            blocks: Vec::new(),
            segs: Vec::new(),
        }
    }

    pub fn block(&mut self, b: Box2) {
        self.blocks.push(b);
    }

    pub fn wire(&mut self, pts: &[(f32, f32)]) {
        for w in pts.windows(2) {
            self.segs.push((w[0], w[1]));
        }
    }

    fn free(&self, r: &Box2) -> bool {
        if r.x0 < 0.0 || r.y0 < 0.0 || r.x1 > VW || r.y1 > VH {
            return false;
        }
        if self.blocks.iter().any(|b| b.overlaps(r)) {
            return false;
        }
        !self.segs.iter().any(|&(a, b)| r.hit_segment(a, b))
    }

    pub fn place(&mut self, text: &str, size: f32, cands: &[Spot]) -> Option<Spot> {
        let w = measure_text(text, size).0;
        for s in cands {
            let r = Box2::text(s.x, s.y, w, size);
            if self.free(&r) {
                self.blocks.push(r);
                return Some(*s);
            }
        }
        None
    }

    pub fn place_led(
        &mut self,
        text: &str,
        size: f32,
        at: (f32, f32),
        cands: &[Spot],
    ) -> Option<(Spot, Option<(f32, f32)>)> {
        let w = measure_text(text, size).0;
        for s in cands {
            let r = Box2::text(s.x, s.y, w, size);
            if !self.free(&r) {
                continue;
            }
            let near = (
                at.0.clamp(s.x, s.x + w),
                at.1.clamp(s.y - 0.75 * size, s.y + 0.25 * size),
            );
            let d = ((near.0 - at.0).powi(2) + (near.1 - at.1).powi(2)).sqrt();
            if d <= 11.0 {
                self.blocks.push(r);
                return Some((*s, None));
            }
            let lead = Box2::new(
                at.0.min(near.0) - 0.5,
                at.1.min(near.1) - 0.5,
                at.0.max(near.0) + 0.5,
                at.1.max(near.1) + 0.5,
            );
            if self.blocks.iter().any(|b| {
                b.overlaps(&lead) && !b.overlaps(&Box2::new(at.0 - 7.0, at.1 - 7.0, at.0 + 7.0, at.1 + 7.0))
            }) {
                continue;
            }
            self.blocks.push(r);
            self.blocks.push(lead);
            return Some((*s, Some(near)));
        }
        None
    }
}

impl Default for Placer {
    fn default() -> Self {
        Placer::new()
    }
}

pub fn around(x: f32, y: f32, w: f32, size: f32, gap: f32) -> Vec<Spot> {
    let mut v = ring(x, y, w, size, gap);
    v.extend(ring(x, y, w, size, gap + 9.0));
    v.extend(ring(x, y, w, size, gap + 18.0));
    v
}

fn ring(x: f32, y: f32, w: f32, size: f32, gap: f32) -> Vec<Spot> {
    let mid = 0.25 * size;
    let up = y - gap - 0.25 * size;
    let down = y + gap + 0.75 * size;
    let shift = gap * 0.6;
    vec![
        Spot {
            x: x + gap,
            y: y + mid,
        },
        Spot {
            x: x - gap - w,
            y: y + mid,
        },
        Spot {
            x: x - w / 2.0,
            y: up,
        },
        Spot {
            x: x - w / 2.0,
            y: down,
        },
        Spot { x: x + shift, y: up },
        Spot {
            x: x - shift - w,
            y: up,
        },
        Spot {
            x: x + shift,
            y: down,
        },
        Spot {
            x: x - shift - w,
            y: down,
        },
        Spot {
            x: x + gap,
            y: y - 3.0 + mid,
        },
        Spot {
            x: x + gap,
            y: y + 3.0 + mid,
        },
        Spot {
            x: x - gap - w,
            y: y - 3.0 + mid,
        },
        Spot {
            x: x - gap - w,
            y: y + 3.0 + mid,
        },
    ]
}

pub fn along(x0: f32, x1: f32, y: f32, w: f32, size: f32, above_first: bool) -> Vec<Spot> {
    let up = y - 5.0 - 0.25 * size;
    let down = y + 5.0 + 0.75 * size;
    let (first, second) = if above_first { (up, down) } else { (down, up) };
    let lo = x0 - w * 0.4;
    let hi = (x1 - w * 0.6).max(lo);
    let mut xs: Vec<f32> = Vec::new();
    let mut x = lo;
    while x <= hi + 0.01 {
        xs.push(x);
        x += 3.0;
    }
    xs.sort_by(|a, b| (a - x0).abs().total_cmp(&(b - x0).abs()));
    let mut out = Vec::new();
    for &yy in &[first, second] {
        for &xx in &xs {
            out.push(Spot { x: xx, y: yy });
        }
    }
    out
}
