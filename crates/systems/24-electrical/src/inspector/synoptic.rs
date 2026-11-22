use super::gfx::{frame, put, put_mid};
use super::topo::{VH, VW};
use super::ui::{AMBER, CYAN, GREEN, arrow, low};
use crate::crew::synoptic::{BatteryLabel, DriveState, SynBattery, SynSource};
use crate::crew::{Colour, CrewInputs, ElementState, SynopticPage as SynData, build_synoptic};
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, Stroke, ellipsize};

const CX: [f32; 4] = [170.0, 350.0, 650.0, 830.0];
const BUS: [&str; 4] = ["BUS L1", "BUS L2", "BUS R1", "BUS R2"];
const GEN: [&str; 4] = ["GEN L1", "GEN L2", "GEN R1", "GEN R2"];
const MAIN: [&str; 4] = ["L1 MAIN", "L2 MAIN", "R1 MAIN", "R2 MAIN"];

pub struct SynopticPage {
    data: SynData,
}

fn colour(c: Colour) -> Color {
    match c {
        Colour::Green => GREEN,
        Colour::White => Color::WHITE,
        Colour::WhiteLow => low(Color::WHITE),
        Colour::Amber => AMBER,
        Colour::AmberLow => low(AMBER),
        Colour::Cyan => CYAN,
    }
}

fn state_text(s: ElementState) -> &'static str {
    match s {
        ElementState::On => "ON",
        ElementState::Available => "AVAIL",
        ElementState::Off => "OFF",
        ElementState::Failed => "FAIL",
        ElementState::Invalid => "INVALID",
    }
}

fn cross(c: &mut Canvas, r: Rect, col: Color) {
    let st = Stroke::new(col, 1.6);
    c.line(r.x + 4.0, r.y + 4.0, r.right() - 4.0, r.bottom() - 4.0, &st);
    c.line(r.right() - 4.0, r.y + 4.0, r.x + 4.0, r.bottom() - 4.0, &st);
}

fn source_box(c: &mut Canvas, r: Rect, name: &str, s: &SynSource) {
    let col = colour(s.colour);
    if s.state == ElementState::On {
        c.fill_rect(r.x, r.y, r.w, r.h, Color::BG.lerp(col, 0.22));
    }
    let width = if matches!(s.state, ElementState::On | ElementState::Failed) {
        2.4
    } else {
        1.4
    };
    c.stroke_rect(r.x, r.y, r.w, r.h, &Stroke::new(col, width));
    if s.state == ElementState::Failed {
        cross(c, r, col);
    }
    put_mid(c, r.x + r.w / 2.0, r.y + 22.0, name, 11.0, Color::WHITE);
    put_mid(
        c,
        r.x + r.w / 2.0,
        r.y + r.h - 10.0,
        state_text(s.state),
        11.0,
        col,
    );
}

fn battery_box(c: &mut Canvas, r: Rect, name: &str, b: &SynBattery) {
    c.fill_rect(r.x, r.y, r.w, r.h, Color::PANEL);
    frame(c, r.x, r.y, r.w, r.h, Color::LINE);
    put(c, r.x + 10.0, r.y + 20.0, name, 12.0, Color::WHITE);
    let v = b.volts.map_or(String::from("--"), |v| v.to_string());
    let a = b.amps.map_or(String::from("--"), |v| v.to_string());
    put(c, r.x + 10.0, r.y + 42.0, "VOLTS", 10.0, Color::DIM);
    put(c, r.x + 70.0, r.y + 42.0, &v, 12.0, Color::WHITE);
    put(c, r.x + 10.0, r.y + 62.0, "AMPS", 10.0, Color::DIM);
    put(c, r.x + 70.0, r.y + 62.0, &a, 12.0, Color::WHITE);
    let label = match b.label {
        Some(BatteryLabel::Chg) => "CHG",
        Some(BatteryLabel::Disch) => "DISCH",
        None => "",
    };
    put(c, r.x + 130.0, r.y + 62.0, label, 11.0, Color::WHITE);
}

impl SynopticPage {
    pub fn new(data: SynData) -> SynopticPage {
        SynopticPage { data }
    }

    pub fn demo() -> SynopticPage {
        SynopticPage::new(build_synoptic(&CrewInputs::default(), [true; 4], [true; 4]))
    }

    pub fn set_data(&mut self, data: SynData) {
        self.data = data;
    }

    pub fn data(&self) -> &SynData {
        &self.data
    }

    fn flow(&self, from: &str, to: &str) -> (bool, Color) {
        match self.data.lines.iter().find(|l| l.from == from && l.to == to) {
            Some(l) => (l.flowing, colour(l.colour)),
            None => (false, Color::WHITE),
        }
    }

