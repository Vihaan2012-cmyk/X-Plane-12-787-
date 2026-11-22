use super::data::{ChannelView, InspectorData};
use super::gfx::{amps, frame, put, put_end, put_mid};
use super::topo::{VH, VW};
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, Stroke};

pub struct BreakerPage {
    data: InspectorData,
    rpdu: String,
    channel: u32,
}

const TAU_S: f64 = 20.0;
const M_MIN: f64 = 0.1;
const M_MAX: f64 = 20.0;
const T_MIN: f64 = 0.001;
const T_MAX: f64 = 10000.0;

pub fn trip_time_s(m: f64, instant_m: f64) -> f64 {
    if m >= instant_m {
        return T_MIN * 10.0;
    }
    if m <= 1.0 {
        return T_MAX;
    }
    let t = TAU_S * (m * m / (m * m - 1.0)).ln();
    t.clamp(T_MIN * 10.0, T_MAX)
}

impl BreakerPage {
    pub fn new(data: InspectorData, rpdu: &str, channel: u32) -> BreakerPage {
        BreakerPage {
            data,
            rpdu: rpdu.to_string(),
            channel,
        }
    }

    pub fn set_data(&mut self, data: InspectorData) {
        self.data = data;
    }

    pub fn select(&mut self, rpdu: &str, channel: u32) {
        self.rpdu = rpdu.to_string();
        self.channel = channel;
    }

    fn view(&self) -> Option<&ChannelView> {
        self.data
            .rpdu(&self.rpdu)
            .and_then(|r| r.channels.iter().find(|c| c.channel == self.channel))
    }

    fn plot_x(m: f64, x0: f32, w: f32) -> f32 {
        let f = (m.clamp(M_MIN, M_MAX).ln() - M_MIN.ln()) / (M_MAX.ln() - M_MIN.ln());
        x0 + w * f as f32
    }

    fn plot_y(t: f64, y0: f32, h: f32) -> f32 {
        let f = (t.clamp(T_MIN, T_MAX).ln() - T_MIN.ln()) / (T_MAX.ln() - T_MIN.ln());
        y0 + h - h * f as f32
    }

    fn draw_curve(&self, c: &mut Canvas, ch: &ChannelView, x0: f32, y0: f32, w: f32, h: f32) {
        put(
            c,
            x0,
            y0 - 8.0,
            "Trip curve  I2t, time to trip against load multiple of rating",
            10.0,
            Color::WHITE,
        );
        c.fill_rect(x0, y0, w, h, Color::PANEL);
        frame(c, x0, y0, w, h, Color::LINE);
        let grid = Stroke::new(Color::LINE, 1.0);
        for m in [0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0] {
            let x = Self::plot_x(m, x0, w);
            c.line(x, y0, x, y0 + h, &grid);
            put_mid(c, x, y0 + h + 12.0, &format!("{}x", m), 10.0, Color::DIM);
        }
        for (t, l) in [
            (0.001, "1 ms"),
            (0.01, "10 ms"),
            (0.1, "100 ms"),
            (1.0, "1 s"),
            (10.0, "10 s"),
            (100.0, "100 s"),
            (1000.0, "1000 s"),
            (10000.0, "10000 s"),
        ] {
            let y = Self::plot_y(t, y0, h);
            c.line(x0, y, x0 + w, y, &grid);
            put_end(c, x0 - 4.0, y + 3.0, l, 10.0, Color::DIM);
        }
        let instant_m = if ch.rating_a > 0.0 {
            ch.instant_trip_a / ch.rating_a
        } else {
            8.0
        };
        let mut pts: Vec<(f32, f32)> = Vec::new();
        let mut m = 1.02f64;
        while m < instant_m {
            pts.push((
                Self::plot_x(m, x0, w),
                Self::plot_y(trip_time_s(m, instant_m), y0, h),
            ));
            m *= 1.04;
        }
        pts.push((Self::plot_x(instant_m, x0, w), Self::plot_y(0.01, y0, h)));
        pts.push((Self::plot_x(M_MAX, x0, w), Self::plot_y(0.01, y0, h)));
        c.polyline(&pts, &Stroke::new(Color::WHITE, 1.6));
        put(
            c,
            Self::plot_x(2.0, x0, w),
            Self::plot_y(6.0, y0, h) - 6.0,
            "thermal I2t",
            10.0,
            Color::DIM,
        );
        put(
            c,
            Self::plot_x(instant_m, x0, w) + 4.0,
            Self::plot_y(0.01, y0, h) - 4.0,
            "instant",
            10.0,
            Color::DIM,
        );
        let m_now = if ch.rating_a > 0.0 {
            ch.current_a / ch.rating_a
        } else {
            0.0
        };
        let xn = Self::plot_x(m_now, x0, w);
        let col = if ch.state.is_fault() {
            Color::BAD
        } else {
            Color::WHITE
        };
        c.line(xn, y0, xn, y0 + h, &Stroke::new(col, 1.0).dashed(&[4.0, 3.0]));
        put(c, xn + 4.0, y0 + 12.0, &format!("now {:.2}x", m_now), 10.0, col);
    }

