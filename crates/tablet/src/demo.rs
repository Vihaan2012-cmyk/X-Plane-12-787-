use crate::page::{Cell, DetailSection, Page, Table};
use render::{Anchor, Canvas, Color, Rect, Stroke, TextStyle};

#[derive(Clone, Debug)]
struct Breaker {
    name: &'static str,
    feeds: &'static str,
    x: f32,
    y: f32,
    closed: bool,
    tripped: bool,
    amps: f32,
}

#[derive(Clone, Debug)]
struct Bus {
    name: &'static str,
    x0: f32,
    x1: f32,
    y: f32,
    volts: f32,
    live: bool,
}

pub struct DemoElectricalPage {
    buses: Vec<Bus>,
    breakers: Vec<Breaker>,
    selected: usize,
}

const B: f32 = 7.0;

impl DemoElectricalPage {
    pub fn new() -> DemoElectricalPage {
        let buses = vec![
            Bus {
                name: "L MAIN AC",
                x0: 60.0,
                x1: 400.0,
                y: 220.0,
                volts: 115.0,
                live: true,
            },
            Bus {
                name: "R MAIN AC",
                x0: 560.0,
                x1: 900.0,
                y: 220.0,
                volts: 115.0,
                live: true,
            },
            Bus {
                name: "L DC",
                x0: 60.0,
                x1: 400.0,
                y: 400.0,
                volts: 28.1,
                live: true,
            },
            Bus {
                name: "R DC",
                x0: 560.0,
                x1: 900.0,
                y: 400.0,
                volts: 0.0,
                live: false,
            },
        ];
        let breakers = vec![
            Breaker {
                name: "GCB L",
                feeds: "GEN L > L MAIN AC",
                x: 230.0,
                y: 150.0,
                closed: true,
                tripped: false,
                amps: 212.0,
            },
            Breaker {
                name: "GCB R",
                feeds: "GEN R > R MAIN AC",
                x: 730.0,
                y: 150.0,
                closed: true,
                tripped: false,
                amps: 198.0,
            },
            Breaker {
                name: "BTB",
                feeds: "L MAIN AC <> R MAIN AC",
                x: 480.0,
                y: 220.0,
                closed: false,
                tripped: false,
                amps: 0.0,
            },
            Breaker {
                name: "TRU L CB",
                feeds: "L MAIN AC > TRU L",
                x: 230.0,
                y: 280.0,
                closed: true,
                tripped: false,
                amps: 41.5,
            },
            Breaker {
                name: "TRU R CB",
                feeds: "R MAIN AC > TRU R",
                x: 730.0,
                y: 280.0,
                closed: false,
                tripped: true,
                amps: 0.0,
            },
            Breaker {
                name: "BAT CB",
                feeds: "BATTERY > L DC",
                x: 230.0,
                y: 470.0,
                closed: true,
                tripped: false,
                amps: 3.2,
            },
        ];
        DemoElectricalPage {
            buses,
            breakers,
            selected: 4,
        }
    }

    pub fn selected_breaker(&self) -> &str {
        self.breakers[self.selected].name
    }

    pub fn tripped_count(&self) -> usize {
        self.breakers.iter().filter(|b| b.tripped).count()
    }

    fn breaker_color(b: &Breaker) -> Color {
        if b.tripped {
            Color::BAD
        } else if b.closed {
            Color::WHITE
        } else {
            Color::DIM
        }
    }

    fn draw_breaker(c: &mut Canvas, b: &Breaker, selected: bool) {
        let col = Self::breaker_color(b);
        let s = Stroke::new(col, 1.2);
        if selected {
            c.stroke_rect(
                b.x - B - 4.0,
                b.y - B - 4.0,
                2.0 * B + 8.0,
                2.0 * B + 8.0,
                &Stroke::new(Color::FAINT, 1.0).dashed(&[3.0, 2.0]),
            );
        }
        c.fill_rect(b.x - B, b.y - B, 2.0 * B, 2.0 * B, Color::BG);
        c.stroke_rect(b.x - B, b.y - B, 2.0 * B, 2.0 * B, &s);
        if b.tripped {
            c.line(b.x - B + 3.0, b.y - B + 3.0, b.x + B - 3.0, b.y + B - 3.0, &s);
            c.line(b.x + B - 3.0, b.y - B + 3.0, b.x - B + 3.0, b.y + B - 3.0, &s);
        } else if b.closed {
            c.fill_rect(b.x - B + 3.0, b.y - B + 3.0, 2.0 * B - 6.0, 2.0 * B - 6.0, col);
        }
        let style =
            TextStyle::new(11.0, if b.tripped { Color::BAD } else { Color::DIM }).anchor(Anchor::Start);
        c.draw_text(b.x + B + 8.0, b.y + 4.0, b.name, &style);
    }

    fn box_with_label(c: &mut Canvas, x: f32, y: f32, w: f32, h: f32, label: &str, col: Color) {
        c.stroke_rect(x, y, w, h, &Stroke::new(col, 1.2));
        c.draw_text(
            x + w / 2.0,
            y + h / 2.0 + 4.0,
            label,
            &TextStyle::new(11.0, col).anchor(Anchor::Middle),
        );
    }
}

