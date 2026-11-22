use super::gfx::{put, put_mid};
use render::{Canvas, Color, Rect, Stroke, ellipsize, fit_size};

pub const AMBER: Color = Color::hex(0xf2a93b);
pub const GREEN: Color = Color::hex(0x4cd964);
pub const CYAN: Color = Color::hex(0x4fc3e8);
pub const BLUE: Color = Color::hex(0x3d7bd9);
pub const KEEP: usize = 6;

pub fn low(c: Color) -> Color {
    c.lerp(Color::BG, 0.55)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelAction {
    pub actor: String,
    pub action: String,
    pub target: String,
    pub value: Option<String>,
}

impl PanelAction {
    pub fn new(actor: &str, action: &str, target: &str, value: Option<&str>) -> PanelAction {
        PanelAction {
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            value: value.map(str::to_string),
        }
    }

    pub fn text(&self) -> String {
        match &self.value {
            Some(v) => format!("{} {} {}", self.action, self.target, v),
            None => format!("{} {}", self.action, self.target),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Outbox {
    queue: Vec<PanelAction>,
    recent: Vec<String>,
}

impl Outbox {
    pub fn push(&mut self, a: PanelAction) {
        self.recent.push(a.text());
        if self.recent.len() > KEEP {
            self.recent.remove(0);
        }
        self.queue.push(a);
    }

    pub fn take(&mut self) -> Vec<PanelAction> {
        std::mem::take(&mut self.queue)
    }

    pub fn pending(&self) -> &[PanelAction] {
        &self.queue
    }

    pub fn recent(&self) -> &[String] {
        &self.recent
    }
}

pub fn lamp_cell(c: &mut Canvas, r: Rect, text: &str, color: Color, glow: f32) {
    let lit = glow > 0.0;
    let g = glow.clamp(0.0, 1.0);
    let edge = if lit {
        color.lerp(Color::BG, 1.0 - g)
    } else {
        Color::LINE
    };
    let fill = if lit {
        Color::BG.lerp(color, 0.22 * g)
    } else {
        Color::PANEL
    };
    c.fill_rect(r.x, r.y, r.w, r.h, fill);
    c.stroke_rect(r.x, r.y, r.w, r.h, &Stroke::new(edge, 1.2));
    let ink = if lit {
        color.lerp(Color::BG, 1.0 - g)
    } else {
        Color::hex(0x707070)
    };
    let size = fit_size(text, 11.0, 10.0, r.w - 6.0);
    put_mid(c, r.x + r.w / 2.0, r.y + r.h / 2.0 + size * 0.3, text, size, ink);
}

pub fn heading(c: &mut Canvas, x: f32, y: f32, w: f32, text: &str) {
    put(c, x, y, &ellipsize(text, 10.0, w), 10.0, Color::DIM);
    c.line(x, y + 4.0, x + w, y + 4.0, &Stroke::new(Color::LINE, 1.0));
}

pub fn button(c: &mut Canvas, r: Rect, text: &str, enabled: bool, hot: bool) {
    let edge = if hot { Color::WHITE } else { Color::LINE };
    c.fill_rect(r.x, r.y, r.w, r.h, Color::PANEL2);
    c.stroke_rect(r.x, r.y, r.w, r.h, &Stroke::new(edge, 1.2));
    let ink = if enabled { Color::WHITE } else { Color::FAINT };
    let size = fit_size(text, 11.0, 10.0, r.w - 8.0);
    put_mid(c, r.x + r.w / 2.0, r.y + r.h / 2.0 + size * 0.3, text, size, ink);
}

pub fn arrow(c: &mut Canvas, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, head: bool) {
    c.line(x0, y0, x1, y1, &Stroke::new(color, 2.0));
    if !head {
        return;
    }
    let (dx, dy) = (x1 - x0, y1 - y0);
    let n = (dx * dx + dy * dy).sqrt().max(0.001);
    let (ux, uy) = (dx / n, dy / n);
    let (px, py) = (-uy, ux);
    c.fill_polygon(
        &[
            (x1, y1),
            (x1 - ux * 9.0 + px * 4.5, y1 - uy * 9.0 + py * 4.5),
            (x1 - ux * 9.0 - px * 4.5, y1 - uy * 9.0 - py * 4.5),
        ],
        color,
    );
}
