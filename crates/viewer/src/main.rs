mod arial;
mod pages;

use pages::{Shared, status_pages};
use render::{Canvas, Color, Stroke, ellipsize, fit_size, write_png};
use std::cell::RefCell;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};
use sys_electrical::crew::Level;
use sys_electrical::inspector::{
    BreakerPage, CbCommand, CbicView, EicasPage, InspectorData, OverheadPage, OverheadState, PanelAction,
    PanelPage, ProtectionPage, RpduPage, SingleLinePage, SynopticPage, seed_cbic,
};
use sys_electrical::system::Electrical;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const FRAME_US: u64 = 33_333;
const SPEEDS: [u64; 5] = [1, 4, 16, 60, 240];
const SEEDS: [&str; 6] = [
    "cold and dark",
    "battery on",
    "external power",
    "APU running",
    "engines running",
    "cruise",
];
const EXT: &str = "v_ll = 115.0, hz = 400.0, kva = 90.0";
const KEEP: usize = 3;

struct Args {
    root: Option<PathBuf>,
    shots: Option<PathBuf>,
}

fn parse_args() -> Args {
    let mut a = Args {
        root: None,
        shots: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        if x == "--shots" {
            a.shots = it.next().map(PathBuf::from);
        } else {
            a.root = Some(PathBuf::from(x));
        }
    }
    a
}

fn find_root(arg: Option<PathBuf>) -> Option<PathBuf> {
    if arg.is_some() {
        return arg;
    }
    let mut cur = std::env::current_dir().ok();
    while let Some(d) = cur {
        if d.join("aircraft").join("24-electrical").is_dir() {
            return Some(d);
        }
        cur = d.parent().map(Path::to_path_buf);
    }
    None
}

struct Sim {
    sys: Electrical,
    data_dir: PathBuf,
    t_us: u64,
    speed: usize,
    paused: bool,
    help: bool,
    seed_idx: usize,
    history: Vec<String>,
    last: String,
    seed_name: String,
}

impl Sim {
    fn new(data_dir: PathBuf) -> Result<Sim, String> {
        let mut s = Sim {
            sys: Electrical::new(&data_dir)?,
            data_dir,
            t_us: 0,
            speed: 0,
            paused: false,
            help: true,
            seed_idx: 0,
            history: Vec::new(),
            last: String::new(),
            seed_name: String::new(),
        };
        s.seed(0);
        Ok(s)
    }

    fn act(&mut self, actor: &str, action: &str, target: &str, value: Option<&str>) {
        let line = match self.sys.action(actor, action, target, value) {
            Ok(()) => format!("{action} {target} {}", value.unwrap_or(""))
                .trim_end()
                .to_string(),
            Err(e) => format!("refused {action} {target}: {e}"),
        };
        self.note(line);
    }

    fn note(&mut self, line: String) {
        self.history.push(line.clone());
        if self.history.len() > KEEP {
            self.history.remove(0);
        }
        self.last = line;
    }

    fn set(&mut self, key: &str, value: &str) {
        let _ = self.sys.action("sim", "set", key, Some(value));
    }

    fn seed(&mut self, n: usize) {
        if let Ok(s) = Electrical::new(&self.data_dir) {
            self.sys = s;
        }
        self.t_us = 0;
        self.seed_idx = n;
        self.history.clear();
        self.seed_name = SEEDS[n].to_string();
        let ground = n < 5;
        self.set("airplane_state", if ground { "cold_dark" } else { "cruise" });
        self.set("on_ground", if ground { "true" } else { "false" });
        self.set("oat_c", "15.0");
        self.set("gen_ctrl_switches", "ON");
        self.set("apu_battery_connected", "true");
        self.set("battery_switch", if n >= 1 { "ON" } else { "OFF" });
        match n {
            2 => {
                self.act("ground", "connect", "EXT_PWR_FWD_L", Some(EXT));
                self.act("ground", "connect", "EXT_PWR_FWD_R", Some(EXT));
                self.step(2_000_000);
                self.act("crew", "press", "EXT_PWR_FWD_L", Some("ON"));
            }
            3 => {
                self.set("apu_running", "true");
                self.set("apu_gens_online", "true");
            }
            4 => self.set("engines_running", "[true, true]"),
            5 => {
                self.set("alt_ft", "35000.0");
                self.set("engines_running", "[true, true]");
            }
            _ => {}
        }
        self.step(5_000_000);
        self.fill_cbic();
        self.history.clear();
        self.note(format!("seed {}", SEEDS[n]));
    }