impl Default for DemoElectricalPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for DemoElectricalPage {
    fn id(&self) -> &str {
        "electrical"
    }

    fn title(&self) -> &str {
        "Electrical"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        c.save();
        c.translate(area.x, area.y);
        let dim = Stroke::new(Color::FAINT, 1.0);
        let live = Stroke::new(Color::WHITE, 1.2);
        c.draw_text(
            16.0,
            26.0,
            "ELECTRICAL SINGLE LINE  787-9  sheet 1 of 4",
            &TextStyle::new(11.0, Color::DIM),
        );
        Self::box_with_label(c, 190.0, 60.0, 80.0, 40.0, "GEN L", Color::WHITE);
        Self::box_with_label(c, 690.0, 60.0, 80.0, 40.0, "GEN R", Color::WHITE);
        Self::box_with_label(c, 440.0, 60.0, 80.0, 40.0, "APU GEN", Color::DIM);
        c.line(230.0, 100.0, 230.0, 220.0, &live);
        c.line(730.0, 100.0, 730.0, 220.0, &live);
        c.line(480.0, 100.0, 480.0, 213.0, &dim);
        for bus in &self.buses {
            let col = if bus.live { Color::WHITE } else { Color::DIM };
            c.line(bus.x0, bus.y, bus.x1, bus.y, &Stroke::new(col, 3.0));
            c.draw_text(bus.x0, bus.y - 10.0, bus.name, &TextStyle::new(11.0, col));
            let v = format!("{:.1} V", bus.volts);
            c.draw_text(
                bus.x1,
                bus.y - 10.0,
                &v,
                &TextStyle::new(11.0, col).anchor(Anchor::End),
            );
        }
        c.line(400.0, 220.0, 473.0, 220.0, &dim);
        c.line(487.0, 220.0, 560.0, 220.0, &dim);
        for (x, label, on) in [(230.0, "TRU L", true), (730.0, "TRU R", false)] {
            let col = if on { Color::WHITE } else { Color::DIM };
            let s = Stroke::new(col, 1.2);
            c.line(x, 220.0, x, 273.0, &s);
            c.line(x, 287.0, x, 320.0, &s);
            Self::box_with_label(c, x - 40.0, 320.0, 80.0, 40.0, label, col);
            c.line(x, 360.0, x, 400.0, &s);
        }
        c.line(230.0, 400.0, 230.0, 463.0, &live);
        c.line(230.0, 477.0, 230.0, 500.0, &live);
        c.stroke_circle(230.0, 520.0, 18.0, &live);
        c.draw_text(
            230.0,
            524.0,
            "BAT",
            &TextStyle::new(11.0, Color::WHITE).anchor(Anchor::Middle),
        );
        c.draw_text(
            16.0,
            560.0,
            "white: live  grey: dead  red: tripped",
            &TextStyle::new(11.0, Color::FAINT),
        );
        for (i, b) in self.breakers.iter().enumerate() {
            Self::draw_breaker(c, b, i == self.selected);
        }
        c.restore();
    }

    fn click(&mut self, x: f32, y: f32) -> bool {
        for (i, b) in self.breakers.iter().enumerate() {
            if (x - b.x).abs() <= B + 4.0 && (y - b.y).abs() <= B + 4.0 {
                self.selected = i;
                return true;
            }
        }
        false
    }

    fn detail_title(&self) -> String {
        format!("Electrical > {}", self.breakers[self.selected].name)
    }

    fn detail(&self) -> Vec<DetailSection> {
        let b = &self.breakers[self.selected];
        let state = if b.tripped {
            Cell::fault("TRIPPED")
        } else if b.closed {
            Cell::new("closed")
        } else {
            Cell::new("open")
        };
        vec![
            DetailSection {
                title: String::from("Breaker"),
                rows: vec![
                    (String::from("device"), Cell::new(b.name)),
                    (String::from("feeds"), Cell::new(b.feeds)),
                    (String::from("state"), state),
                    (String::from("current"), Cell::new(format!("{:.1} A", b.amps))),
                ],
            },
            DetailSection {
                title: String::from("Failures"),
                rows: vec![(String::from("armed"), Cell::new("none"))],
            },
        ]
    }

    fn table(&self) -> Table {
        let mut t = Table {
            title: format!(
                "Feeders  {} of {} devices  {} tripped",
                self.breakers.len(),
                self.breakers.len(),
                self.tripped_count()
            ),
            columns: ["device", "feeds", "state", "current"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            rows: Vec::new(),
        };
        for b in &self.breakers {
            let state = if b.tripped {
                Cell::fault("tripped")
            } else if b.closed {
                Cell::new("closed")
            } else {
                Cell::new("open")
            };
            t.rows.push(vec![
                Cell::new(b.name),
                Cell::new(b.feeds),
                state,
                Cell::new(format!("{:.1} A", b.amps)),
            ]);
        }
        t
    }
}
