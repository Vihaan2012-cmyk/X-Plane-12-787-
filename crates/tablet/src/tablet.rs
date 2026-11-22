use crate::nav::NAV;
use crate::page::{DetailSection, Page, Table};
use render::{Anchor, Canvas, Color, Rect, Stroke, TextStyle};

const TEXT: f32 = 12.0;
const COL_GAP: f32 = 24.0;
const NAV_ROW: f32 = 23.0;
const NAV_HEAD: f32 = 22.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub width: f32,
    pub height: f32,
    pub top_h: f32,
    pub nav_w: f32,
    pub side_w: f32,
    pub bottom_h: f32,
}

impl Layout {
    pub const DEFAULT: Layout = Layout {
        width: 1600.0,
        height: 960.0,
        top_h: 38.0,
        nav_w: 170.0,
        side_w: 430.0,
        bottom_h: 250.0,
    };

    pub fn top(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.top_h)
    }

    pub fn nav(&self) -> Rect {
        Rect::new(0.0, self.top_h, self.nav_w, self.height - self.top_h)
    }

    pub fn diagram(&self) -> Rect {
        Rect::new(
            self.nav_w,
            self.top_h,
            self.width - self.nav_w - self.side_w,
            self.height - self.top_h - self.bottom_h,
        )
    }

    pub fn side(&self) -> Rect {
        Rect::new(
            self.width - self.side_w,
            self.top_h,
            self.side_w,
            self.height - self.top_h,
        )
    }

    pub fn bottom(&self) -> Rect {
        Rect::new(
            self.nav_w,
            self.height - self.bottom_h,
            self.width - self.nav_w - self.side_w,
            self.bottom_h,
        )
    }
}

pub struct Tablet {
    layout: Layout,
    pages: Vec<Box<dyn Page>>,
    selected: String,
    sim_time: f64,
    phase: String,
    view: String,
}

impl Tablet {
    pub fn new() -> Tablet {
        Tablet {
            layout: Layout::DEFAULT,
            pages: Vec::new(),
            selected: String::from("electrical"),
            sim_time: 0.0,
            phase: String::from("ground"),
            view: String::from("sheet 1"),
        }
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.layout.width = width.max(900.0);
        self.layout.height = height.max(560.0);
    }

    pub fn register(&mut self, page: Box<dyn Page>) {
        self.pages.retain(|p| p.id() != page.id());
        self.pages.push(page);
    }

    pub fn page_ids(&self) -> Vec<String> {
        self.pages.iter().map(|p| p.id().to_string()).collect()
    }

    pub fn select(&mut self, id: &str) {
        self.selected = id.to_string();
    }

    pub fn selected(&self) -> &str {
        &self.selected
    }

    pub fn set_sim_time(&mut self, seconds: f64) {
        self.sim_time = seconds;
    }

    pub fn set_phase(&mut self, phase: &str) {
        self.phase = phase.to_string();
    }

    pub fn set_view(&mut self, view: &str) {
        self.view = view.to_string();
    }

    fn active(&self) -> Option<&dyn Page> {
        self.pages
            .iter()
            .find(|p| p.id() == self.selected)
            .map(|p| p.as_ref())
    }

    fn nav_rows() -> Vec<(f32, Option<usize>, usize)> {
        let mut rows = Vec::new();
        let mut y = 10.0;
        for (gi, g) in NAV.iter().enumerate() {
            if gi > 0 {
                y += 8.0;
            }
            rows.push((y, None, gi));
            y += NAV_HEAD;
            for (ii, _) in g.items.iter().enumerate() {
                rows.push((y, Some(ii), gi));
                y += NAV_ROW;
            }
        }
        rows
    }

    pub fn click(&mut self, x: f32, y: f32) -> bool {
        let nav = self.layout.nav();
        if nav.contains(x, y) {
            let ly = y - nav.y;
            for (ry, item, gi) in Self::nav_rows() {
                if let Some(ii) = item
                    && ly >= ry
                    && ly < ry + NAV_ROW
                {
                    self.selected = NAV[gi].items[ii].id.to_string();
                    return true;
                }
            }
            return false;
        }
        let d = self.layout.diagram();
        if d.contains(x, y) {
            let id = self.selected.clone();
            if let Some(p) = self.pages.iter_mut().find(|p| p.id() == id) {
                return p.click(x - d.x, y - d.y);
            }
        }
        false
    }

