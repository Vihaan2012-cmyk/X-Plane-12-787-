use super::data::{EicasLevel, InspectorData};
use super::gfx::{amps, frame, put, put_mid, symbol, tone, volts, wire_stroke};
use super::label::{Box2, Placer, along, around};
use super::topo::{BARS, BOXES, BoxDef, CONTACTORS, VH, VW, WIRES};
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, Stroke, ellipsize, fit_size, measure_text};

const LABEL: f32 = 10.0;
use std::cell::Cell as StdCell;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    None,
    Contactor(String),
    Bus(String),
}

pub struct SingleLinePage {
    data: InspectorData,
    selection: Selection,
    scale: StdCell<f32>,
}

impl SingleLinePage {
    pub fn new(data: InspectorData) -> SingleLinePage {
        SingleLinePage {
            data,
            selection: Selection::None,
            scale: StdCell::new(1.0),
        }
    }

    pub fn set_data(&mut self, data: InspectorData) {
        self.data = data;
    }

    pub fn data(&self) -> &InspectorData {
        &self.data
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn select_contactor(&mut self, name: &str) {
        self.selection = Selection::Contactor(name.to_string());
    }

    fn node_live(&self, id: &str) -> bool {
        if let Some(b) = self.data.bus(id) {
            return b.live;
        }
        if let Some(s) = self.data.source(id) {
            return s.online;
        }
        if let Some(s) = self.data.converter(id) {
            return s.live;
        }
        if let Some(s) = self.data.battery(id) {
            return s.voltage_v > 1.0;
        }
        false
    }

    fn node_fault(&self, id: &str) -> bool {
        self.data.bus(id).is_some_and(|b| b.fault)
            || self.data.source(id).is_some_and(|b| b.fault)
            || self.data.converter(id).is_some_and(|b| b.fault)
            || self.data.battery(id).is_some_and(|b| b.fault)
    }

    fn box_lines(&self, id: &str, label: &str) -> (String, String) {
        if let Some(s) = self.data.source(id) {
            if s.online {
                if s.freq_hz > 0.0 {
                    return (
                        label.to_string(),
                        format!("{} {:.0}Hz", volts(s.voltage_v), s.freq_hz),
                    );
                }
                return (label.to_string(), volts(s.voltage_v));
            }
            return (label.to_string(), "off".to_string());
        }
        if let Some(s) = self.data.converter(id) {
            if s.live {
                if s.power_kw >= 1.0 {
                    return (
                        label.to_string(),
                        format!("{} {:.1}kW", volts(s.output_v), s.power_kw),
                    );
                }
                return (label.to_string(), volts(s.output_v));
            }
            return (label.to_string(), "off".to_string());
        }
        if let Some(b) = self.data.battery(id) {
            return (
                label.to_string(),
                format!("{} {:+.1}A", volts(b.voltage_v), b.current_a),
            );
        }
        (label.to_string(), "-".to_string())
    }

    fn bus_texts(&self, id: &str) -> Vec<String> {
        match self.data.bus(id) {
            Some(v) if v.live && v.freq_hz > 0.0 => vec![
                format!("{} {} {:.0}Hz", id, volts(v.voltage_v), v.freq_hz),
                format!("{} {}", id, volts(v.voltage_v)),
                id.to_string(),
            ],
            Some(v) if v.live => vec![format!("{} {}", id, volts(v.voltage_v)), id.to_string()],
            _ => vec![format!("{} dead", id), id.to_string()],
        }
    }

    fn draw_box_text(&self, c: &mut Canvas, b: &BoxDef, live: bool) {
        let (l1, l2) = self.box_lines(b.id, b.label);
        let room = b.w - 4.0;
        let dim = if live { Color::WHITE } else { Color::DIM };
        if b.h < 24.0 {
            let s = fit_size(&l1, 10.0, 8.0, room);
            let t = ellipsize(&l1, s, room);
            put_mid(c, b.x + b.w / 2.0, b.y + b.h / 2.0 + 0.25 * s, &t, s, dim);
        } else {
            let s1 = fit_size(&l1, 10.0, 8.0, room);
            put_mid(
                c,
                b.x + b.w / 2.0,
                b.y + 12.0,
                &ellipsize(&l1, s1, room),
                s1,
                Color::WHITE,
            );
            let s2 = fit_size(&l2, 10.0, 8.0, room);
            put_mid(
                c,
                b.x + b.w / 2.0,
                b.y + b.h - 5.0,
                &ellipsize(&l2, s2, room),
                s2,
                dim,
            );
        }
    }

    fn draw_virtual(&self, c: &mut Canvas) {
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        let mut placer = Placer::new();
        for w in WIRES.iter() {
            let live = self.node_live(w.live);
            c.polyline(w.pts, &wire_stroke(live, false));
            placer.wire(w.pts);
        }
        for b in BARS.iter() {
            let live = self.node_live(b.id);
            let fault = self.node_fault(b.id);
            let sel = self.selection == Selection::Bus(b.id.to_string());
            c.fill_rect(b.x0, b.y - 2.0, b.x1 - b.x0, 4.0, tone(live, fault));
            if sel {
                frame(c, b.x0 - 3.0, b.y - 6.0, b.x1 - b.x0 + 6.0, 12.0, Color::WHITE);
            }
            placer.block(Box2::new(b.x0 - 3.0, b.y - 4.0, b.x1 + 3.0, b.y + 4.0));
        }
        for b in BOXES.iter() {
            let live = self.node_live(b.id);
            let fault = self.node_fault(b.id);
            c.fill_rect(b.x, b.y, b.w, b.h, Color::PANEL);
            frame(c, b.x, b.y, b.w, b.h, tone(live, fault));
            self.draw_box_text(c, b, live);
            placer.block(Box2::new(b.x - 2.0, b.y - 2.0, b.x + b.w + 2.0, b.y + b.h + 2.0));
        }
        for k in CONTACTORS.iter() {
            let cv = self.data.contactor(k.name);
            let closed = cv.is_some_and(|c| c.closed);
            let fault = cv.is_some_and(|c| c.fault);
            let live = self.node_live(k.a) && self.node_live(k.b);
            let sel = self.selection == Selection::Contactor(k.name.to_string());
            if sel {
                frame(c, k.x - 8.0, k.y - 8.0, 16.0, 16.0, Color::WHITE);
            }
            symbol(c, k.x, k.y, closed, live, fault);
            placer.block(Box2::new(k.x - 7.0, k.y - 7.0, k.x + 7.0, k.y + 7.0));
        }
        let (ex, ey, ew, eh) = (20.0, 560.0, 290.0, 90.0);
        placer.block(Box2::new(ex - 2.0, ey - 2.0, ex + ew + 2.0, ey + eh + 2.0));
        let legend = "white live  grey dashed dead  red fault  filled closed  hollow open";
        put(c, 10.0, 666.0, legend, 10.0, Color::FAINT);
        let lw = measure_text(legend, 10.0).0;
        placer.block(Box2::text(10.0, 666.0, lw, 10.0));
        let mut order: Vec<usize> = (0..CONTACTORS.len()).collect();
        order.sort_by_key(|&i| {
            let k = &CONTACTORS[i];
            let sel = self.selection == Selection::Contactor(k.name.to_string());
            let fault = self.data.contactor(k.name).is_some_and(|c| c.fault);
            (
                if sel {
                    0
                } else if fault {
                    1
                } else {
                    2
                },
                i,
            )
        });
        for i in order {
            let k = &CONTACTORS[i];
            let fault = self.data.contactor(k.name).is_some_and(|c| c.fault);
            let sel = self.selection == Selection::Contactor(k.name.to_string());
            let col = if fault {
                Color::BAD
            } else if sel {
                Color::WHITE
            } else {
                Color::DIM
            };
            let w = measure_text(k.name, LABEL).0;
            let cands = around(k.x, k.y, w, LABEL, 8.0);
            if let Some((s, lead)) = placer.place_led(k.name, LABEL, (k.x, k.y), &cands) {
                if let Some(p) = lead {
                    let (dx, dy) = (p.0 - k.x, p.1 - k.y);
                    let len = (dx * dx + dy * dy).sqrt().max(0.01);
                    let st = (k.x + dx / len * 6.0, k.y + dy / len * 6.0);
                    c.line(st.0, st.1, p.0, p.1, &Stroke::new(Color::FAINT, 0.8));
                }
                put(c, s.x, s.y, k.name, LABEL, col);
            } else if sel || fault {
                let s = cands[2];
                c.fill_rect(s.x - 2.0, s.y - 9.0, w + 4.0, 12.0, Color::BG);
                put(c, s.x, s.y, k.name, LABEL, col);
            }
        }
        for b in BARS.iter() {
            let live = self.node_live(b.id);
            let col = if live { Color::WHITE } else { Color::DIM };
            for text in self.bus_texts(b.id) {
                let w = measure_text(&text, LABEL).0;
                let cands = along(b.x0, b.x1, b.y, w, LABEL, b.unit == "HV");
                if let Some(s) = placer.place(&text, LABEL, &cands) {
                    put(c, s.x, s.y, &text, LABEL, col);
                    break;
                }
            }
        }
        self.draw_eicas(c, ex, ey, ew);
    }

    fn draw_eicas(&self, c: &mut Canvas, x: f32, y: f32, w: f32) {
        frame(c, x, y, w, 90.0, Color::LINE);
        put(c, x + 6.0, y + 13.0, "EICAS", 10.0, Color::DIM);
        let room = w - 12.0;
        if self.data.eicas.is_empty() {
            put(c, x + 6.0, y + 29.0, "no electrical messages", 10.0, Color::FAINT);
        }
        for (i, m) in self.data.eicas.iter().take(6).enumerate() {
            let col = match m.level {
                EicasLevel::Warning => Color::BAD,
                EicasLevel::Caution => Color::WHITE,
                _ => Color::DIM,
            };
            put(
                c,
                x + 6.0,
                y + 29.0 + i as f32 * 11.0,
                &ellipsize(&m.text, 10.0, room),
                10.0,
                col,
            );
        }
    }

    fn hit(&self, vx: f32, vy: f32) -> Selection {
        let mut best: Option<(f32, &str)> = None;
        for k in CONTACTORS.iter() {
            let d = (k.x - vx).abs().max((k.y - vy).abs());
            if d <= 9.0 && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, k.name));
            }
        }
        if let Some((_, n)) = best {
            return Selection::Contactor(n.to_string());
        }
        for b in BARS.iter() {
            if vx >= b.x0 - 3.0 && vx <= b.x1 + 3.0 && (vy - b.y).abs() <= 6.0 {
                return Selection::Bus(b.id.to_string());
            }
        }
        Selection::None
    }

    fn contactor_row(&self, name: &str, a: &str, b: &str, panel: &str) -> Vec<Cell> {
        let cv = self.data.contactor(name);
        let closed = cv.is_some_and(|c| c.closed);
        let fault = cv.is_some_and(|c| c.fault);
        let state = if fault {
            Cell::fault(cv.map_or(String::new(), |c| {
                if c.fault_text.is_empty() {
                    "fault".to_string()
                } else {
                    c.fault_text.clone()
                }
            }))
        } else if closed {
            Cell::new("closed")
        } else {
            Cell::new("open")
        };
        let va = self.data.bus(a).map_or(0.0, |x| x.voltage_v);
        let vb = self.data.bus(b).map_or(0.0, |x| x.voltage_v);
        vec![
            Cell::new(name),
            Cell::new(a),
            Cell::new(b),
            state,
            Cell::new(volts(va)),
            Cell::new(volts(vb)),
            Cell::new(amps(cv.map_or(0.0, |c| c.current_a))),
            Cell::new(panel),
        ]
    }
}