    fn link(&self, c: &mut Canvas, from: &str, to: &str, x: f32, y0: f32, y1: f32) {
        let (flowing, col) = self.flow(from, to);
        if flowing {
            arrow(c, x, y0, x, y1, col, true);
        } else {
            c.line(x, y0, x, y1, &Stroke::new(col, 1.6));
        }
    }

    fn drive_symbol(&self, c: &mut Canvas, i: usize) {
        let g = &self.data.gens[i];
        let x = CX[i];
        let col = colour(g.drive_colour);
        let r = Rect::new(x - 40.0, 468.0, 80.0, 34.0);
        c.stroke_rect(r.x, r.y, r.w, r.h, &Stroke::new(col, 1.6));
        if g.drive == DriveState::Disconnected {
            cross(c, r, col);
        } else {
            put_mid(c, x, 490.0, "DRIVE", 10.0, col);
        }
        c.line(x, 468.0, x, 432.0, &Stroke::new(col, 1.6));
        if g.drive_text {
            put(c, x + 46.0, 490.0, "DRIVE", 11.0, colour(g.drive_colour));
        }
    }

    fn gen_symbol(&self, c: &mut Canvas, i: usize) {
        let g = &self.data.gens[i];
        let x = CX[i];
        let r = Rect::new(x - 55.0, 350.0, 110.0, 82.0);
        source_box(c, r, &format!("GEN {}", ["L1", "L2", "R1", "R2"][i]), &g.source);
        if g.starter_on {
            let s = Rect::new(x - 55.0, 436.0, 22.0, 22.0);
            c.fill_rect(s.x, s.y, s.w, s.h, Color::WHITE);
            arrow(
                c,
                s.x + 11.0,
                s.y + 3.0,
                s.x + 11.0,
                s.y + 19.0,
                Color::hex(0x1f9d3a),
                true,
            );
        }
        if g.starters_failed_text {
            put_mid(c, x, 520.0, "STARTERS", 11.0, AMBER);
        }
    }

    fn load_shed(&self, c: &mut Canvas) {
        let r = Rect::new(620.0, 540.0, 360.0, 120.0);
        c.stroke_rect(r.x, r.y, r.w, r.h, &Stroke::new(CYAN, 1.6));
        put(c, r.x + 10.0, r.y + 18.0, "LOAD SHED", 12.0, CYAN);
        if self.data.load_shed.is_empty() {
            put(c, r.x + 10.0, r.y + 40.0, "none", 11.0, Color::DIM);
        }
        for (i, t) in self.data.load_shed.iter().take(6).enumerate() {
            put(
                c,
                r.x + 10.0,
                r.y + 40.0 + 14.0 * i as f32,
                &ellipsize(t, 11.0, r.w - 20.0),
                11.0,
                CYAN,
            );
        }
        if self.data.load_shed.len() > 6 {
            let more = format!("+{} more", self.data.load_shed.len() - 6);
            put(c, r.x + r.w - 80.0, r.y + 18.0, &more, 10.0, CYAN);
        }
    }

    fn large_motor(&self, c: &mut Canvas) {
        let m = &self.data.large_motor;
        let r = Rect::new(400.0, 150.0, 200.0, 62.0);
        let on = m.aft_feed_flowing;
        c.stroke_rect(
            r.x,
            r.y,
            r.w,
            r.h,
            &Stroke::new(if on { GREEN } else { Color::WHITE }, 1.6),
        );
        put_mid(
            c,
            r.x + r.w / 2.0,
            r.y + 20.0,
            "LARGE MOTOR POWER",
            11.0,
            Color::WHITE,
        );
        let items = [
            ("L START", m.l_start, r.x + 40.0),
            ("APU START", m.apu_start, r.x + 100.0),
            ("R START", m.r_start, r.x + 160.0),
        ];
        for (t, lit, x) in items {
            put_mid(c, x, r.y + 52.0, t, 10.0, if lit { GREEN } else { Color::DIM });
        }
        let (flow, col) = self.flow("AFT EXT PWR", "LARGE MOTOR POWER");
        if flow {
            arrow(c, 500.0, 100.0, 500.0, 150.0, col, true);
        } else {
            c.line(500.0, 100.0, 500.0, 150.0, &Stroke::new(col, 1.6));
        }
        if m.l_start {
            arrow(c, 400.0, 181.0, 300.0, 181.0, GREEN, true);
            arrow(c, 300.0, 181.0, 300.0, 240.0, GREEN, true);
        }
        if m.r_start {
            arrow(c, 600.0, 181.0, 700.0, 181.0, GREEN, true);
            arrow(c, 700.0, 181.0, 700.0, 240.0, GREEN, true);
        }
    }
}

impl Page for SynopticPage {
    fn id(&self) -> &str {
        "electrical.synoptic"
    }