    pub fn render(&self) -> Canvas {
        let mut c = Canvas::new(self.layout.width as usize, self.layout.height as usize);
        self.draw(&mut c);
        c
    }

    pub fn draw(&self, c: &mut Canvas) {
        c.clear(Color::BG);
        self.draw_top(c);
        self.draw_nav(c);
        self.draw_diagram(c);
        self.draw_side(c);
        self.draw_bottom(c);
    }

    fn label(c: &mut Canvas, x: f32, y: f32, s: &str, color: Color) {
        c.draw_text(x, y, s, &TextStyle::new(TEXT, color));
    }

    fn draw_top(&self, c: &mut Canvas) {
        let r = self.layout.top();
        c.fill_rect(r.x, r.y, r.w, r.h, Color::PANEL2);
        c.line(0.0, r.h - 0.5, r.w, r.h - 0.5, &Stroke::new(Color::LINE, 1.0));
        let title = self
            .active()
            .map(|p| p.title().to_string())
            .unwrap_or_else(|| String::from("-"));
        let base = 24.0;
        let t = format!("{:.3} s", self.sim_time);
        let items: [(&str, Color, f32); 7] = [
            ("SYSTEMS", Color::WHITE, 28.0),
            (title.as_str(), Color::WHITE, 28.0),
            ("sim time", Color::DIM, 10.0),
            (t.as_str(), Color::WHITE, 28.0),
            ("view", Color::DIM, 10.0),
            (self.view.as_str(), Color::WHITE, 28.0),
            ("phase", Color::DIM, 10.0),
        ];
        let mut x = 16.0;
        for (s, col, gap) in items {
            Self::label(c, x, base, s, col);
            x += render::measure_text(s, TEXT).0 + gap;
        }
        Self::label(c, x, base, &self.phase, Color::WHITE);
    }

    fn draw_nav(&self, c: &mut Canvas) {
        let r = self.layout.nav();
        c.fill_rect(r.x, r.y, r.w, r.h, Color::PANEL);
        c.line(
            r.right() - 0.5,
            r.y,
            r.right() - 0.5,
            r.bottom(),
            &Stroke::new(Color::LINE, 1.0),
        );
        for (ry, item, gi) in Self::nav_rows() {
            let y = r.y + ry;
            match item {
                None => Self::label(c, 16.0, y + 14.0, NAV[gi].name, Color::FAINT),
                Some(ii) => {
                    let it = NAV[gi].items[ii];
                    let on = it.id == self.selected;
                    if on {
                        c.fill_rect(0.0, y, r.w - 1.0, NAV_ROW, Color::rgb(0x1c, 0x1c, 0x1c));
                    }
                    let has = self.pages.iter().any(|p| p.id() == it.id);
                    let col = if on {
                        Color::WHITE
                    } else if has {
                        Color::rgb(0xc0, 0xc0, 0xc0)
                    } else {
                        Color::DIM
                    };
                    Self::label(c, 16.0, y + 16.0, it.label, col);
                }
            }
        }
    }

    fn draw_diagram(&self, c: &mut Canvas) {
        let d = self.layout.diagram();
        c.save();
        c.clip_rect(d);
        match self.active() {
            Some(p) => p.draw(c, d),
            None => {
                let msg = format!("no page registered for '{}'", self.selected);
                c.draw_text(
                    d.x + d.w / 2.0,
                    d.y + d.h / 2.0,
                    &msg,
                    &TextStyle::new(TEXT, Color::DIM).anchor(Anchor::Middle),
                );
            }
        }
        c.restore();
        c.line(
            d.x,
            d.bottom() - 0.5,
            d.right(),
            d.bottom() - 0.5,
            &Stroke::new(Color::LINE, 1.0),
        );
    }