impl Page for SingleLinePage {
    fn id(&self) -> &str {
        "electrical"
    }

    fn title(&self) -> &str {
        "Electrical"
    }

    fn draw(&self, canvas: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        self.scale.set(s);
        canvas.save();
        canvas.translate(area.x, area.y);
        canvas.scale(s, s);
        self.draw_virtual(canvas);
        canvas.restore();
    }

    fn click(&mut self, x: f32, y: f32) -> bool {
        let s = self.scale.get().max(0.01);
        let sel = self.hit(x / s, y / s);
        let changed = sel != self.selection;
        self.selection = sel;
        changed
    }

    fn detail_title(&self) -> String {
        match &self.selection {
            Selection::None => "Electrical power".to_string(),
            Selection::Contactor(n) => format!("Contactor {n}"),
            Selection::Bus(n) => format!("Bus {n}"),
        }
    }

    fn detail(&self) -> Vec<DetailSection> {
        match &self.selection {
            Selection::None => {
                let live = self.data.buses.iter().filter(|b| b.live).count();
                let closed = self.data.contactors.iter().filter(|c| c.closed).count();
                let faults = self.data.contactors.iter().filter(|c| c.fault).count();
                vec![DetailSection {
                    title: "Summary".to_string(),
                    rows: vec![
                        (
                            "buses live".to_string(),
                            Cell::new(format!("{} of {}", live, self.data.buses.len())),
                        ),
                        (
                            "contactors closed".to_string(),
                            Cell::new(format!("{} of {}", closed, self.data.contactors.len())),
                        ),
                        (
                            "contactor faults".to_string(),
                            if faults > 0 {
                                Cell::fault(faults.to_string())
                            } else {
                                Cell::new("0")
                            },
                        ),
                        (
                            "messages".to_string(),
                            Cell::new(self.data.eicas.len().to_string()),
                        ),
                    ],
                }]
            }
            Selection::Contactor(n) => {
                let def = CONTACTORS.iter().find(|k| k.name == n);
                let cv = self.data.contactor(n);
                let (a, b, p) = def.map_or(("-", "-", "-"), |k| (k.a, k.b, k.panel));
                let state = match cv {
                    Some(c) if c.fault => Cell::fault(if c.fault_text.is_empty() {
                        "fault"
                    } else {
                        &c.fault_text
                    }),
                    Some(c) if c.closed => Cell::new("closed"),
                    Some(_) => Cell::new("open"),
                    None => Cell::new("no data"),
                };
                let side = |id: &str| match self.data.bus(id) {
                    Some(v) => format!("{} {}", volts(v.voltage_v), if v.live { "live" } else { "dead" }),
                    None => "-".to_string(),
                };
                vec![
                    DetailSection {
                        title: "Contactor".to_string(),
                        rows: vec![
                            ("state".to_string(), state),
                            (
                                "current".to_string(),
                                Cell::new(amps(cv.map_or(0.0, |c| c.current_a))),
                            ),
                            ("panel".to_string(), Cell::new(p)),
                        ],
                    },
                    DetailSection {
                        title: "Connects".to_string(),
                        rows: vec![
                            (a.to_string(), Cell::new(side(a))),
                            (b.to_string(), Cell::new(side(b))),
                        ],
                    },
                ]
            }
            Selection::Bus(n) => {
                let bv = self.data.bus(n);
                let feeders: Vec<(String, Cell)> = CONTACTORS
                    .iter()
                    .filter(|k| k.a == n || k.b == n)
                    .map(|k| {
                        let cv = self.data.contactor(k.name);
                        let t = format!(
                            "{} {}",
                            if cv.is_some_and(|c| c.closed) {
                                "closed"
                            } else {
                                "open"
                            },
                            amps(cv.map_or(0.0, |c| c.current_a))
                        );
                        (
                            k.name.to_string(),
                            if cv.is_some_and(|c| c.fault) {
                                Cell::fault(t)
                            } else {
                                Cell::new(t)
                            },
                        )
                    })
                    .collect();
                vec![
                    DetailSection {
                        title: "Bus".to_string(),
                        rows: vec![
                            (
                                "voltage".to_string(),
                                Cell::new(volts(bv.map_or(0.0, |b| b.voltage_v))),
                            ),
                            (
                                "frequency".to_string(),
                                Cell::new(format!("{:.0} Hz", bv.map_or(0.0, |b| b.freq_hz))),
                            ),
                            ("load".to_string(), Cell::new(amps(bv.map_or(0.0, |b| b.load_a)))),
                            (
                                "state".to_string(),
                                if bv.is_some_and(|b| b.fault) {
                                    Cell::fault("fault")
                                } else if bv.is_some_and(|b| b.live) {
                                    Cell::new("live")
                                } else {
                                    Cell::new("dead")
                                },
                            ),
                        ],
                    },
                    DetailSection {
                        title: "Feeders".to_string(),
                        rows: feeders,
                    },
                ]
            }
        }
    }

    fn table(&self) -> Table {
        let sel = match &self.selection {
            Selection::Contactor(n) => n.clone(),
            _ => String::new(),
        };
        let mut idx: Vec<usize> = (0..CONTACTORS.len()).collect();
        idx.sort_by_key(|&i| {
            let k = &CONTACTORS[i];
            let cv = self.data.contactor(k.name);
            let rank = if k.name == sel {
                0
            } else if cv.is_some_and(|c| c.fault) {
                1
            } else if cv.is_some_and(|c| c.closed) {
                2
            } else {
                3
            };
            (rank, i)
        });
        Table {
            title: "Feeders".to_string(),
            columns: ["device", "from", "to", "state", "V from", "V to", "A", "panel"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            rows: idx
                .into_iter()
                .map(|i| {
                    let k = &CONTACTORS[i];
                    self.contactor_row(k.name, k.a, k.b, k.panel)
                })
                .collect(),
        }
    }
}
