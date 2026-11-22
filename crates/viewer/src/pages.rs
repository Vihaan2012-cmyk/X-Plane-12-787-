use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, Stroke};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub struct Shared<P: Page> {
    id: String,
    title: String,
    page: Rc<RefCell<P>>,
}

impl<P: Page> Shared<P> {
    pub fn new(page: P) -> (Shared<P>, Rc<RefCell<P>>) {
        let id = page.id().to_string();
        let title = page.title().to_string();
        let rc = Rc::new(RefCell::new(page));
        (
            Shared {
                id,
                title,
                page: rc.clone(),
            },
            rc,
        )
    }
}

impl<P: Page> Page for Shared<P> {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn draw(&self, canvas: &mut Canvas, area: Rect) {
        self.page.borrow().draw(canvas, area);
    }
    fn click(&mut self, x: f32, y: f32) -> bool {
        self.page.borrow_mut().click(x, y)
    }
    fn detail_title(&self) -> String {
        self.page.borrow().detail_title()
    }
    fn detail(&self) -> Vec<DetailSection> {
        self.page.borrow().detail()
    }
    fn table(&self) -> Table {
        self.page.borrow().table()
    }
}

pub struct StatusPage {
    id: String,
    title: String,
    plans: Vec<(String, String)>,
    facts: Vec<(String, usize)>,
    crate_dir: String,
}

type SystemRow = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static [&'static str],
    &'static str,
);

const SYSTEMS: &[SystemRow] = &[
    (
        "engines",
        "Engines",
        &[("71-engines.md", "draft, from 711 verified facts")],
        &["eng_arch", "eng_crew", "eng_tcds"],
        "71-engines",
    ),
    ("apu", "APU", &[], &["facts_apu"], "49-apu"),
    ("fuel", "Fuel", &[], &["facts_fuel"], "28-fuel"),
    ("hydraulics", "Hydraulics", &[], &["facts_hyd"], "29-hydraulics"),
    (
        "flight-controls",
        "Flight controls",
        &[],
        &["facts_fctl"],
        "27-flight-controls",
    ),
    ("gear-brakes", "Gear, brakes", &[], &["facts_gear"], "32-gear"),
    ("autoflight", "Autoflight", &[], &["facts_afds"], "22-autoflight"),
    ("fms", "FMS", &[], &["facts_fms"], "34-fms"),
    (
        "air-pressurisation",
        "Air, pressurisation",
        &[],
        &["facts_ecs"],
        "21-air",
    ),
    ("ice-rain", "Ice and rain", &[], &["facts_ice"], "30-ice-rain"),
    ("fire", "Fire", &[], &["facts_fire"], "26-fire"),
    (
        "air-data",
        "Air data, inertial",
        &[("34-adirs.md", "draft"), ("34-gnss.md", "draft")],
        &["facts_nav"],
        "34-adirs",
    ),
    (
        "nav-radios",
        "Navigation radios",
        &[("shared-radio.md", "draft")],
        &["facts_nav"],
        "34-radio-nav",
    ),
    (
        "surveillance",
        "Surveillance",
        &[],
        &["facts_surv"],
        "34-surveillance",
    ),
    ("comms", "Communications", &[], &["facts_comms"], "23-comms"),
    (
        "displays-alerting",
        "Displays, alerting",
        &[],
        &["facts_displays"],
        "31-displays",
    ),
    (
        "ccs",
        "CCS computers",
        &[("42-ccs-cdn.md", "draft"), ("computers.md", "286 boxes")],
        &["facts_ccs"],
        "42-ccs",
    ),
    (
        "cdn",
        "CDN network",
        &[("42-ccs-cdn.md", "draft"), ("data-flows.md", "about 220 flows")],
        &["facts_ccs"],
        "42-ccs",
    ),
    (
        "lights",
        "Lights",
        &[("33-exterior-lights.md", "draft")],
        &["facts_lights"],
        "33-lights",
    ),
    (
        "doors-oxygen-water",
        "Doors, oxygen, water",
        &[],
        &["facts_cabin"],
        "52-doors",
    ),
    (
        "efb",
        "EFB, info systems",
        &[("46-efb.md", "draft")],
        &["facts_info"],
        "46-efb",
    ),
];

fn count_facts(root: &Path, stem: &str) -> usize {
    let p = root
        .join("refs")
        .join("facts")
        .join(format!("{stem}.verified.md"));
    std::fs::read_to_string(p)
        .map(|t| {
            t.lines()
                .filter(|l| l.starts_with("| ") && !l.starts_with("| Topic"))
                .count()
        })
        .unwrap_or(0)
}

pub fn status_pages(root: &Path) -> Vec<StatusPage> {
    SYSTEMS
        .iter()
        .map(|(id, title, plans, facts, krate)| StatusPage {
            id: id.to_string(),
            title: title.to_string(),
            plans: plans
                .iter()
                .map(|(f, s)| (format!("docs/systems/{f}"), s.to_string()))
                .collect(),
            facts: facts
                .iter()
                .map(|f| (format!("refs/facts/{f}.verified.md"), count_facts(root, f)))
                .collect(),
            crate_dir: format!("crates/systems/{krate}"),
        })
        .collect()
}

impl Page for StatusPage {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn draw(&self, c: &mut Canvas, area: Rect) {
        let x = area.x + 32.0;
        let mut y = area.y + 48.0;
        c.text(x, y, &self.title, 20.0, Color::WHITE);
        y += 22.0;
        c.line(x, y, area.x + area.w - 32.0, y, &Stroke::new(Color::LINE, 1.0));
        y += 30.0;
        c.text(x, y, "live model", 12.0, Color::DIM);
        c.text(
            x + 160.0,
            y,
            "not built yet (systems are built one at a time; electrical is first)",
            12.0,
            Color::WHITE,
        );
        y += 30.0;
        c.text(x, y, "plan", 12.0, Color::DIM);
        if self.plans.is_empty() {
            c.text(x + 160.0, y, "not written yet", 12.0, Color::WHITE);
            y += 22.0;
        }
        for (f, s) in &self.plans {
            c.text(x + 160.0, y, &format!("{f}  ({s})"), 12.0, Color::WHITE);
            y += 22.0;
        }
        y += 8.0;
        c.text(x, y, "verified facts", 12.0, Color::DIM);
        for (f, n) in &self.facts {
            c.text(x + 160.0, y, &format!("{n:>4}  {f}"), 12.0, Color::WHITE);
            y += 22.0;
        }
        y += 8.0;
        c.text(x, y, "crate", 12.0, Color::DIM);
        c.text(
            x + 160.0,
            y,
            &format!("{}  (skeleton)", self.crate_dir),
            12.0,
            Color::WHITE,
        );
    }
    fn click(&mut self, _x: f32, _y: f32) -> bool {
        false
    }
    fn detail(&self) -> Vec<DetailSection> {
        vec![DetailSection {
            title: String::from("Status"),
            rows: vec![
                (String::from("model"), Cell::new("not built")),
                (String::from("plans"), Cell::new(self.plans.len().to_string())),
                (
                    String::from("facts"),
                    Cell::new(self.facts.iter().map(|f| f.1).sum::<usize>().to_string()),
                ),
            ],
        }]
    }
}
