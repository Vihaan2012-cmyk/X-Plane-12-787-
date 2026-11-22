use super::data::{ChannelState, InspectorData};
use super::gfx::{amps, frame, put, tone};
use super::topo::{VH, VW};
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, ellipsize, fit_size};
use std::cell::Cell as StdCell;

pub struct RpduPage {
    data: InspectorData,
    rpdu: usize,
    channel: usize,
    scale: StdCell<f32>,
}

const COLS: usize = 8;
const TW: f32 = 119.0;
const TH: f32 = 62.0;
const TOP: f32 = 64.0;

impl RpduPage {
    pub fn new(data: InspectorData) -> RpduPage {
        RpduPage {
            data,
            rpdu: 0,
            channel: 0,
            scale: StdCell::new(1.0),
        }
    }

    pub fn set_data(&mut self, data: InspectorData) {
        self.data = data;
    }

    pub fn rpdu_id(&self) -> String {
        self.data
            .rpdus
            .get(self.rpdu)
            .map_or(String::new(), |r| r.id.clone())
    }

    pub fn selected_channel(&self) -> Option<u32> {
        self.data
            .rpdus
            .get(self.rpdu)
            .and_then(|r| r.channels.get(self.channel))
            .map(|c| c.channel)
    }

    pub fn select_rpdu(&mut self, id: &str) -> bool {
        match self.data.rpdus.iter().position(|r| r.id == id) {
            Some(i) => {
                self.rpdu = i;
                self.channel = 0;
                true
            }
            None => false,
        }
    }

    pub fn select_channel(&mut self, channel: u32) -> bool {
        let Some(r) = self.data.rpdus.get(self.rpdu) else {
            return false;
        };
        match r.channels.iter().position(|c| c.channel == channel) {
            Some(i) => {
                self.channel = i;
                true
            }
            None => false,
        }
    }

    fn tile(i: usize) -> (f32, f32) {
        (
            10.0 + (i % COLS) as f32 * (TW + 5.0),
            TOP + (i / COLS) as f32 * (TH + 5.0),
        )
    }

    fn picker(i: usize) -> (f32, f32, f32) {
        (10.0 + i as f32 * 80.0, 8.0, 74.0)
    }
}

impl Page for RpduPage {
    fn id(&self) -> &str {
        "electrical.rpdu"
    }

    fn title(&self) -> &str {
        "Electrical RPDU"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        self.scale.set(s);
        c.save();
        c.translate(area.x, area.y);
        c.scale(s, s);
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        for (i, r) in self.data.rpdus.iter().enumerate() {
            let (x, y, w) = Self::picker(i);
            let sel = i == self.rpdu;
            frame(c, x, y, w, 18.0, if sel { Color::WHITE } else { Color::LINE });
            put(
                c,
                x + 5.0,
                y + 13.0,
                &r.id,
                10.0,
                tone(r.powered, r.tripped() > 0),
            );
        }
        if let Some(r) = self.data.rpdus.get(self.rpdu) {
            let head = format!(
                "{} ({})  {} channels  {} on  {} off  {} shed  {} tripped  input {:.1} V  {:.1} A",
                r.id,
                r.location,
                r.channels.len(),
                r.count(ChannelState::On),
                r.count(ChannelState::Off),
                r.count(ChannelState::Shed),
                r.tripped(),
                r.input_v,
                r.input_a
            );
            put(
                c,
                10.0,
                46.0,
                &ellipsize(&head, 10.0, VW - 20.0),
                10.0,
                Color::DIM,
            );
            for (i, ch) in r.channels.iter().enumerate() {
                let (x, y) = Self::tile(i);
                let fault = ch.state.is_fault();
                let sel = i == self.channel;
                let border = if fault {
                    Color::BAD
                } else if sel {
                    Color::WHITE
                } else {
                    Color::LINE
                };
                c.fill_rect(x, y, TW, TH, if sel { Color::PANEL2 } else { Color::PANEL });
                frame(c, x, y, TW, TH, border);
                let live = ch.state == ChannelState::On;
                let name_col = if fault {
                    Color::BAD
                } else if live {
                    Color::WHITE
                } else {
                    Color::DIM
                };
                put(
                    c,
                    x + 6.0,
                    y + 13.0,
                    &format!("{:02} {}A", ch.channel, ch.rating_a),
                    10.0,
                    name_col,
                );
                let room = TW - 12.0;
                let ls = fit_size(&ch.load, 10.0, 9.0, room);
                put(c, x + 6.0, y + 31.0, &ellipsize(&ch.load, ls, room), ls, name_col);
                let state = if live {
                    format!("on  {}", amps(ch.current_a))
                } else {
                    ch.state.label().to_string()
                };
                put(
                    c,
                    x + 6.0,
                    y + 50.0,
                    &state,
                    10.0,
                    if fault { Color::BAD } else { Color::DIM },
                );
            }
        } else {
            put(c, 10.0, 46.0, "no RPDU data", 10.0, Color::DIM);
        }
        c.restore();
    }

