use super::data::{InspectorData, ProtectionRow};
use super::gfx::{put, put_end};
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, Stroke, ellipsize};
use std::cell::Cell as StdCell;

const VW: f32 = 960.0;
const VH: f32 = 640.0;
const COLS: usize = 3;
const COL_W: f32 = 300.0;
const ROW_H: f32 = 15.0;
const HEAD_H: f32 = 22.0;
const PAD: f32 = 16.0;

pub struct ProtectionPage {
    data: InspectorData,
    selected: usize,
    scale: StdCell<f32>,
    hits: std::cell::RefCell<Vec<(Rect, usize)>>,
}

impl ProtectionPage {
    pub fn new(data: InspectorData) -> ProtectionPage {
        ProtectionPage {
            data,
            selected: 0,
            scale: StdCell::new(1.0),
            hits: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn set_data(&mut self, data: InspectorData) {
        self.data = data;
        if self.selected >= self.data.protections.len() {
            self.selected = 0;
        }
    }

    fn groups(&self) -> Vec<(String, Vec<(usize, &ProtectionRow)>)> {
        let mut out: Vec<(String, Vec<(usize, &ProtectionRow)>)> = Vec::new();
        for (i, r) in self.data.protections.iter().enumerate() {
            match out.iter_mut().find(|(g, _)| *g == r.group) {
                Some((_, v)) => v.push((i, r)),
                None => out.push((r.group.clone(), vec![(i, r)])),
            }
        }
        out
    }

    fn row(&self) -> Option<&ProtectionRow> {
        self.data.protections.get(self.selected)
    }
}

impl Page for ProtectionPage {
    fn id(&self) -> &str {
        "electrical.protection"
    }

    fn title(&self) -> &str {
        "Electrical protection"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        self.scale.set(s);
        c.save();
        c.translate(area.x, area.y);
        c.scale(s, s);
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        let tripped = self.data.protections.iter().filter(|r| r.tripped).count();
        put(c, PAD, 24.0, "PROTECTION", 13.0, Color::WHITE);
        put_end(
            c,
            VW - PAD,
            24.0,
            &format!("{tripped} tripped of {}", self.data.protections.len()),
            11.0,
            if tripped > 0 { Color::BAD } else { Color::DIM },
        );
        let mut col_y = [44.0_f32; COLS];
        let mut hits = Vec::new();
        for (g, rows) in self.groups() {
            let col = (0..COLS)
                .min_by(|a, b| {
                    col_y[*a]
                        .partial_cmp(&col_y[*b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(0);
            let x = PAD + col as f32 * (COL_W + PAD);
            let mut y = col_y[col];
            let h = HEAD_H + rows.len() as f32 * ROW_H + 8.0;
            c.fill_rect(x, y, COL_W, h, Color::PANEL);
            c.stroke_rect(x, y, COL_W, h, &Stroke::new(Color::LINE, 1.0));
            put(c, x + 8.0, y + 15.0, &g, 11.0, Color::WHITE);
            y += HEAD_H;
            for (i, r) in rows {
                let sel = i == self.selected;
                if sel {
                    c.fill_rect(x + 2.0, y - 1.0, COL_W - 4.0, ROW_H, Color::PANEL2);
                }
                let tone = if r.tripped { Color::BAD } else { Color::DIM };
                put(
                    c,
                    x + 8.0,
                    y + 10.0,
                    &ellipsize(&r.name, 10.0, 120.0),
                    10.0,
                    if sel { Color::WHITE } else { Color::DIM },
                );
                put_end(
                    c,
                    x + COL_W - 8.0,
                    y + 10.0,
                    &ellipsize(&r.state, 10.0, COL_W - 150.0),
                    10.0,
                    tone,
                );
                hits.push((Rect::new(x, y - 1.0, COL_W, ROW_H), i));
                y += ROW_H;
            }
            col_y[col] = y + 8.0 + PAD * 0.5;
        }
        *self.hits.borrow_mut() = hits;
        c.restore();
    }

    fn click(&mut self, x: f32, y: f32) -> bool {
        let s = self.scale.get().max(1.0e-3);
        let (px, py) = (x / s, y / s);
        let hit = self
            .hits
            .borrow()
            .iter()
            .find(|(r, _)| px >= r.x && px <= r.x + r.w && py >= r.y && py <= r.y + r.h)
            .map(|(_, i)| *i);
        match hit {
            Some(i) => {
                self.selected = i;
                true
            }
            None => false,
        }
    }

    fn detail_title(&self) -> String {
        self.row()
            .map(|r| r.name.clone())
            .unwrap_or_else(|| "Protection".to_string())
    }

    fn detail(&self) -> Vec<DetailSection> {
        let Some(r) = self.row() else {
            return Vec::new();
        };
        let state = if r.tripped {
            Cell::fault(r.state.clone())
        } else {
            Cell::new(r.state.clone())
        };
        vec![DetailSection {
            title: r.group.clone(),
            rows: vec![
                ("state".to_string(), state),
                ("live".to_string(), Cell::new(r.detail.clone())),
            ],
        }]
    }

    fn table(&self) -> Table {
        Table {
            title: "All protections".to_string(),
            columns: vec!["group".to_string(), "device".to_string(), "state".to_string()],
            rows: self
                .data
                .protections
                .iter()
                .map(|r| {
                    vec![
                        Cell::new(r.group.clone()),
                        Cell::new(r.name.clone()),
                        if r.tripped {
                            Cell::fault(r.state.clone())
                        } else {
                            Cell::new(r.state.clone())
                        },
                    ]
                })
                .collect(),
        }
    }
}