    fn fill_cbic(&mut self) {
        if self.sys.cbic().app.search("").is_err()
            && let Ok(csv) = std::fs::read_to_string(self.data_dir.join("breakers.csv"))
        {
            seed_cbic(&mut self.sys, &csv, 60, 20);
        }
    }

    fn step(&mut self, dt_us: u64) {
        self.sys.step_idle(dt_us);
        self.t_us += dt_us;
    }

    fn data(&self) -> InspectorData {
        InspectorData::from_system(&self.sys)
    }

    fn overhead(&self) -> OverheadState {
        let mut st = OverheadState::from_panel(self.sys.panel(), self.sys.lamps());
        for (k, e) in self.sys.inputs().ext.iter().enumerate() {
            st.plugged[k] = e.connected;
        }
        st.apu_fault = self
            .sys
            .eicas_lines()
            .iter()
            .any(|l| l.name.contains("APU") && matches!(l.level, Level::Warning | Level::Caution));
        st
    }

    fn key(&mut self, k: KeyCode) {
        let fkeys = [
            KeyCode::F1,
            KeyCode::F2,
            KeyCode::F3,
            KeyCode::F4,
            KeyCode::F5,
            KeyCode::F6,
        ];
        if let Some(i) = fkeys.iter().position(|f| *f == k) {
            self.seed(i);
            return;
        }
        let gens = [
            (KeyCode::Digit1, "GEN_CTRL_L1"),
            (KeyCode::Digit2, "GEN_CTRL_L2"),
            (KeyCode::Digit3, "GEN_CTRL_R1"),
            (KeyCode::Digit4, "GEN_CTRL_R2"),
        ];
        if let Some(g) = gens.iter().find(|g| g.0 == k) {
            self.act("crew", "press", g.1, None);
            return;
        }
        match k {
            KeyCode::KeyB => self.act("crew", "press", "BATTERY", None),
            KeyCode::KeyC => self.seed(self.seed_idx),
            KeyCode::KeyL => {
                self.act("ground", "connect", "EXT_PWR_FWD_L", Some(EXT));
                self.act("crew", "press", "EXT_PWR_FWD_L", Some("ON"));
            }
            KeyCode::KeyR => {
                self.act("ground", "connect", "EXT_PWR_FWD_R", Some(EXT));
                self.act("crew", "press", "EXT_PWR_FWD_R", Some("ON"));
            }
            KeyCode::KeyK => {
                self.act("crew", "press", "EXT_PWR_FWD_L", Some("OFF"));
                self.act("crew", "press", "EXT_PWR_FWD_R", Some("OFF"));
            }
            KeyCode::KeyA => self.act("crew", "select", "APU_SWITCH", Some("START")),
            KeyCode::KeyG => {
                self.act("crew", "press", "APU_GEN_L", Some("ON"));
                self.act("crew", "press", "APU_GEN_R", Some("ON"));
            }
            KeyCode::KeyS => {
                self.act("crew", "rotate", "ENG_START_L", Some("START"));
                self.act("crew", "move", "FUEL_CONTROL_L", Some("RUN"));
            }
            KeyCode::KeyD => self.act("fault", "vfsg_fail", "L1", Some("drive_disconnect")),
            KeyCode::KeyF => self.act("fault", "gen_loss", "ALL_VFSG", None),
            KeyCode::KeyX => self.act("fault", "bus_fail", "L1_235", Some("dead_short_lockout")),
            KeyCode::KeyP => self.act("fault", "bpcu_fail", "L_AND_R", None),
            KeyCode::KeyO => self.act("fault", "all_engines_flameout", "ALL", None),
            KeyCode::Space => self.paused = !self.paused,
            KeyCode::Equal | KeyCode::NumpadAdd | KeyCode::PageUp => {
                self.speed = (self.speed + 1).min(SPEEDS.len() - 1)
            }
            KeyCode::Minus | KeyCode::NumpadSubtract | KeyCode::PageDown => {
                self.speed = self.speed.saturating_sub(1)
            }
            KeyCode::KeyH => self.help = !self.help,
            _ => {}
        }
    }
}