    fn click(&mut self, x: f32, y: f32) -> bool {
        let s = self.scale.get().max(0.01);
        let (vx, vy) = (x / s, y / s);
        for i in 0..self.data.rpdus.len() {
            let (px, py, pw) = Self::picker(i);
            if vx >= px && vx <= px + pw && vy >= py && vy <= py + 18.0 {
                let changed = self.rpdu != i;
                self.rpdu = i;
                self.channel = 0;
                return changed;
            }
        }
        let n = self.data.rpdus.get(self.rpdu).map_or(0, |r| r.channels.len());
        for i in 0..n {
            let (tx, ty) = Self::tile(i);
            if vx >= tx && vx <= tx + TW && vy >= ty && vy <= ty + TH {
                let changed = self.channel != i;
                self.channel = i;
                return changed;
            }
        }
        false
    }

    fn detail_title(&self) -> String {
        match self.selected_channel() {
            Some(ch) => format!("{} ch {:02}", self.rpdu_id(), ch),
            None => "RPDU".to_string(),
        }
    }

    fn detail(&self) -> Vec<DetailSection> {
        let Some(ch) = self
            .data
            .rpdus
            .get(self.rpdu)
            .and_then(|r| r.channels.get(self.channel))
        else {
            return Vec::new();
        };
        vec![DetailSection {
            title: format!("SSPC {} A  {}", ch.rating_a, ch.load),
            rows: vec![
                (
                    "state".to_string(),
                    if ch.state.is_fault() {
                        Cell::fault(ch.state.label())
                    } else {
                        Cell::new(ch.state.label())
                    },
                ),
                ("current".to_string(), Cell::new(amps(ch.current_a))),
                (
                    "i2t".to_string(),
                    Cell::new(format!("{:.0} % of trip", ch.i2t_pct)),
                ),
                ("temp".to_string(), Cell::new(format!("{:.0} C", ch.temp_c))),
            ],
        }]
    }

    fn table(&self) -> Table {
        let mut t = Table {
            title: format!("{} channels", self.rpdu_id()),
            columns: ["ch", "rating", "load", "state", "A", "i2t %"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            rows: Vec::new(),
        };
        if let Some(r) = self.data.rpdus.get(self.rpdu) {
            let mut v: Vec<_> = r.channels.iter().collect();
            v.sort_by_key(|c| (!c.state.is_fault(), c.state != ChannelState::Shed, c.channel));
            for c in v {
                let st = if c.state.is_fault() {
                    Cell::fault(c.state.label())
                } else {
                    Cell::new(c.state.label())
                };
                t.rows.push(vec![
                    Cell::new(format!("{:02}", c.channel)),
                    Cell::new(format!("{} A", c.rating_a)),
                    Cell::new(c.load.clone()),
                    st,
                    Cell::new(amps(c.current_a)),
                    Cell::new(format!("{:.0}", c.i2t_pct)),
                ]);
            }
        }
        t
    }
}
