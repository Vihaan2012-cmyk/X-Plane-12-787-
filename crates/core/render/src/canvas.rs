use crate::color::Color;
use crate::font::{self, TextStyle};
use crate::math::{Affine, Rect, cos_f, sin_f};
use crate::raster::{FillRule, Surface, blend, build_edges, fill_edges};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub cap: Cap,
    pub join: Join,
    pub dash: Vec<f32>,
    pub dash_offset: f32,
}

impl Stroke {
    pub fn new(color: Color, width: f32) -> Stroke {
        Stroke {
            color,
            width,
            cap: Cap::Butt,
            join: Join::Round,
            dash: Vec::new(),
            dash_offset: 0.0,
        }
    }

    pub fn dashed(mut self, pattern: &[f32]) -> Stroke {
        self.dash = pattern.to_vec();
        self
    }

    pub fn with_cap(mut self, cap: Cap) -> Stroke {
        self.cap = cap;
        self
    }

    pub fn with_join(mut self, join: Join) -> Stroke {
        self.join = join;
        self
    }

    pub fn with_dash_offset(mut self, off: f32) -> Stroke {
        self.dash_offset = off;
        self
    }
}

type ClipBox = (i32, i32, i32, i32);

#[derive(Clone, Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    transform: Affine,
    transform_stack: Vec<Affine>,
    clip: ClipBox,
    clip_stack: Vec<ClipBox>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        Canvas {
            width,
            height,
            pixels: vec![0; width * height * 4],
            transform: Affine::IDENTITY,
            transform_stack: Vec::new(),
            clip: (0, 0, width as i32, height as i32),
            clip_stack: Vec::new(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn rgba(&self) -> &[u8] {
        &self.pixels
    }

    pub fn rgba_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn into_rgba(self) -> Vec<u8> {
        self.pixels
    }

    pub fn pixel(&self, x: usize, y: usize) -> Color {
        if x >= self.width || y >= self.height {
            return Color::TRANSPARENT;
        }
        let i = (y * self.width + x) * 4;
        Color::rgba(
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        )
    }

    pub fn clear(&mut self, color: Color) {
        for p in self.pixels.chunks_exact_mut(4) {
            p[0] = color.r;
            p[1] = color.g;
            p[2] = color.b;
            p[3] = color.a;
        }
    }

    pub fn save(&mut self) {
        self.transform_stack.push(self.transform);
        self.clip_stack.push(self.clip);
    }

    pub fn restore(&mut self) {
        if let Some(t) = self.transform_stack.pop() {
            self.transform = t;
        }
        if let Some(c) = self.clip_stack.pop() {
            self.clip = c;
        }
    }

    pub fn reset_transform(&mut self) {
        self.transform = Affine::IDENTITY;
    }

    pub fn transform(&self) -> Affine {
        self.transform
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform = self.transform.then(&Affine::translate(x, y));
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.transform = self.transform.then(&Affine::scale(x, y));
    }

    pub fn rotate(&mut self, rad: f32) {
        self.transform = self.transform.then(&Affine::rotate(rad));
    }

    pub fn clip_rect(&mut self, r: Rect) {
        let t = self.transform;
        let pts = [
            t.apply(r.x, r.y),
            t.apply(r.right(), r.y),
            t.apply(r.right(), r.bottom()),
            t.apply(r.x, r.bottom()),
        ];
        let mut x0 = f32::MAX;
        let mut y0 = f32::MAX;
        let mut x1 = f32::MIN;
        let mut y1 = f32::MIN;
        for (x, y) in pts {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        let nx0 = (x0.round() as i32).max(self.clip.0);
        let ny0 = (y0.round() as i32).max(self.clip.1);
        let nx1 = (x1.round() as i32).min(self.clip.2).max(nx0);
        let ny1 = (y1.round() as i32).min(self.clip.3).max(ny0);
        self.clip = (nx0, ny0, nx1, ny1);
    }

    pub fn reset_clip(&mut self) {
        self.clip = (0, 0, self.width as i32, self.height as i32);
    }

    pub fn clip_bounds(&self) -> Rect {
        Rect::new(
            self.clip.0 as f32,
            self.clip.1 as f32,
            (self.clip.2 - self.clip.0) as f32,
            (self.clip.3 - self.clip.1) as f32,
        )
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < self.clip.0 || y < self.clip.1 || x >= self.clip.2 || y >= self.clip.3 {
            return;
        }
        let i = (y as usize * self.width + x as usize) * 4;
        blend(&mut self.pixels[i..i + 4], color, 1.0);
    }

    fn device(&self, pts: &[(f32, f32)]) -> Vec<(f32, f32)> {
        pts.iter().map(|&(x, y)| self.transform.apply(x, y)).collect()
    }

    fn fill_device(&mut self, polys: &[Vec<(f32, f32)>], rule: FillRule, color: Color) {
        let edges = build_edges(polys);
        let clip = self.clip;
        let mut surf = Surface {
            width: self.width,
            pixels: &mut self.pixels,
        };
        fill_edges(&mut surf, &edges, rule, color, clip);
    }

    pub fn fill_polygon_rule(&mut self, pts: &[(f32, f32)], rule: FillRule, color: Color) {
        let d = self.device(pts);
        self.fill_device(&[d], rule, color);
    }

    pub fn fill_contours(&mut self, polys: &[Vec<(f32, f32)>], rule: FillRule, color: Color) {
        let dev: Vec<Vec<(f32, f32)>> = polys.iter().map(|p| self.device(p)).collect();
        self.fill_device(&dev, rule, color);
    }

    pub fn fill_polygon(&mut self, pts: &[(f32, f32)], color: Color) {
        self.fill_polygon_rule(pts, FillRule::NonZero, color);
    }

    pub fn stroke_polygon(&mut self, pts: &[(f32, f32)], stroke: &Stroke) {
        self.stroke_path(pts, true, stroke);
    }

    pub fn polyline(&mut self, pts: &[(f32, f32)], stroke: &Stroke) {
        self.stroke_path(pts, false, stroke);
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, stroke: &Stroke) {
        self.stroke_path(&[(x0, y0), (x1, y1)], false, stroke);
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.fill_polygon(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], color);
    }

    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32, stroke: &Stroke) {
        self.stroke_path(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], true, stroke);
    }

    fn rounded_rect_points(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<(f32, f32)> {
        let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
        if r <= 0.0 {
            return vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
        }
        let mut pts = Vec::new();
        let corners = [
            (x + w - r, y + r, 270.0f32),
            (x + w - r, y + h - r, 0.0),
            (x + r, y + h - r, 90.0),
            (x + r, y + r, 180.0),
        ];
        let steps = ((r * 1.5) as usize).clamp(4, 24);
        for (cx, cy, start) in corners {
            for i in 0..=steps {
                let a = (start + 90.0 * i as f32 / steps as f32).to_radians();
                pts.push((cx + r * cos_f(a), cy + r * sin_f(a)));
            }
        }
        pts
    }

    pub fn fill_rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
        let pts = Self::rounded_rect_points(x, y, w, h, r);
        self.fill_polygon(&pts, color);
    }

    pub fn stroke_rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, stroke: &Stroke) {
        let pts = Self::rounded_rect_points(x, y, w, h, r);
        self.stroke_path(&pts, true, stroke);
    }

    fn ellipse_points(&self, cx: f32, cy: f32, rx: f32, ry: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
        let s = self.transform.mean_scale().max(0.01);
        let rmax = rx.abs().max(ry.abs()) * s;
        let sweep = (a1 - a0).abs();
        let full = (rmax.sqrt() * 8.0).ceil().clamp(16.0, 720.0);
        let steps = ((full * sweep / std::f32::consts::TAU).ceil() as usize).max(4);
        (0..=steps)
            .map(|i| {
                let a = a0 + (a1 - a0) * i as f32 / steps as f32;
                (cx + rx * cos_f(a), cy + ry * sin_f(a))
            })
            .collect()
    }

    pub fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: Color) {
        let mut pts = self.ellipse_points(cx, cy, rx, ry, 0.0, std::f32::consts::TAU);
        pts.pop();
        self.fill_polygon(&pts, color);
    }

    pub fn stroke_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, stroke: &Stroke) {
        let mut pts = self.ellipse_points(cx, cy, rx, ry, 0.0, std::f32::consts::TAU);
        pts.pop();
        self.stroke_path(&pts, true, stroke);
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32, color: Color) {
        self.fill_ellipse(cx, cy, r, r, color);
    }

    pub fn stroke_circle(&mut self, cx: f32, cy: f32, r: f32, stroke: &Stroke) {
        self.stroke_ellipse(cx, cy, r, r, stroke);
    }

    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, start: f32, end: f32, stroke: &Stroke) {
        let pts = self.ellipse_points(cx, cy, r, r, start, end);
        self.stroke_path(&pts, false, stroke);
    }

    pub fn fill_pie(&mut self, cx: f32, cy: f32, r: f32, start: f32, end: f32, color: Color) {
        let mut pts = vec![(cx, cy)];
        pts.extend(self.ellipse_points(cx, cy, r, r, start, end));
        self.fill_polygon(&pts, color);
    }

    pub fn stroke_path(&mut self, pts: &[(f32, f32)], closed: bool, stroke: &Stroke) {
        if pts.is_empty() || stroke.width <= 0.0 || stroke.color.a == 0 {
            return;
        }
        let scale = self.transform.mean_scale();
        let dev = self.device(pts);
        let w = (stroke.width * scale).max(0.0);
        let dash: Vec<f32> = stroke.dash.iter().map(|d| d * scale).collect();
        let runs = if dash.len() >= 2 && dash.iter().all(|d| *d > 0.0) {
            dash_path(&dev, closed, &dash, stroke.dash_offset * scale)
        } else {
            let mut p = dev;
            if closed && p.len() > 2 {
                let first = p[0];
                p.push(first);
                vec![(p, true)]
            } else {
                vec![(p, false)]
            }
        };
        let mut polys: Vec<Vec<(f32, f32)>> = Vec::new();
        for (run, is_closed) in &runs {
            stroke_run(run, *is_closed, w, stroke.cap, stroke.join, &mut polys);
        }
        self.fill_device(&polys, FillRule::NonZero, stroke.color);
    }

    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, style: &TextStyle) {
        font::draw_text(self, x, y, text, style);
    }

    pub fn text(&mut self, x: f32, y: f32, text: &str, size: f32, color: Color) {
        font::draw_text(self, x, y, text, &TextStyle::new(size, color));
    }

    pub fn draw_canvas(&mut self, src: &Canvas, dx: i32, dy: i32) {
        for sy in 0..src.height {
            for sx in 0..src.width {
                let c = src.pixel(sx, sy);
                if c.a != 0 {
                    self.set_pixel(dx + sx as i32, dy + sy as i32, c);
                }
            }
        }
    }
}

