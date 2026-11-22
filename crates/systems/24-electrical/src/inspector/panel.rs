use super::data::InspectorData;
use super::gfx::{amps, frame, put, put_end, symbol, tone, volts};
use super::topo::{CONTACTORS, PANELS, VH, VW};
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, ellipsize, measure_text};
use std::cell::Cell as StdCell;

pub struct PanelPage {
    data: InspectorData,
    selected: usize,
    scale: StdCell<f32>,
}

const COLS: usize = 5;
const TW: f32 = 188.0;
const TH: f32 = 318.0;
const GAP: f32 = 12.0;

impl PanelPage {
    pub fn new(data: InspectorData) -> PanelPage {
        PanelPage {
            data,
            selected: 3,
            scale: StdCell::new(1.0),
        }
    }

    pub fn set_data(&mut self, data: InspectorData) {
        self.data = data;
    }

    pub fn selected_panel(&self) -> &'static str {
        PANELS[self.selected].id
    }

    pub fn select(&mut self, id: &str) -> bool {
        match PANELS.iter().position(|p| p.id == id) {
            Some(i) => {
                self.selected = i;
                true
            }
            None => false,
        }
    }

    fn origin(i: usize) -> (f32, f32) {
        let col = (i % COLS) as f32;
        let row = (i / COLS) as f32;
        (10.0 + col * (TW + GAP), 12.0 + row * (TH + GAP))
    }
}

impl Page for PanelPage {
    fn id(&self) -> &str {
        "electrical.panels"
    }

    fn title(&self) -> &str {
        "Electrical panels"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        self.scale.set(s);
        c.save();
        c.translate(area.x, area.y);
        c.scale(s, s);
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        for (i, p) in PANELS.iter().enumerate() {
            let (x, y) = Self::origin(i);
            let sel = i == self.selected;
            c.fill_rect(x, y, TW, TH, Color::PANEL);
            frame(c, x, y, TW, TH, if sel { Color::WHITE } else { Color::LINE });
            put(c, x + 8.0, y + 16.0, p.id, 12.0, Color::WHITE);
            let idw = measure_text(p.id, 12.0).0;
            let pl = ellipsize(p.place, 10.0, TW - 24.0 - idw);
            put_end(c, x + TW - 8.0, y + 16.0, &pl, 10.0, Color::DIM);
            let mut ty = y + 32.0;
            for t in p.text.iter() {
                put(c, x + 8.0, ty, &ellipsize(t, 10.0, TW - 16.0), 10.0, Color::DIM);
                ty += 12.0;
            }
            ty += 6.0;
            for b in p.buses.iter() {
                let bv = self.data.bus(b);
                let live = bv.is_some_and(|v| v.live);
                put(c, x + 8.0, ty, b, 10.0, tone(live, bv.is_some_and(|v| v.fault)));
                put_end(
                    c,
                    x + TW - 8.0,
                    ty,
                    &volts(bv.map_or(0.0, |v| v.voltage_v)),
                    10.0,
                    tone(live, false),
                );
                ty += 14.0;
            }
            for u in p.units.iter() {
                let live = self.data.converter(u).is_some_and(|v| v.live)
                    || self.data.source(u).is_some_and(|v| v.online)
                    || self.data.battery(u).is_some_and(|v| v.voltage_v > 1.0);
                put(c, x + 8.0, ty, u, 10.0, tone(live, false));
                ty += 12.0;
            }
            ty += 4.0;
            for k in CONTACTORS.iter().filter(|k| k.panel == p.id) {
                let cv = self.data.contactor(k.name);
                let closed = cv.is_some_and(|v| v.closed);
                let fault = cv.is_some_and(|v| v.fault);
                symbol(c, x + 14.0, ty - 3.0, closed, closed, fault);
                put(c, x + 24.0, ty, k.name, 10.0, tone(closed, fault));
                put_end(
                    c,
                    x + TW - 8.0,
                    ty,
                    &amps(cv.map_or(0.0, |v| v.current_a)),
                    10.0,
                    if fault { Color::BAD } else { Color::DIM },
                );
                ty += 13.0;
            }
        }
        c.restore();
    }

    fn click(&mut self, x: f32, y: f32) -> bool {
        let s = self.scale.get().max(0.01);
        let (vx, vy) = (x / s, y / s);
        for i in 0..PANELS.len() {
            let (px, py) = Self::origin(i);
            if vx >= px && vx <= px + TW && vy >= py && vy <= py + TH {
                let changed = self.selected != i;
                self.selected = i;
                return changed;
            }
        }
        false
    }

    fn detail_title(&self) -> String {
        format!("Panel {}", PANELS[self.selected].id)
    }

    fn detail(&self) -> Vec<DetailSection> {
        let p = &PANELS[self.selected];
        let mut out = vec![DetailSection {
            title: "Location".to_string(),
            rows: vec![("bay".to_string(), Cell::new(p.place))],
        }];
        out.push(DetailSection {
            title: "Contents".to_string(),
            rows: p.text.iter().map(|t| (String::new(), Cell::new(*t))).collect(),
        });
        out.push(DetailSection {
            title: "Buses".to_string(),
            rows: p
                .buses
                .iter()
                .map(|b| {
                    let v = self.data.bus(b);
                    let t = format!(
                        "{} {}",
                        volts(v.map_or(0.0, |x| x.voltage_v)),
                        if v.is_some_and(|x| x.live) { "live" } else { "dead" }
                    );
                    (b.to_string(), Cell::new(t))
                })
                .collect(),
        });
        out
    }

    fn table(&self) -> Table {
        let p = &PANELS[self.selected];
        Table {
            title: format!("{} contactors", p.id),
            columns: ["device", "from", "to", "state", "A"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            rows: CONTACTORS
                .iter()
                .filter(|k| k.panel == p.id)
                .map(|k| {
                    let cv = self.data.contactor(k.name);
                    let st = if cv.is_some_and(|c| c.fault) {
                        Cell::fault("fault")
                    } else if cv.is_some_and(|c| c.closed) {
                        Cell::new("closed")
                    } else {
                        Cell::new("open")
                    };
                    vec![
                        Cell::new(k.name),
                        Cell::new(k.a),
                        Cell::new(k.b),
                        st,
                        Cell::new(amps(cv.map_or(0.0, |c| c.current_a))),
                    ]
                })
                .collect(),
        }
    }
}