struct Live {
    single: Rc<RefCell<SingleLinePage>>,
    panels: Rc<RefCell<PanelPage>>,
    rpdu: Rc<RefCell<RpduPage>>,
    breaker: Rc<RefCell<BreakerPage>>,
    overhead: Rc<RefCell<OverheadPage>>,
    synoptic: Rc<RefCell<SynopticPage>>,
    eicas: Rc<RefCell<EicasPage>>,
    protection: Rc<RefCell<ProtectionPage>>,
}

struct Viewer {
    sim: Sim,
    tablet: tablet::Tablet,
    live: Live,
    order: Vec<String>,
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    cursor: (f64, f64),
}

impl Viewer {
    fn new(root: &Path) -> Result<Viewer, String> {
        let sim = Sim::new(root.join("aircraft").join("24-electrical"))?;
        let data = sim.data();
        let (s1, single) = Shared::new(SingleLinePage::new(data.clone()));
        let (s2, panels) = Shared::new(PanelPage::new(data.clone()));
        let (s3, rpdu) = Shared::new(RpduPage::new(data.clone()));
        let rid = data
            .rpdus
            .first()
            .map(|r| r.id.clone())
            .unwrap_or_else(|| String::from("RPDU 75"));
        let (s4, breaker) = Shared::new(BreakerPage::new(data.clone(), &rid, 1));
        let (s5, overhead) = Shared::new(OverheadPage::new(sim.overhead()));
        let (s6, synoptic) = Shared::new(SynopticPage::new(sim.sys.synoptic().clone()));
        let (s8, protection) = Shared::new(ProtectionPage::new(data.clone()));
        let (s7, eicas) = Shared::new(EicasPage::new(data));
        let mut tablet = tablet::Tablet::new();
        tablet.register(Box::new(s1));
        tablet.register(Box::new(s2));
        tablet.register(Box::new(s3));
        tablet.register(Box::new(s4));
        tablet.register(Box::new(s5));
        tablet.register(Box::new(s6));
        tablet.register(Box::new(s7));
        tablet.register(Box::new(s8));
        for p in status_pages(root) {
            tablet.register(Box::new(p));
        }
        tablet.select("electrical");
        let order = tablet.page_ids();
        Ok(Viewer {
            sim,
            tablet,
            live: Live {
                single,
                panels,
                rpdu,
                breaker,
                overhead,
                synoptic,
                eicas,
                protection,
            },
            order,
            window: None,
            surface: None,
            cursor: (0.0, 0.0),
        })
    }

    fn refresh(&mut self) {
        let data = self.sim.data();
        self.live.single.borrow_mut().set_data(data.clone());
        self.live.panels.borrow_mut().set_data(data.clone());
        self.live.rpdu.borrow_mut().set_data(data.clone());
        self.live.breaker.borrow_mut().set_data(data.clone());
        self.live.protection.borrow_mut().set_data(data.clone());
        self.live.overhead.borrow_mut().set_state(self.sim.overhead());
        self.live
            .synoptic
            .borrow_mut()
            .set_data(self.sim.sys.synoptic().clone());
        {
            let mut e = self.live.eicas.borrow_mut();
            e.set_data(data.clone());
            e.set_cbic(CbicView::from_system(&self.sim.sys));
        }
        self.tablet.set_sim_time(data.sim_time_s);
        self.tablet.set_phase(self.sim.sys.mode());
        self.tablet.set_view(&self.sim.seed_name);
    }

    fn drain(&mut self) {
        let mut queued: Vec<PanelAction> = self.live.overhead.borrow_mut().take_actions();
        queued.extend(self.live.eicas.borrow_mut().take_actions());
        for a in queued {
            self.sim.act(&a.actor, &a.action, &a.target, a.value.as_deref());
        }
    }

    fn frame(&mut self, w: usize, h: usize, scale: f32) -> Canvas {
        self.refresh();
        self.tablet.set_size(w as f32 / scale, h as f32 / scale);
        let mut c = Canvas::new(w, h);
        c.save();
        c.scale(scale, scale);
        self.tablet.draw(&mut c);
        if self.sim.help {
            draw_help(&mut c, &self.sim, &self.tablet.layout());
        }
        c.restore();
        c
    }

