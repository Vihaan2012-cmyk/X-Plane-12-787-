use crate::color::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

pub(crate) const SUB: usize = 6;

pub(crate) struct Edge {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub dir: i32,
}

pub(crate) fn build_edges(polys: &[Vec<(f32, f32)>]) -> Vec<Edge> {
    let mut edges = Vec::new();
    for p in polys {
        let n = p.len();
        if n < 3 {
            continue;
        }
        for i in 0..n {
            let (ax, ay) = p[i];
            let (bx, by) = p[(i + 1) % n];
            if ay == by || !ax.is_finite() || !ay.is_finite() || !bx.is_finite() || !by.is_finite() {
                continue;
            }
            if ay < by {
                edges.push(Edge {
                    x0: ax,
                    y0: ay,
                    x1: bx,
                    y1: by,
                    dir: 1,
                });
            } else {
                edges.push(Edge {
                    x0: bx,
                    y0: by,
                    x1: ax,
                    y1: ay,
                    dir: -1,
                });
            }
        }
    }
    edges
}

pub(crate) struct Surface<'a> {
    pub width: usize,
    pub pixels: &'a mut [u8],
}

pub(crate) fn fill_edges(
    surf: &mut Surface,
    edges: &[Edge],
    rule: FillRule,
    paint: Color,
    clip: (i32, i32, i32, i32),
) {
    if edges.is_empty() || paint.a == 0 {
        return;
    }
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    for e in edges {
        min_y = min_y.min(e.y0);
        max_y = max_y.max(e.y1);
        min_x = min_x.min(e.x0.min(e.x1));
        max_x = max_x.max(e.x0.max(e.x1));
    }
    let (cx0, cy0, cx1, cy1) = clip;
    let row0 = (min_y.floor() as i64).max(cy0 as i64);
    let row1 = (max_y.ceil() as i64).min(cy1 as i64);
    let col0 = (min_x.floor() as i64).max(cx0 as i64);
    let col1 = (max_x.ceil() as i64).min(cx1 as i64);
    if row0 >= row1 || col0 >= col1 {
        return;
    }
    let width = (col1 - col0) as usize;
    let mut cover = vec![0.0f32; width];
    let mut xs: Vec<(f32, i32)> = Vec::new();
    let colf = col0 as f32;
    let widthf = width as f32;
    for row in row0..row1 {
        for c in cover.iter_mut() {
            *c = 0.0;
        }
        let mut any = false;
        for k in 0..SUB {
            let sy = row as f32 + (k as f32 + 0.5) / SUB as f32;
            xs.clear();
            for e in edges {
                if sy >= e.y0 && sy < e.y1 {
                    let t = (sy - e.y0) / (e.y1 - e.y0);
                    xs.push((e.x0 + (e.x1 - e.x0) * t - colf, e.dir));
                }
            }
            if xs.len() < 2 {
                continue;
            }
            xs.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut wind = 0i32;
            for i in 0..xs.len() - 1 {
                wind += xs[i].1;
                let inside = match rule {
                    FillRule::NonZero => wind != 0,
                    FillRule::EvenOdd => (i + 1) % 2 == 1,
                };
                if !inside {
                    continue;
                }
                let xa = xs[i].0.max(0.0);
                let xb = xs[i + 1].0.min(widthf);
                if xb <= xa {
                    continue;
                }
                any = true;
                let ia = xa.floor() as usize;
                let ib = (xb.ceil() as usize).min(width);
                for (ix, slot) in cover.iter_mut().enumerate().take(ib).skip(ia) {
                    let lo = xa.max(ix as f32);
                    let hi = xb.min(ix as f32 + 1.0);
                    if hi > lo {
                        *slot += (hi - lo) / SUB as f32;
                    }
                }
            }
        }
        if !any {
            continue;
        }
        for (ix, c) in cover.iter().enumerate() {
            if *c <= 0.0 {
                continue;
            }
            let px = col0 as usize + ix;
            let idx = (row as usize * surf.width + px) * 4;
            blend(&mut surf.pixels[idx..idx + 4], paint, c.min(1.0));
        }
    }
}

pub(crate) fn blend(dst: &mut [u8], src: Color, coverage: f32) {
    let sa = (src.a as f32 / 255.0) * coverage;
    if sa <= 0.0 {
        return;
    }
    let da = dst[3] as f32 / 255.0;
    let oa = sa + da * (1.0 - sa);
    if oa <= 0.0 {
        return;
    }
    let mix = |s: u8, d: u8| -> u8 {
        let v = (s as f32 * sa + d as f32 * da * (1.0 - sa)) / oa;
        (v + 0.5).clamp(0.0, 255.0) as u8
    };
    dst[0] = mix(src.r, dst[0]);
    dst[1] = mix(src.g, dst[1]);
    dst[2] = mix(src.b, dst[2]);
    dst[3] = (oa * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
}