    fn draw_side(&self, c: &mut Canvas) {
        let r = self.layout.side();
        c.fill_rect(r.x, r.y, r.w, r.h, Color::PANEL);
        c.line(
            r.x + 0.5,
            r.y,
            r.x + 0.5,
            r.bottom(),
            &Stroke::new(Color::LINE, 1.0),
        );
        c.save();
        c.clip_rect(r);
        let (title, sections): (String, Vec<DetailSection>) = match self.active() {
            Some(p) => (p.detail_title(), p.detail()),
            None => (String::from("-"), Vec::new()),
        };
        c.fill_rect(r.x + 1.0, r.y, r.w - 1.0, 30.0, Color::PANEL2);
        Self::label(c, r.x + 16.0, r.y + 20.0, &title, Color::WHITE);
        c.line(
            r.x,
            r.y + 30.0,
            r.right(),
            r.y + 30.0,
            &Stroke::new(Color::LINE, 1.0),
        );
        let mut y = r.y + 54.0;
        let inner = r.w - 32.0;
        let key_w = sections
            .iter()
            .flat_map(|s| s.rows.iter())
            .map(|(k, _)| render::measure_text(k, TEXT).0)
            .fold(0.0f32, f32::max)
            .min(inner * 0.45)
            + 16.0;
        for s in &sections {
            Self::label(
                c,
                r.x + 16.0,
                y,
                &render::ellipsize(&s.title, TEXT, inner),
                Color::WHITE,
            );
            y += 20.0;
            for (k, v) in &s.rows {
                Self::label(
                    c,
                    r.x + 16.0,
                    y,
                    &render::ellipsize(k, TEXT, key_w - 16.0),
                    Color::DIM,
                );
                let col = if v.fault { Color::BAD } else { Color::WHITE };
                let room = inner - key_w;
                Self::label(
                    c,
                    r.x + 16.0 + key_w,
                    y,
                    &render::ellipsize(&v.text, TEXT, room),
                    col,
                );
                y += 18.0;
            }
            y += 14.0;
        }
        c.restore();
    }

    fn column_widths(table: &Table, avail: f32) -> Vec<f32> {
        let n = table.columns.len().max(1);
        let mut w: Vec<f32> = table
            .columns
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let cells = table
                    .rows
                    .iter()
                    .filter_map(|r| r.get(i))
                    .map(|c| render::measure_text(&c.text, TEXT).0)
                    .fold(0.0f32, f32::max);
                cells.max(render::measure_text(h, TEXT).0) + COL_GAP
            })
            .collect();
        let total: f32 = w.iter().sum();
        if total > avail {
            let floor = 4.0 * COL_GAP;
            let mut over = total - avail;
            while over > 0.5 {
                let (mi, mx) = w
                    .iter()
                    .enumerate()
                    .fold((0, 0.0f32), |a, (i, &v)| if v > a.1 { (i, v) } else { a });
                if mx <= floor {
                    break;
                }
                let cut = over.min(mx - floor).min(4.0);
                w[mi] -= cut;
                over -= cut;
            }
        } else {
            let extra = (avail - total) / n as f32;
            for v in &mut w {
                *v += extra;
            }
        }
        w
    }

    fn draw_bottom(&self, c: &mut Canvas) {
        let r = self.layout.bottom();
        let table: Table = self.active().map(|p| p.table()).unwrap_or_default();
        c.save();
        c.clip_rect(r);
        c.fill_rect(r.x, r.y, r.w, 32.0, Color::PANEL2);
        Self::label(c, r.x + 16.0, r.y + 21.0, &table.title, Color::WHITE);
        c.line(
            r.x,
            r.y + 32.0,
            r.right(),
            r.y + 32.0,
            &Stroke::new(Color::LINE, 1.0),
        );
        let widths = Self::column_widths(&table, r.w - 32.0);
        let mut xs = Vec::new();
        let mut cx = r.x + 16.0;
        for w in &widths {
            xs.push(cx);
            cx += w;
        }
        let hy = r.y + 52.0;
        for (i, h) in table.columns.iter().enumerate() {
            let room = widths[i] - COL_GAP;
            Self::label(c, xs[i], hy, &render::ellipsize(h, TEXT, room), Color::DIM);
        }
        c.line(
            r.x + 16.0,
            hy + 6.0,
            r.right() - 16.0,
            hy + 6.0,
            &Stroke::new(Color::LINE, 1.0),
        );
        let mut y = hy + 24.0;
        for row in &table.rows {
            if y + 4.0 > r.bottom() {
                break;
            }
            for (i, cell) in row.iter().enumerate().take(widths.len()) {
                let col = if cell.fault { Color::BAD } else { Color::WHITE };
                let room = widths[i] - COL_GAP;
                Self::label(c, xs[i], y, &render::ellipsize(&cell.text, TEXT, room), col);
            }
            y += 21.0;
        }
        c.restore();
    }
}

impl Default for Tablet {
    fn default() -> Self {
        Self::new()
    }
}