    fn redraw(&mut self) {
        let (Some(w), Some(_)) = (self.window.clone(), self.surface.as_ref()) else {
            return;
        };
        let size = w.inner_size();
        let (Some(pw), Some(ph)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };
        let scale = w.scale_factor() as f32;
        let c = self.frame(size.width as usize, size.height as usize, scale);
        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        if surface.resize(pw, ph).is_err() {
            return;
        }
        let Ok(mut buf) = surface.buffer_mut() else {
            return;
        };
        for (px, d) in c.rgba().chunks_exact(4).zip(buf.iter_mut()) {
            *d = ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | px[2] as u32;
        }
        let _ = buf.present();
    }

    fn page_key(&mut self, code: KeyCode) -> bool {
        if self.tablet.selected() != "electrical.eicas" {
            return false;
        }
        let mut e = self.live.eicas.borrow_mut();
        match code {
            KeyCode::ArrowUp => e.select_prev(),
            KeyCode::ArrowDown => e.select_next(),
            KeyCode::KeyZ => {
                e.command(CbCommand::Open);
            }
            KeyCode::KeyV => {
                e.command(CbCommand::Close);
            }
            KeyCode::KeyN => {
                e.command(CbCommand::Lock);
            }
            KeyCode::KeyM => {
                e.command(CbCommand::Unlock);
            }
            KeyCode::KeyY => {
                e.command(CbCommand::Reset);
            }
            _ => return false,
        }
        true
    }
}

fn draw_help(c: &mut Canvas, sim: &Sim, l: &tablet::Layout) {
    let lines = [
        "F1-F6 seed   C reseed, clears faults   H help",
        "B batt  L R ext on  K ext off  A APU start  G APU gens",
        "1-4 GEN CTRL  S start L engine  Space pause  + - speed",
        "D drive L1  F all VFSG  X L1 short  P BPCU  O flameout",
        "Tab next page   click switches on the overhead page",
        "EICAS page: Up Down Z open V close N lock M unlock Y reset",
    ];
    let w = l.side_w - 24.0;
    let rows = lines.len() + KEEP + 1;
    let h = 16.0 * rows as f32 + 30.0;
    let x = l.width - l.side_w + 12.0;
    let y = l.height - h - 12.0;
    c.fill_rect(x, y, w, h, Color::PANEL2);
    c.stroke_rect(x, y, w, h, &Stroke::new(Color::LINE, 1.0));
    let fit = |s: &str| fit_size(s, 11.0, 10.0, w - 20.0);
    for (i, s) in lines.iter().enumerate() {
        c.text(x + 10.0, y + 20.0 + 16.0 * i as f32, s, fit(s), Color::DIM);
    }
    let y1 = y + 20.0 + 16.0 * lines.len() as f32;
    c.line(
        x + 8.0,
        y1 - 10.0,
        x + w - 8.0,
        y1 - 10.0,
        &Stroke::new(Color::LINE, 1.0),
    );
    let n = sim.history.len();
    for (i, s) in sim.history.iter().enumerate() {
        let col = if i + 1 == n { Color::WHITE } else { Color::DIM };
        let t = ellipsize(s, 11.0, w - 20.0);
        c.text(x + 10.0, y1 + 6.0 + 16.0 * i as f32, &t, 11.0, col);
    }
    let status = format!(
        "x{}{}  {}",
        SPEEDS[sim.speed],
        if sim.paused { " paused" } else { "" },
        sim.seed_name
    );
    c.text(
        x + 10.0,
        y + h - 10.0,
        &ellipsize(&status, 11.0, w - 20.0),
        11.0,
        Color::FAINT,
    );
}

#[allow(clippy::disallowed_methods)]
fn next_frame() -> Instant {
    Instant::now() + Duration::from_micros(FRAME_US)
}