type Run = (Vec<(f32, f32)>, bool);

fn dash_path(pts: &[(f32, f32)], closed: bool, dash: &[f32], offset: f32) -> Vec<Run> {
    let mut p = pts.to_vec();
    if closed && p.len() > 2 {
        p.push(p[0]);
    }
    let total: f32 = dash.iter().sum();
    let mut idx = 0usize;
    let mut remaining = dash[0];
    let mut off = offset.rem_euclid(total);
    while off > 0.0 {
        if off >= remaining {
            off -= remaining;
            idx = (idx + 1) % dash.len();
            remaining = dash[idx];
        } else {
            remaining -= off;
            off = 0.0;
        }
    }
    let mut out = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    if idx.is_multiple_of(2) && !p.is_empty() {
        cur.push(p[0]);
    }
    for seg in p.windows(2) {
        let (ax, ay) = seg[0];
        let (bx, by) = seg[1];
        let len = ((bx - ax) * (bx - ax) + (by - ay) * (by - ay)).sqrt();
        if len <= 0.0 {
            continue;
        }
        let mut pos = 0.0f32;
        while pos < len {
            let step = remaining.min(len - pos);
            pos += step;
            remaining -= step;
            let t = pos / len;
            let q = (ax + (bx - ax) * t, ay + (by - ay) * t);
            if idx.is_multiple_of(2) {
                cur.push(q);
            }
            if remaining <= 1e-6 {
                if idx.is_multiple_of(2) && cur.len() >= 2 {
                    out.push((std::mem::take(&mut cur), false));
                } else {
                    cur.clear();
                }
                idx = (idx + 1) % dash.len();
                remaining = dash[idx];
                if idx.is_multiple_of(2) {
                    cur.push(q);
                }
            }
        }
    }
    if cur.len() >= 2 {
        out.push((cur, false));
    }
    out
}

