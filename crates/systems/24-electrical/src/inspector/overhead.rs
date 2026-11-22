use super::gfx::{frame, put, put_mid};
use super::topo::{VH, VW};
use super::ui::{AMBER, CYAN, GREEN, Outbox, PanelAction, button, heading, lamp_cell};
use crate::crew::{ApuSelector, CrewInputs, Lamp, LampColor, OverheadPanel, PanelLamps};
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, Stroke, ellipsize};
use std::cell::Cell as StdCell;

const EXT: &str = "v_ll = 115.0, hz = 400.0, kva = 90.0";
const SW: f32 = 80.0;
const SH: f32 = 82.0;
const CART: [&str; 3] = ["EXT_PWR_FWD_L", "EXT_PWR_FWD_R", "EXT_PWR_AFT"];
const GEN_X: [f32; 4] = [36.0, 122.0, 296.0, 382.0];
const GEN_T: [&str; 4] = ["GEN_CTRL_L1", "GEN_CTRL_L2", "GEN_CTRL_R1", "GEN_CTRL_R2"];
const DRV_T: [&str; 4] = ["DRIVE_DISC_L1", "DRIVE_DISC_L2", "DRIVE_DISC_R1", "DRIVE_DISC_R2"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hit {
    Battery,
    Ife,
    Cabin,
    Selector(ApuSelector),
    ApuGen(usize),
    ExtFwd(usize),
    ExtAft,
    Cart(usize),
    Gen(usize),
    Drive(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverheadState {
    pub battery_on: bool,
    pub ife_on: bool,
    pub cabin_on: bool,
    pub selector: ApuSelector,
    pub apu_gen_on: [bool; 2],
    pub gen_ctrl_on: [bool; 4],
    pub plugged: [bool; 3],
    pub apu_fault: bool,
    pub lamps: PanelLamps,
}

impl OverheadState {
    pub fn from_panel(panel: &OverheadPanel, lamps: &PanelLamps) -> OverheadState {
        OverheadState {
            battery_on: panel.battery.is_on(),
            ife_on: panel.ife_seats.is_on(),
            cabin_on: panel.cabin_util.is_on(),
            selector: panel.apu_selector,
            apu_gen_on: [panel.apu_gen[0].is_on(), panel.apu_gen[1].is_on()],
            gen_ctrl_on: [
                panel.gen_ctrl[0].is_on(),
                panel.gen_ctrl[1].is_on(),
                panel.gen_ctrl[2].is_on(),
                panel.gen_ctrl[3].is_on(),
            ],
            plugged: [false; 3],
            apu_fault: false,
            lamps: *lamps,
        }
    }

    pub fn demo() -> OverheadState {
        let mut panel = OverheadPanel::new();
        panel.set_normal();
        let (_, lamps) = panel.update(&CrewInputs::default());
        OverheadState::from_panel(&panel, &lamps)
    }
}

pub struct OverheadPage {
    state: OverheadState,
    guard: [bool; 4],
    out: Outbox,
    scale: StdCell<f32>,
}

fn sw(x: f32, y: f32) -> Rect {
    Rect::new(x, y, SW, SH)
}

fn hits() -> Vec<(Rect, Hit)> {
    let mut v = vec![
        (sw(36.0, 74.0), Hit::Ife),
        (sw(128.0, 74.0), Hit::Cabin),
        (sw(236.0, 74.0), Hit::Battery),
        (
            Rect::new(346.0, 96.0, 50.0, 28.0),
            Hit::Selector(ApuSelector::Off),
        ),
        (Rect::new(396.0, 52.0, 50.0, 28.0), Hit::Selector(ApuSelector::On)),
        (
            Rect::new(446.0, 96.0, 50.0, 28.0),
            Hit::Selector(ApuSelector::Start),
        ),
        (sw(170.0, 206.0), Hit::ApuGen(0)),
        (sw(260.0, 206.0), Hit::ApuGen(1)),
        (sw(36.0, 324.0), Hit::ExtFwd(0)),
        (sw(124.0, 324.0), Hit::ExtFwd(1)),
        (sw(386.0, 324.0), Hit::ExtAft),
        (Rect::new(36.0, 412.0, 80.0, 20.0), Hit::Cart(0)),
        (Rect::new(124.0, 412.0, 80.0, 20.0), Hit::Cart(1)),
        (Rect::new(386.0, 412.0, 80.0, 20.0), Hit::Cart(2)),
    ];
    for (i, x) in GEN_X.iter().enumerate() {
        v.push((Rect::new(*x, 472.0, SW, SH), Hit::Gen(i)));
        v.push((Rect::new(*x, 570.0, 82.0, 56.0), Hit::Drive(i)));
    }
    v
}

fn glow(l: &Lamp) -> f32 {
    if l.lit { l.brightness as f32 } else { 0.0 }
}

fn lamp_color(l: &Lamp) -> Color {
    match l.color {
        LampColor::Amber => AMBER,
        LampColor::Green => GREEN,
        LampColor::Blue => CYAN,
        LampColor::White => Color::WHITE,
    }
}

fn flag(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

fn caption(c: &mut Canvas, x: f32, y: f32, text: &str) {
    put_mid(c, x, y, text, 10.0, Color::DIM);
}

fn two_state(c: &mut Canvas, r: Rect, top: (&str, f32, Color), bot: (&str, f32, Color)) {
    c.fill_rect(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, Color::PANEL);
    frame(c, r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, Color::LINE);
    let a = Rect::new(r.x + 4.0, r.y + 4.0, r.w - 8.0, r.h / 2.0 - 6.0);
    let b = Rect::new(r.x + 4.0, r.y + r.h / 2.0 + 2.0, r.w - 8.0, r.h / 2.0 - 6.0);
    lamp_cell(c, a, top.0, top.2, top.1);
    lamp_cell(c, b, bot.0, bot.2, bot.1);
}

fn avail_text(on: &Lamp, avail: &Lamp, plugged: bool) -> &'static str {
    if on.lit {
        "ON"
    } else if avail.lit {
        "AVAIL"
    } else if plugged {
        "plugged"
    } else {
        "unplugged"
    }
}

impl OverheadPage {
    pub fn new(state: OverheadState) -> OverheadPage {
        OverheadPage {
            state,
            guard: [false; 4],
            out: Outbox::default(),
            scale: StdCell::new(1.0),
        }
    }

    pub fn set_state(&mut self, state: OverheadState) {
        self.state = state;
    }

    pub fn take_actions(&mut self) -> Vec<PanelAction> {
        self.out.take()
    }

    pub fn pending(&self) -> &[PanelAction] {
        self.out.pending()
    }

    pub fn guard_open(&self, i: usize) -> bool {
        self.guard.get(i).copied().unwrap_or(false)
    }

    pub fn hit_center(&self, target: &str) -> Option<(f32, f32)> {
        let want = match target {
            "BATTERY" => Hit::Battery,
            "IFE_SEATS" => Hit::Ife,
            "CABIN_UTIL" => Hit::Cabin,
            "APU_SELECTOR_OFF" => Hit::Selector(ApuSelector::Off),
            "APU_SELECTOR_ON" => Hit::Selector(ApuSelector::On),
            "APU_SELECTOR_START" => Hit::Selector(ApuSelector::Start),
            "APU_GEN_L" => Hit::ApuGen(0),
            "APU_GEN_R" => Hit::ApuGen(1),
            "EXT_PWR_FWD_L" => Hit::ExtFwd(0),
            "EXT_PWR_FWD_R" => Hit::ExtFwd(1),
            "EXT_PWR_AFT" => Hit::ExtAft,
            "CART_FWD_L" => Hit::Cart(0),
            "CART_FWD_R" => Hit::Cart(1),
            "CART_AFT" => Hit::Cart(2),
            t => {
                let g = GEN_T.iter().position(|n| *n == t).map(Hit::Gen);
                let d = DRV_T.iter().position(|n| *n == t).map(Hit::Drive);
                g.or(d)?
            }
        };
        let s = self.scale.get();
        hits()
            .into_iter()
            .find(|(_, h)| *h == want)
            .map(|(r, _)| ((r.x + r.w / 2.0) * s, (r.y + r.h / 2.0) * s))
    }

    fn press(&mut self, hit: Hit) {
        let st = self.state;
        let tgl = |on: bool| Some(if on { "OFF" } else { "ON" });
        let side = |i: usize| if i == 0 { "L" } else { "R" };
        match hit {
            Hit::Battery => self
                .out
                .push(PanelAction::new("crew", "press", "BATTERY", tgl(st.battery_on))),
            Hit::Ife => self
                .out
                .push(PanelAction::new("crew", "press", "IFE_SEATS", tgl(st.ife_on))),
            Hit::Cabin => self
                .out
                .push(PanelAction::new("crew", "press", "CABIN_UTIL", tgl(st.cabin_on))),
            Hit::Selector(p) => {
                let v = match p {
                    ApuSelector::Off => "OFF",
                    ApuSelector::On => "ON",
                    ApuSelector::Start => "START",
                };
                self.out
                    .push(PanelAction::new("crew", "select", "APU_SELECTOR", Some(v)));
            }
            Hit::ApuGen(i) => {
                let t = format!("APU_GEN_{}", side(i));
                self.out
                    .push(PanelAction::new("crew", "press", &t, tgl(st.apu_gen_on[i])));
            }
            Hit::ExtFwd(i) => {
                let t = format!("EXT_PWR_FWD_{}", side(i));
                self.out.push(PanelAction::new("crew", "press", &t, None));
            }
            Hit::ExtAft => self
                .out
                .push(PanelAction::new("crew", "press", "EXT_PWR_AFT", None)),
            Hit::Cart(k) => {
                if st.plugged[k] {
                    self.out
                        .push(PanelAction::new("ground", "disconnect", CART[k], None));
                } else {
                    self.out
                        .push(PanelAction::new("ground", "connect", CART[k], Some(EXT)));
                }
                self.state.plugged[k] = !st.plugged[k];
            }
            Hit::Gen(i) => self.out.push(PanelAction::new(
                "crew",
                "press",
                GEN_T[i],
                tgl(st.gen_ctrl_on[i]),
            )),
            Hit::Drive(i) => {
                if self.guard[i] {
                    self.out.push(PanelAction::new("crew", "press", DRV_T[i], None));
                    self.guard[i] = false;
                } else {
                    self.guard[i] = true;
                }
            }
        }
    }

    fn knob(&self, c: &mut Canvas) {
        let (cx, cy) = (421.0, 108.0);
        c.fill_circle(cx, cy, 22.0, Color::PANEL);
        c.stroke_circle(cx, cy, 22.0, &Stroke::new(Color::DIM, 1.4));
        let ang: f32 = match self.state.selector {
            ApuSelector::Off => -2.5,
            ApuSelector::On => -std::f32::consts::FRAC_PI_2,
            ApuSelector::Start => -0.64,
        };
        c.line(
            cx,
            cy,
            cx + 18.0 * ang.cos(),
            cy + 18.0 * ang.sin(),
            &Stroke::new(Color::WHITE, 3.0),
        );
        let labels = [
            (ApuSelector::Off, "OFF", 370.0, 112.0),
            (ApuSelector::On, "ON", 421.0, 66.0),
            (ApuSelector::Start, "START", 472.0, 112.0),
        ];
        for (p, t, x, y) in labels {
            let col = if p == self.state.selector {
                Color::WHITE
            } else {
                Color::DIM
            };
            put_mid(c, x, y, t, 11.0, col);
        }
        caption(c, cx, 50.0, "APU");
    }

    fn drive(&self, c: &mut Canvas, i: usize) {
        let r = Rect::new(GEN_X[i], 570.0, 82.0, 56.0);
        c.fill_rect(r.x, r.y, r.w, r.h, Color::PANEL);
        frame(c, r.x, r.y, r.w, r.h, Color::LINE);
        let open = self.guard[i];
        let lamp = Rect::new(r.x + 6.0, r.y + 30.0, r.w - 12.0, 20.0);
        lamp_cell(c, lamp, "DRIVE", AMBER, glow(&self.state.lamps.drive[i]));
        let h = if open { 8.0 } else { 22.0 };
        let cover = Rect::new(r.x + 6.0, r.y + 5.0, r.w - 12.0, h);
        c.fill_rect(cover.x, cover.y, cover.w, cover.h, Color::LINE);
        c.stroke_rect(
            cover.x,
            cover.y,
            cover.w,
            cover.h,
            &Stroke::new(if open { AMBER } else { Color::DIM }, 1.2),
        );
        if !open {
            put_mid(
                c,
                cover.x + cover.w / 2.0,
                cover.y + 15.0,
                "GUARD",
                10.0,
                Color::DIM,
            );
        }
    }

    fn rows(&self) -> Vec<(String, String, bool)> {
        let s = &self.state;
        let onoff = |b: bool| if b { "ON" } else { "OFF" }.to_string();
        let sel = match s.selector {
            ApuSelector::Off => "OFF",
            ApuSelector::On => "ON",
            ApuSelector::Start => "START",
        };
        let l = &s.lamps;
        let mut v = vec![
            ("BATTERY".to_string(), onoff(s.battery_on), l.battery_off.lit),
            ("IFE/PASS SEATS".to_string(), onoff(s.ife_on), l.ife_off.lit),
            (
                "CABIN/UTILITY".to_string(),
                onoff(s.cabin_on),
                l.cabin_util_off.lit,
            ),
            (
                "APU selector".to_string(),
                if s.apu_fault {
                    format!("{sel}  FAULT")
                } else {
                    sel.to_string()
                },
                s.apu_fault,
            ),
        ];
        for i in 0..2 {
            let n = if i == 0 { "APU GEN L" } else { "APU GEN R" };
            let state = if l.apu_gen_on[i].lit {
                "ON"
            } else if l.apu_gen_avail[i].lit {
                "AVAIL"
            } else {
                "OFF"
            };
            v.push((
                n.to_string(),
                state.to_string(),
                l.apu_gen_off[i].lit && s.apu_gen_on[i],
            ));
        }
        for i in 0..2 {
            let n = if i == 0 { "FWD EXT PWR L" } else { "FWD EXT PWR R" };
            let t = avail_text(&l.ext_fwd_on[i], &l.ext_fwd_avail[i], s.plugged[i]);
            v.push((n.to_string(), t.to_string(), false));
        }
        let t = avail_text(&l.ext_aft_on, &l.ext_aft_avail, s.plugged[2]);
        v.push(("AFT EXT PWR".to_string(), t.to_string(), false));
        for (i, n) in ["GEN CTRL L1", "GEN CTRL L2", "GEN CTRL R1", "GEN CTRL R2"]
            .iter()
            .enumerate()
        {
            v.push((n.to_string(), onoff(s.gen_ctrl_on[i]), l.gen_off[i].lit));
        }
        for (i, n) in ["DRIVE DISC L1", "DRIVE DISC L2", "DRIVE DISC R1", "DRIVE DISC R2"]
            .iter()
            .enumerate()
        {
            let t = if l.drive[i].lit {
                "DRIVE lit"
            } else if self.guard[i] {
                "guard open"
            } else {
                "guarded"
            };
            v.push((n.to_string(), t.to_string(), l.drive[i].lit));
        }
        v
    }

    fn side(&self, c: &mut Canvas) {
        let x = 530.0;
        let w = 450.0;
        let rows = self.rows();
        heading(c, x, 28.0, w, "SWITCH POSITIONS AND LAMPS (live)");
        for (i, (n, v, fault)) in rows.iter().enumerate() {
            let y = 50.0 + 17.0 * i as f32;
            put(c, x, y, n, 11.0, Color::DIM);
            put(
                c,
                x + 170.0,
                y,
                v,
                11.0,
                if *fault { AMBER } else { Color::WHITE },
            );
        }
        let y0 = 50.0 + 17.0 * rows.len() as f32 + 18.0;
        heading(c, x, y0, w, "HOW TO USE");
        let help = [
            "click a switch to press it; the action is queued",
            "DRIVE DISC: first click lifts the guard, second presses",
            "APU selector: click OFF, ON or START",
            "PLUG CART: ground cart at the receptacle",
            "amber lamp OFF/FAULT/DRIVE, white lamp ON/AVAIL",
        ];
        for (i, t) in help.iter().enumerate() {
            put(c, x, y0 + 20.0 + 15.0 * i as f32, t, 10.0, Color::DIM);
        }
        let y1 = y0 + 20.0 + 15.0 * help.len() as f32 + 12.0;
        heading(c, x, y1, w, "LAST ACTIONS");
        let recent = self.out.recent();
        if recent.is_empty() {
            put(c, x, y1 + 20.0, "none yet", 10.0, Color::FAINT);
        }
        for (i, t) in recent.iter().rev().take(4).enumerate() {
            let col = if i == 0 { Color::WHITE } else { Color::DIM };
            put(
                c,
                x,
                y1 + 20.0 + 15.0 * i as f32,
                &ellipsize(t, 11.0, w),
                11.0,
                col,
            );
        }
    }
}

impl Page for OverheadPage {
    fn id(&self) -> &str {
        "electrical.overhead"
    }

    fn title(&self) -> &str {
        "Electrical overhead panel"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        self.scale.set(s);
        c.save();
        c.translate(area.x, area.y);
        c.scale(s, s);
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        let st = &self.state;
        let l = &st.lamps;
        c.fill_rect(14.0, 8.0, 492.0, 654.0, Color::PANEL2);
        frame(c, 14.0, 8.0, 492.0, 654.0, Color::DIM);
        put_mid(c, 260.0, 30.0, "ELECTRICAL", 12.0, Color::WHITE);
        caption(c, 76.0, 56.0, "IFE/PASS");
        caption(c, 76.0, 67.0, "SEATS");
        caption(c, 168.0, 56.0, "CABIN/");
        caption(c, 168.0, 67.0, "UTILITY");
        caption(c, 276.0, 67.0, "BATTERY");
        let w = Color::WHITE;
        two_state(
            c,
            sw(36.0, 74.0),
            ("ON", flag(st.ife_on), w),
            ("OFF", glow(&l.ife_off), AMBER),
        );
        two_state(
            c,
            sw(128.0, 74.0),
            ("ON", flag(st.cabin_on), w),
            ("OFF", glow(&l.cabin_util_off), AMBER),
        );
        two_state(
            c,
            sw(236.0, 74.0),
            ("ON", flag(st.battery_on), w),
            ("OFF", glow(&l.battery_off), AMBER),
        );
        self.knob(c);
        lamp_cell(
            c,
            Rect::new(381.0, 146.0, 80.0, 26.0),
            "FAULT",
            AMBER,
            flag(st.apu_fault),
        );
        caption(c, 255.0, 186.0, "APU GEN");
        caption(c, 210.0, 199.0, "L");
        caption(c, 300.0, 199.0, "R");
        for i in 0..2 {
            let x = 170.0 + 90.0 * i as f32;
            let top = if l.apu_gen_on[i].lit {
                ("ON", glow(&l.apu_gen_on[i]), lamp_color(&l.apu_gen_on[i]))
            } else if l.apu_gen_avail[i].lit {
                (
                    "AVAIL",
                    glow(&l.apu_gen_avail[i]),
                    lamp_color(&l.apu_gen_avail[i]),
                )
            } else {
                ("ON", flag(st.apu_gen_on[i]) * 0.45, w)
            };
            two_state(c, sw(x, 206.0), top, ("OFF", glow(&l.apu_gen_off[i]), AMBER));
        }
        caption(c, 120.0, 304.0, "FWD EXT PWR");
        caption(c, 76.0, 316.0, "L");
        caption(c, 164.0, 316.0, "R");
        caption(c, 426.0, 304.0, "AFT EXT PWR");
        for i in 0..2 {
            let x = 36.0 + 88.0 * i as f32;
            two_state(
                c,
                sw(x, 324.0),
                ("ON", glow(&l.ext_fwd_on[i]), w),
                ("AVAIL", glow(&l.ext_fwd_avail[i]), w),
            );
        }
        two_state(
            c,
            sw(386.0, 324.0),
            ("ON", glow(&l.ext_aft_on), w),
            ("AVAIL", glow(&l.ext_aft_avail), w),
        );
        for (x, k) in [(36.0, 0), (124.0, 1), (386.0, 2)] {
            let t = if st.plugged[k] { "UNPLUG CART" } else { "PLUG CART" };
            button(c, Rect::new(x, 412.0, 80.0, 20.0), t, true, st.plugged[k]);
        }
        c.line(46.0, 442.0, 466.0, 442.0, &Stroke::new(Color::DIM, 3.0));
        put_mid(c, 256.0, 458.0, "AC BUSES", 10.0, Color::DIM);
        for (i, n) in ["L1", "L2", "R1", "R2"].iter().enumerate() {
            let x = GEN_X[i];
            caption(c, x + 40.0, 468.0, &format!("GEN CTRL {n}"));
            two_state(
                c,
                Rect::new(x, 472.0, SW, SH),
                ("ON", flag(st.gen_ctrl_on[i]), w),
                ("OFF", glow(&l.gen_off[i]), AMBER),
            );
            self.drive(c, i);
        }
        caption(c, 256.0, 646.0, "L1    L2     DRIVE DISC     R1    R2");
        self.side(c);
        c.restore();
    }

    fn click(&mut self, x: f32, y: f32) -> bool {
        let s = self.scale.get().max(0.01);
        let (px, py) = (x / s, y / s);
        let found = hits()
            .into_iter()
            .find(|(r, _)| r.contains(px, py))
            .map(|(_, h)| h);
        match found {
            Some(h) => {
                self.press(h);
                true
            }
            None => false,
        }
    }

    fn detail_title(&self) -> String {
        String::from("Overhead ELEC panel")
    }

    fn detail(&self) -> Vec<DetailSection> {
        vec![DetailSection {
            title: String::from("Switches"),
            rows: self
                .rows()
                .into_iter()
                .map(|(n, v, f)| (n, if f { Cell::fault(&v) } else { Cell::new(&v) }))
                .collect(),
        }]
    }

    fn table(&self) -> Table {
        Table {
            title: String::from("Recent actions"),
            columns: vec![String::from("action")],
            rows: self
                .out
                .recent()
                .iter()
                .rev()
                .map(|t| vec![Cell::new(t)])
                .collect(),
        }
    }
}