impl ApplicationHandler for Viewer {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("787-9 sim tablet")
            .with_inner_size(LogicalSize::new(1600.0, 960.0));
        let Ok(w) = el.create_window(attrs) else {
            el.exit();
            return;
        };
        let w = Rc::new(w);
        let Ok(ctx) = softbuffer::Context::new(w.clone()) else {
            el.exit();
            return;
        };
        let Ok(surface) = softbuffer::Surface::new(&ctx, w.clone()) else {
            el.exit();
            return;
        };
        self.window = Some(w);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => self.cursor = (position.x, position.y),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let s = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                self.tablet
                    .click((self.cursor.0 / s) as f32, (self.cursor.1 / s) as f32);
                self.drain();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => match code {
                KeyCode::Escape => el.exit(),
                KeyCode::Tab => {
                    let cur = self
                        .order
                        .iter()
                        .position(|p| p == self.tablet.selected())
                        .unwrap_or(0);
                    let next = self.order[(cur + 1) % self.order.len()].clone();
                    self.tablet.select(&next);
                }
                other => {
                    if !self.page_key(other) {
                        self.sim.key(other);
                    }
                    self.drain();
                }
            },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        self.drain();
        if !self.sim.paused {
            self.sim.step(FRAME_US * SPEEDS[self.sim.speed]);
        }
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        el.set_control_flow(ControlFlow::WaitUntil(next_frame()));
    }
}

fn shots(viewer: &mut Viewer, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let plan: [(usize, &str, &str); 8] = [
        (0, "electrical.overhead", "overhead-cold"),
        (4, "electrical.overhead", "overhead-engines"),
        (2, "electrical.synoptic", "synoptic-ext"),
        (4, "electrical.synoptic", "synoptic-engines"),
        (3, "electrical.synoptic", "synoptic-apu"),
        (4, "electrical.eicas", "eicas-engines"),
        (5, "electrical.eicas", "eicas-cruise"),
        (4, "electrical.protection", "protection-engines"),
    ];
    for (seed, page, name) in plan {
        viewer.sim.seed(seed);
        if name == "synoptic-engines" || name == "eicas-engines" {
            viewer
                .sim
                .act("fault", "vfsg_fail", "L1", Some("drive_disconnect"));
            viewer.sim.step(3_000_000);
        }
        viewer.tablet.select(page);
        let c = viewer.frame(1600, 960, 1.0);
        let path = dir.join(format!("{name}.png"));
        write_png(&path, c.width(), c.height(), c.rgba()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn run() -> Result<(), String> {
    arial::install();
    let args = parse_args();
    let root = find_root(args.root)
        .ok_or("project folder with aircraft/24-electrical not found; pass it as the first argument")?;
    let mut viewer = Viewer::new(&root)?;
    if let Some(dir) = args.shots {
        return shots(&mut viewer, &dir);
    }
    let el = EventLoop::new().map_err(|e| e.to_string())?;
    el.run_app(&mut viewer).map_err(|e| e.to_string())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("viewer: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewer() -> Viewer {
        let root = PathBuf::from("e:/X-Plane 787");
        Viewer::new(&root).unwrap()
    }

    #[test]
    fn overhead_click_reaches_the_sim() {
        let mut v = viewer();
        v.sim.seed(1);
        v.tablet.select("electrical.overhead");
        let _ = v.frame(1600, 960, 1.0);
        let l = v.tablet.layout().diagram();
        let (x, y) = v.live.overhead.borrow().hit_center("BATTERY").unwrap();
        assert!(v.tablet.click(l.x + x, l.y + y));
        v.drain();
        assert!(v.sim.history.last().unwrap().starts_with("press BATTERY"));
        assert!(!v.sim.sys.panel().battery.is_on());
    }

    #[test]
    fn eicas_keys_queue_cbic_actions() {
        let mut v = viewer();
        v.tablet.select("electrical.eicas");
        let _ = v.frame(1600, 960, 1.0);
        assert!(v.page_key(KeyCode::KeyZ));
        v.drain();
        assert!(v.sim.history.last().unwrap().contains("cbic_open"));
    }

    #[test]
    fn c_reseeds_and_clears_faults() {
        let mut v = viewer();
        v.sim.seed(4);
        v.sim.key(KeyCode::KeyF);
        v.sim.key(KeyCode::KeyC);
        assert_eq!(v.sim.seed_name, SEEDS[4]);
        assert_eq!(v.sim.history.len(), 1);
    }

    #[test]
    fn nav_lists_the_new_pages() {
        let v = viewer();
        let ids = v.tablet.page_ids();
        for id in ["electrical.overhead", "electrical.synoptic", "electrical.eicas"] {
            assert!(ids.iter().any(|p| p == id));
            assert!(tablet::NAV.iter().any(|g| g.items.iter().any(|i| i.id == id)));
        }
    }
}