    fn title(&self) -> &str {
        "Electrical synoptic (ELEC)"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        c.save();
        c.translate(area.x, area.y);
        c.scale(s, s);
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        put(c, 14.0, 22.0, "ELEC", 14.0, Color::WHITE);
        let d = &self.data;
        let top = [
            (CX[0], "APU GEN L", &d.apu_gens[0], "APU GEN L", 0usize),
            (CX[1], "FWD EXT PWR L", &d.ext_fwd[0], "FWD EXT PWR L", 1),
            (CX[2], "APU GEN R", &d.apu_gens[1], "APU GEN R", 2),
            (CX[3], "FWD EXT PWR R", &d.ext_fwd[1], "FWD EXT PWR R", 3),
        ];
        for (x, name, src, key, bus) in top {
            source_box(c, Rect::new(x - 70.0, 40.0, 140.0, 60.0), name, src);
            self.link(c, key, BUS[bus], x, 100.0, 258.0);
        }
        source_box(c, Rect::new(435.0, 40.0, 130.0, 60.0), "AFT EXT PWR", &d.ext_aft);
        self.large_motor(c);
        c.stroke_rounded_rect(50.0, 238.0, 900.0, 76.0, 20.0, &Stroke::new(Color::DIM, 1.4));
        put_mid(c, 500.0, 252.0, "AC BUSES", 10.0, Color::DIM);
        for i in 0..4 {
            let b = &d.buses[i];
            let col = colour(b.colour);
            let r = Rect::new(CX[i] - 70.0, 258.0, 140.0, 44.0);
            if b.powered == Some(true) {
                c.fill_rect(r.x, r.y, r.w, r.h, Color::BG.lerp(col, 0.22));
            }
            c.stroke_rect(r.x, r.y, r.w, r.h, &Stroke::new(col, 2.0));
            put_mid(c, CX[i], 285.0, MAIN[i], 12.0, col);
        }
        for i in 0..4 {
            self.link(c, GEN[i], BUS[i], CX[i], 350.0, 302.0);
            self.gen_symbol(c, i);
            self.drive_symbol(c, i);
        }
        put_mid(c, 260.0, 530.0, "L1 - GEN - L2", 10.0, Color::DIM);
        put_mid(c, 740.0, 530.0, "R1 - GEN - R2", 10.0, Color::DIM);
        battery_box(
            c,
            Rect::new(50.0, 560.0, 240.0, 80.0),
            "MAIN BAT",
            &d.main_battery,
        );
        battery_box(c, Rect::new(320.0, 560.0, 240.0, 80.0), "APU BAT", &d.apu_battery);
        self.load_shed(c);
        c.restore();
    }

    fn click(&mut self, _x: f32, _y: f32) -> bool {
        false
    }

    fn detail_title(&self) -> String {
        String::from("ELEC synoptic")
    }

    fn detail(&self) -> Vec<DetailSection> {
        let d = &self.data;
        let mut sources = Vec::new();
        for (i, g) in d.gens.iter().enumerate() {
            let cell = if g.source.state == ElementState::Failed {
                Cell::fault(state_text(g.source.state))
            } else {
                Cell::new(state_text(g.source.state))
            };
            sources.push((GEN[i].to_string(), cell));
        }
        for (n, s) in [
            ("APU GEN L", &d.apu_gens[0]),
            ("APU GEN R", &d.apu_gens[1]),
            ("FWD EXT PWR L", &d.ext_fwd[0]),
            ("FWD EXT PWR R", &d.ext_fwd[1]),
            ("AFT EXT PWR", &d.ext_aft),
        ] {
            let cell = if s.state == ElementState::Failed {
                Cell::fault(state_text(s.state))
            } else {
                Cell::new(state_text(s.state))
            };
            sources.push((n.to_string(), cell));
        }
        let buses = d
            .buses
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let t = match b.powered {
                    Some(true) => "powered",
                    Some(false) => "not powered",
                    None => "invalid",
                };
                let cell = if b.powered == Some(false) {
                    Cell::fault(t)
                } else {
                    Cell::new(t)
                };
                (MAIN[i].to_string(), cell)
            })
            .collect();
        vec![
            DetailSection {
                title: String::from("Sources"),
                rows: sources,
            },
            DetailSection {
                title: String::from("AC buses"),
                rows: buses,
            },
        ]
    }

    fn table(&self) -> Table {
        Table {
            title: String::from("Flow lines"),
            columns: vec![String::from("from"), String::from("to"), String::from("flow")],
            rows: self
                .data
                .lines
                .iter()
                .map(|l| {
                    vec![
                        Cell::new(l.from),
                        Cell::new(l.to),
                        Cell::new(if l.flowing { "flowing" } else { "none" }),
                    ]
                })
                .collect(),
        }
    }
}