    fn draw_trace(&self, c: &mut Canvas, ch: &ChannelView, x0: f32, y0: f32, w: f32, h: f32) {
        put(c, x0, y0 - 8.0, "Channel current  last 120 s", 10.0, Color::WHITE);
        c.fill_rect(x0, y0, w, h, Color::PANEL);
        frame(c, x0, y0, w, h, Color::LINE);
        let top = (ch.rating_a as f32).max(1.0);
        let grid = Stroke::new(Color::LINE, 1.0);
        for f in [0.0f32, 0.5, 1.0] {
            let y = y0 + h - h * f;
            c.line(x0, y, x0 + w, y, &grid);
            put_end(
                c,
                x0 - 4.0,
                y + 3.0,
                &format!("{:.1} A", top * f),
                10.0,
                Color::DIM,
            );
        }
        let n = ch.trace.len();
        if n > 1 {
            let pts: Vec<(f32, f32)> = ch
                .trace
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    (
                        x0 + w * i as f32 / (n - 1) as f32,
                        y0 + h - h * (a / top).clamp(0.0, 1.0),
                    )
                })
                .collect();
            c.polyline(&pts, &Stroke::new(Color::WHITE, 1.4));
        }
        put(c, x0 + 2.0, y0 + h + 12.0, "-120 s", 10.0, Color::DIM);
        put_end(c, x0 + w, y0 + h + 12.0, "now", 10.0, Color::DIM);
    }
}

impl Page for BreakerPage {
    fn id(&self) -> &str {
        "electrical.breaker"
    }

    fn title(&self) -> &str {
        "Electrical breaker"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        c.save();
        c.translate(area.x, area.y);
        c.scale(s, s);
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        let Some(ch) = self.view() else {
            put(c, 20.0, 40.0, "no breaker selected", 12.0, Color::DIM);
            c.restore();
            return;
        };
        let fault = ch.state.is_fault();
        let col = if fault { Color::BAD } else { Color::WHITE };
        put(
            c,
            20.0,
            30.0,
            &format!("{} ch {:02}", self.rpdu, ch.channel),
            14.0,
            Color::WHITE,
        );
        put(
            c,
            220.0,
            30.0,
            &format!("SSPC {} A  {}", ch.rating_a, ch.load),
            14.0,
            Color::WHITE,
        );
        put(
            c,
            20.0,
            54.0,
            &format!(
                "{}  {}  {:.1} V out",
                ch.state.label(),
                amps(ch.current_a),
                ch.voltage_out_v
            ),
            11.0,
            col,
        );
        let chain = ["RPDU input", "SSPC", "wire", "load"];
        for (i, n) in chain.iter().enumerate() {
            let x = 20.0 + i as f32 * 160.0;
            let live = ch.state == super::data::ChannelState::On;
            let k = if i == 1 {
                col
            } else if live {
                Color::WHITE
            } else {
                Color::FAINT
            };
            frame(c, x, 70.0, 110.0, 24.0, k);
            put_mid(c, x + 55.0, 86.0, n, 10.0, k);
            if i < 3 {
                c.line(x + 110.0, 82.0, x + 160.0, 82.0, &Stroke::new(k, 1.4));
            }
        }
        self.draw_curve(c, ch, 80.0, 140.0, 420.0, 400.0);
        self.draw_trace(c, ch, 580.0, 140.0, 390.0, 400.0);
        put(
            c,
            20.0,
            600.0,
            &format!(
                "wire {}  bundle {}  wire temp {:.0} C",
                ch.wire, ch.bundle, ch.wire_temp_c
            ),
            10.0,
            Color::DIM,
        );
        c.restore();
    }

    fn click(&mut self, _x: f32, _y: f32) -> bool {
        false
    }

    fn detail_title(&self) -> String {
        match self.view() {
            Some(ch) => format!("{} ch {:02}  {}", self.rpdu, ch.channel, ch.load),
            None => "Breaker".to_string(),
        }
    }

    fn detail(&self) -> Vec<DetailSection> {
        let Some(ch) = self.view() else {
            return Vec::new();
        };
        let st = if ch.state.is_fault() {
            Cell::fault(ch.state.label())
        } else {
            Cell::new(ch.state.label())
        };
        vec![
            DetailSection {
                title: "Telemetry".to_string(),
                rows: vec![
                    ("state".to_string(), st),
                    ("current".to_string(), Cell::new(amps(ch.current_a))),
                    (
                        "voltage out".to_string(),
                        Cell::new(format!("{:.1} V", ch.voltage_out_v)),
                    ),
                    (
                        "i2t accumulated".to_string(),
                        Cell::new(format!("{:.0} % of trip", ch.i2t_pct)),
                    ),
                    (
                        "arc-fault events".to_string(),
                        if ch.arc_events > 0 {
                            Cell::fault(ch.arc_events.to_string())
                        } else {
                            Cell::new("0")
                        },
                    ),
                    ("instant trip".to_string(), Cell::new(amps(ch.instant_trip_a))),
                    ("SSPC temp".to_string(), Cell::new(format!("{:.0} C", ch.temp_c))),
                    ("trips (life)".to_string(), Cell::new(ch.trips_life.to_string())),
                ],
            },
            DetailSection {
                title: "Wire and load".to_string(),
                rows: vec![
                    ("wire".to_string(), Cell::new(ch.wire.clone())),
                    ("bundle".to_string(), Cell::new(ch.bundle.clone())),
                    (
                        "wire temp".to_string(),
                        Cell::new(format!("{:.0} C", ch.wire_temp_c)),
                    ),
                ],
            },
        ]
    }

    fn table(&self) -> Table {
        let mut t = Table {
            title: "History".to_string(),
            columns: vec!["sim time s".to_string(), "event".to_string()],
            rows: Vec::new(),
        };
        if let Some(ch) = self.view() {
            for (time, text) in &ch.history {
                t.rows
                    .push(vec![Cell::new(format!("{:.1}", time)), Cell::new(text.clone())]);
            }
        }
        t
    }
}