fn orient(mut poly: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let mut area = 0.0f32;
    for i in 0..poly.len() {
        let (ax, ay) = poly[i];
        let (bx, by) = poly[(i + 1) % poly.len()];
        area += ax * by - bx * ay;
    }
    if area < 0.0 {
        poly.reverse();
    }
    poly
}

fn disc(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    let steps = ((r.sqrt() * 6.0).ceil() as usize).clamp(8, 64);
    (0..steps)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / steps as f32;
            (cx + r * cos_f(a), cy + r * sin_f(a))
        })
        .collect()
}

fn stroke_run(
    pts: &[(f32, f32)],
    closed: bool,
    w: f32,
    cap: Cap,
    join: Join,
    out: &mut Vec<Vec<(f32, f32)>>,
) {
    let mut p: Vec<(f32, f32)> = Vec::with_capacity(pts.len());
    for &q in pts {
        if p.last()
            .is_none_or(|l| (l.0 - q.0).abs() > 1e-4 || (l.1 - q.1).abs() > 1e-4)
        {
            p.push(q);
        }
    }
    let h = w / 2.0;
    if p.len() == 1 {
        if cap == Cap::Round {
            out.push(orient(disc(p[0].0, p[0].1, h)));
        } else if cap == Cap::Square {
            let (x, y) = p[0];
            out.push(vec![
                (x - h, y - h),
                (x + h, y - h),
                (x + h, y + h),
                (x - h, y + h),
            ]);
        }
        return;
    }
    if p.len() < 2 {
        return;
    }
    let n = p.len();
    let mut dirs = Vec::with_capacity(n - 1);
    for i in 0..n - 1 {
        let dx = p[i + 1].0 - p[i].0;
        let dy = p[i + 1].1 - p[i].1;
        let l = (dx * dx + dy * dy).sqrt();
        dirs.push((dx / l, dy / l));
    }
    for i in 0..n - 1 {
        let (dx, dy) = dirs[i];
        let (nx, ny) = (-dy * h, dx * h);
        let (mut ax, mut ay) = p[i];
        let (mut bx, mut by) = p[i + 1];
        if !closed && cap == Cap::Square {
            if i == 0 {
                ax -= dx * h;
                ay -= dy * h;
            }
            if i == n - 2 {
                bx += dx * h;
                by += dy * h;
            }
        }
        out.push(orient(vec![
            (ax + nx, ay + ny),
            (bx + nx, by + ny),
            (bx - nx, by - ny),
            (ax - nx, ay - ny),
        ]));
    }
    let wide = h > 0.6;
    let join_range = if closed { 0..n - 1 } else { 1..n - 1 };
    for i in join_range {
        let prev = if i == 0 { dirs[n - 2] } else { dirs[i - 1] };
        let next = dirs[i];
        let cross = prev.0 * next.1 - prev.1 * next.0;
        let dot = prev.0 * next.0 + prev.1 * next.1;
        if cross.abs() < 1e-4 && dot > 0.0 {
            continue;
        }
        let (x, y) = p[i];
        match join {
            Join::Round => {
                if wide {
                    out.push(orient(disc(x, y, h)));
                }
            }
            Join::Bevel | Join::Miter => {
                let side = if cross > 0.0 { -1.0 } else { 1.0 };
                let a = (x + side * -prev.1 * h, y + side * prev.0 * h);
                let b = (x + side * -next.1 * h, y + side * next.0 * h);
                let mut poly = vec![(x, y), a];
                if join == Join::Miter {
                    let denom = 1.0 + dot;
                    if denom > 0.25 {
                        let mx = (-prev.1 - next.1) * side * h / denom;
                        let my = (prev.0 + next.0) * side * h / denom;
                        poly.push((x + mx, y + my));
                    }
                }
                poly.push(b);
                out.push(orient(poly));
            }
        }
    }
    if !closed && cap == Cap::Round && wide {
        out.push(orient(disc(p[0].0, p[0].1, h)));
        out.push(orient(disc(p[n - 1].0, p[n - 1].1, h)));
    }
}
