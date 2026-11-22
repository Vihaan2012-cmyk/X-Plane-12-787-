use super::data::{EicasLevel, InspectorData};
use super::gfx::{frame, put, put_end, put_mid};
use super::topo::{VH, VW};
use super::ui::{AMBER, CYAN, Outbox, PanelAction, button, heading};
use crate::control::cbic::{BreakerKind, CB_STATUS_PAGE_SIZE, CbState, LockTag};
use crate::system::Electrical;
use render::page::{Cell, DetailSection, Page, Table};
use render::{Canvas, Color, Rect, Stroke, ellipsize};
use std::cell::Cell as StdCell;

const LX: f32 = 10.0;
const LW: f32 = 420.0;
const RX: f32 = 452.0;
const RW: f32 = 538.0;
const ROW_Y: f32 = 54.0;
const ROW_H: f32 = 24.0;
const EICAS_ROWS: usize = 32;
const BTN_Y: f32 = 462.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CbCommand {
    Open,
    Close,
    Lock,
    Unlock,
    Reset,
}

impl CbCommand {
    const ALL: [CbCommand; 5] = [
        CbCommand::Open,
        CbCommand::Close,
        CbCommand::Lock,
        CbCommand::Unlock,
        CbCommand::Reset,
    ];

    pub fn action(self) -> &'static str {
        match self {
            CbCommand::Open => "cbic_open",
            CbCommand::Close => "cbic_close",
            CbCommand::Lock => "cbic_lock",
            CbCommand::Unlock => "cbic_unlock",
            CbCommand::Reset => "cbic_reset",
        }
    }

    fn label(self) -> &'static str {
        match self {
            CbCommand::Open => "OPEN",
            CbCommand::Close => "CLOSE",
            CbCommand::Lock => "LOCK",
            CbCommand::Unlock => "UNLOCK",
            CbCommand::Reset => "RESET",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CbRow {
    pub id: u32,
    pub name: String,
    pub kind: BreakerKind,
    pub state: CbState,
    pub load_a: f64,
}

impl CbRow {
    pub fn target(&self) -> String {
        let p = match self.kind {
            BreakerKind::Tcb => "CT",
            BreakerKind::Elcc => "CK",
            BreakerKind::Ecb | BreakerKind::FcSsr => "CE",
        };
        format!("{p}{}", self.id)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CbicView {
    pub mode: String,
    pub rows: Vec<CbRow>,
}

impl CbicView {
    pub fn from_system(sys: &Electrical) -> CbicView {
        let app = &sys.cbic().app;
        let ids = app.search("").unwrap_or_default();
        CbicView {
            mode: format!("{:?}", app.mode()).to_uppercase(),
            rows: ids
                .into_iter()
                .filter_map(|id| app.breaker(id))
                .map(|b| CbRow {
                    id: b.id,
                    name: b.name.clone(),
                    kind: b.kind,
                    state: b.state,
                    load_a: b.load_amps,
                })
                .collect(),
        }
    }

    pub fn demo() -> CbicView {
        let mk = |id: u32, name: &str, kind: BreakerKind, state: CbState, load_a: f64| CbRow {
            id,
            name: name.to_string(),
            kind,
            state,
            load_a,
        };
        CbicView {
            mode: String::from("GROUND"),
            rows: vec![
                mk(
                    1471831,
                    "MAIN DC FEED RPDU 71",
                    BreakerKind::Ecb,
                    CbState::Closed,
                    21.4,
                ),
                mk(
                    1471832,
                    "MAIN DC FEED RPDU 72",
                    BreakerKind::Ecb,
                    CbState::Closed,
                    18.0,
                ),
                mk(
                    2125831,
                    "FLT DECK BOOST FAN",
                    BreakerKind::Ecb,
                    CbState::Tripped,
                    0.0,
                ),
                mk(
                    2127802,
                    "SPLY FAN-FWD EQPT CLG L MC",
                    BreakerKind::Ecb,
                    CbState::Open,
                    0.0,
                ),
                mk(
                    2127803,
                    "SPLY FAN-FWD EQPT CLG R MC",
                    BreakerKind::Ecb,
                    CbState::Locked(LockTag::Inop),
                    0.0,
                ),
                mk(
                    2127804,
                    "GALLEY CHILLER 3",
                    BreakerKind::Ecb,
                    CbState::Locked(LockTag::DoNotClose),
                    0.0,
                ),
                mk(
                    2127805,
                    "CABIN TEMP SENSOR",
                    BreakerKind::Ecb,
                    CbState::Unknown,
                    0.0,
                ),
                mk(
                    3000101,
                    "STBY INSTR LIGHT",
                    BreakerKind::Tcb,
                    CbState::Closed,
                    0.4,
                ),
                mk(3000102, "CAPT PED OUTLET", BreakerKind::Tcb, CbState::Open, 0.0),
            ],
        }
    }
}

pub struct EicasPage {
    data: InspectorData,
    cbic: CbicView,
    selected: usize,
    notice: String,
    out: Outbox,
    scale: StdCell<f32>,
}

fn level_color(l: EicasLevel) -> Color {
    match l {
        EicasLevel::Warning => Color::BAD,
        EicasLevel::Caution | EicasLevel::Advisory => AMBER,
        EicasLevel::Status => Color::WHITE,
    }
}

fn level_tag(l: EicasLevel) -> &'static str {
    match l {
        EicasLevel::Warning => "WARNING",
        EicasLevel::Caution => "CAUTION",
        EicasLevel::Advisory => "ADVISORY",
        EicasLevel::Status => "STATUS",
    }
}

fn rank(l: EicasLevel) -> u8 {
    match l {
        EicasLevel::Warning => 0,
        EicasLevel::Caution => 1,
        EicasLevel::Advisory => 2,
        EicasLevel::Status => 3,
    }
}

fn state_view(s: CbState) -> (&'static str, Color) {
    match s {
        CbState::Closed => ("CLOSED", super::ui::GREEN),
        CbState::Open => ("OPEN", Color::WHITE),
        CbState::Tripped => ("TRIPPED", AMBER),
        CbState::Locked(LockTag::Inop) => ("OPEN INOP", CYAN),
        CbState::Locked(LockTag::DoNotClose) => ("DO NOT CLOSE", Color::BAD),
        CbState::Unknown => ("UNKNOWN", AMBER),
    }
}

fn row_rect(slot: usize) -> Rect {
    Rect::new(RX, ROW_Y + ROW_H * slot as f32, RW, ROW_H - 2.0)
}

fn btn_rect(i: usize) -> Rect {
    Rect::new(RX + 104.0 * i as f32, BTN_Y, 98.0, 28.0)
}

fn page_rect(next: bool) -> Rect {
    let x = if next { RX + RW - 70.0 } else { RX + RW - 144.0 };
    Rect::new(x, 24.0, 66.0, 22.0)
}

fn icon(c: &mut Canvas, x: f32, y: f32, s: CbState, kind: BreakerKind) {
    let (_, col) = state_view(s);
    match s {
        CbState::Closed => c.fill_rect(x, y, 14.0, 14.0, col),
        CbState::Open => {
            c.stroke_rect(x, y, 14.0, 14.0, &Stroke::new(col, 1.4));
            if kind == BreakerKind::Tcb {
                c.line(x + 1.0, y + 13.0, x + 13.0, y + 1.0, &Stroke::new(col, 1.4));
            }
        }
        CbState::Tripped => {
            c.fill_rect(x, y, 14.0, 14.0, col);
            c.fill_rect(x + 5.0, y + 3.0, 4.0, 8.0, Color::BG);
        }
        CbState::Locked(LockTag::Inop) => {
            c.stroke_rect(x - 1.0, y - 1.0, 16.0, 16.0, &Stroke::new(col, 2.0));
            c.fill_rect(x + 2.0, y + 5.0, 10.0, 4.0, super::ui::BLUE);
        }
        CbState::Locked(LockTag::DoNotClose) => {
            c.stroke_rect(x - 1.0, y - 1.0, 16.0, 16.0, &Stroke::new(CYAN, 2.0));
            c.fill_rect(x + 2.0, y + 5.0, 10.0, 4.0, col);
        }
        CbState::Unknown => {
            c.stroke_rect(x, y, 14.0, 14.0, &Stroke::new(col, 1.4));
            put_mid(c, x + 7.0, y + 11.0, "?", 11.0, col);
        }
    }
}

impl EicasPage {
    pub fn new(data: InspectorData) -> EicasPage {
        EicasPage {
            data,
            cbic: CbicView::demo(),
            selected: 0,
            notice: String::new(),
            out: Outbox::default(),
            scale: StdCell::new(1.0),
        }
    }

    pub fn set_data(&mut self, data: InspectorData) {
        self.data = data;
    }

    pub fn set_cbic(&mut self, cbic: CbicView) {
        self.cbic = cbic;
        if self.selected >= self.cbic.rows.len() {
            self.selected = self.cbic.rows.len().saturating_sub(1);
        }
    }

    pub fn take_actions(&mut self) -> Vec<PanelAction> {
        self.out.take()
    }

    pub fn pending(&self) -> &[PanelAction] {
        self.out.pending()
    }

    pub fn selected_cb(&self) -> Option<&CbRow> {
        self.cbic.rows.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.cbic.rows.len() {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_id(&mut self, id: u32) -> bool {
        match self.cbic.rows.iter().position(|r| r.id == id) {
            Some(i) => {
                self.selected = i;
                true
            }
            None => false,
        }
    }

    pub fn command(&mut self, cmd: CbCommand) -> bool {
        let Some(row) = self.cbic.rows.get(self.selected) else {
            self.notice = String::from("no breaker selected");
            return false;
        };
        if row.kind == BreakerKind::Tcb {
            self.notice = format!("{} is a TCB: indication only", row.target());
            return false;
        }
        let target = row.target();
        self.notice = format!("queued {} {}", cmd.label(), target);
        self.out
            .push(PanelAction::new("crew", cmd.action(), &target, None));
        true
    }

    pub fn button_center(&self, cmd: CbCommand) -> (f32, f32) {
        let i = CbCommand::ALL.iter().position(|c| *c == cmd).unwrap_or(0);
        let r = btn_rect(i);
        let s = self.scale.get();
        ((r.x + r.w / 2.0) * s, (r.y + r.h / 2.0) * s)
    }

    pub fn row_center(&self, index: usize) -> (f32, f32) {
        let slot = index % CB_STATUS_PAGE_SIZE;
        let r = row_rect(slot);
        let s = self.scale.get();
        ((r.x + r.w / 2.0) * s, (r.y + r.h / 2.0) * s)
    }

    fn first(&self) -> usize {
        self.selected / CB_STATUS_PAGE_SIZE * CB_STATUS_PAGE_SIZE
    }

    fn messages(&self) -> Vec<(EicasLevel, &str)> {
        let mut v: Vec<(EicasLevel, &str)> = self
            .data
            .eicas
            .iter()
            .map(|m| (m.level, m.text.as_str()))
            .collect();
        v.sort_by_key(|(l, _)| rank(*l));
        v
    }

    fn draw_eicas(&self, c: &mut Canvas) {
        let msgs = self.messages();
        heading(
            c,
            LX,
            28.0,
            LW,
            &format!("EICAS ELECTRICAL MESSAGES ({})", msgs.len()),
        );
        if msgs.is_empty() {
            put(c, LX, 54.0, "no messages", 11.0, Color::DIM);
        }
        for (i, (l, t)) in msgs.iter().take(EICAS_ROWS).enumerate() {
            let y = 54.0 + 18.0 * i as f32;
            put(c, LX, y, level_tag(*l), 10.0, Color::DIM);
            let col = level_color(*l);
            let indent = if matches!(l, EicasLevel::Advisory) {
                10.0
            } else {
                0.0
            };
            put(
                c,
                LX + 84.0 + indent,
                y,
                &ellipsize(t, 12.0, LW - 84.0 - indent),
                12.0,
                col,
            );
        }
        if msgs.len() > EICAS_ROWS {
            let more = format!("+{} more", msgs.len() - EICAS_ROWS);
            put_end(c, LX + LW, 28.0, &more, 10.0, Color::DIM);
        }
    }

    fn draw_cbic(&self, c: &mut Canvas) {
        let rows = &self.cbic.rows;
        let first = self.first();
        let pages = rows.len().div_ceil(CB_STATUS_PAGE_SIZE).max(1);
        heading(
            c,
            RX,
            28.0,
            RW - 160.0,
            &format!(
                "CBIC CB STATUS  {}  {} breakers  page {}/{}",
                self.cbic.mode,
                rows.len(),
                first / CB_STATUS_PAGE_SIZE + 1,
                pages
            ),
        );
        button(c, page_rect(false), "PREV", first > 0, false);
        button(
            c,
            page_rect(true),
            "NEXT",
            first + CB_STATUS_PAGE_SIZE < rows.len(),
            false,
        );
        for slot in 0..CB_STATUS_PAGE_SIZE {
            let Some(r) = rows.get(first + slot) else { break };
            let rr = row_rect(slot);
            let sel = first + slot == self.selected;
            if sel {
                c.fill_rect(rr.x, rr.y, rr.w, rr.h, Color::PANEL2);
                c.stroke_rect(rr.x, rr.y, rr.w, rr.h, &Stroke::new(Color::WHITE, 1.2));
            }
            icon(c, rr.x + 8.0, rr.y + 4.0, r.state, r.kind);
            put(c, rr.x + 32.0, rr.y + 16.0, &r.target(), 11.0, Color::DIM);
            put(
                c,
                rr.x + 118.0,
                rr.y + 16.0,
                &ellipsize(&r.name, 11.0, 218.0),
                11.0,
                Color::WHITE,
            );
            let (t, col) = state_view(r.state);
            put(c, rr.x + 346.0, rr.y + 16.0, t, 11.0, col);
            if r.load_a > 0.0 {
                put_end(
                    c,
                    rr.x + rr.w - 8.0,
                    rr.y + 16.0,
                    &format!("{:.1} A", r.load_a),
                    11.0,
                    Color::DIM,
                );
            }
        }
        if rows.is_empty() {
            put(
                c,
                RX,
                ROW_Y + 14.0,
                "no breakers in the CBIC database",
                11.0,
                Color::DIM,
            );
        }
        for (i, cmd) in CbCommand::ALL.iter().enumerate() {
            let enabled = self.selected_cb().is_some_and(|r| r.kind != BreakerKind::Tcb);
            button(c, btn_rect(i), cmd.label(), enabled, false);
        }
        self.draw_selected(c);
    }

    fn draw_selected(&self, c: &mut Canvas) {
        let y = 520.0;
        heading(c, RX, y, RW, "SELECTED");
        match self.selected_cb() {
            Some(r) => {
                let (t, col) = state_view(r.state);
                put(
                    c,
                    RX,
                    y + 22.0,
                    &ellipsize(&format!("{}  {}", r.target(), r.name), 12.0, RW),
                    12.0,
                    Color::WHITE,
                );
                put(c, RX, y + 40.0, &format!("{:?}  state", r.kind), 11.0, Color::DIM);
                put(c, RX + 130.0, y + 40.0, t, 11.0, col);
                put(
                    c,
                    RX + 260.0,
                    y + 40.0,
                    &format!("load {:.1} A", r.load_a),
                    11.0,
                    Color::DIM,
                );
            }
            None => put(c, RX, y + 22.0, "none", 11.0, Color::DIM),
        }
        let msg = if self.notice.is_empty() {
            "click a row, then OPEN, CLOSE, LOCK, UNLOCK or RESET"
        } else {
            self.notice.as_str()
        };
        put(c, RX, y + 62.0, &ellipsize(msg, 11.0, RW), 11.0, Color::WHITE);
        put(
            c,
            RX,
            y + 80.0,
            "keys: Up Down select   Z open   V close   N lock   M unlock   Y reset",
            10.0,
            Color::DIM,
        );
        put(
            c,
            RX,
            y + 96.0,
            "ECB is controlled here, TCB shows indication only",
            10.0,
            Color::DIM,
        );
        frame(c, RX - 4.0, 12.0, RW + 8.0, VH - 24.0, Color::LINE);
    }
}

impl Page for EicasPage {
    fn id(&self) -> &str {
        "electrical.eicas"
    }

    fn title(&self) -> &str {
        "Electrical EICAS and CBIC"
    }

    fn draw(&self, c: &mut Canvas, area: Rect) {
        let s = (area.w / VW).min(area.h / VH);
        self.scale.set(s);
        c.save();
        c.translate(area.x, area.y);
        c.scale(s, s);
        c.fill_rect(0.0, 0.0, VW, VH, Color::BG);
        self.draw_eicas(c);
        self.draw_cbic(c);
        c.restore();
    }

    fn click(&mut self, x: f32, y: f32) -> bool {
        let s = self.scale.get().max(0.01);
        let (px, py) = (x / s, y / s);
        let first = self.first();
        if page_rect(true).contains(px, py) {
            let next = first + CB_STATUS_PAGE_SIZE;
            if next < self.cbic.rows.len() {
                self.selected = next;
            }
            return true;
        }
        if page_rect(false).contains(px, py) {
            self.selected = first.saturating_sub(CB_STATUS_PAGE_SIZE);
            return true;
        }
        for slot in 0..CB_STATUS_PAGE_SIZE {
            if row_rect(slot).contains(px, py) && first + slot < self.cbic.rows.len() {
                self.selected = first + slot;
                self.notice.clear();
                return true;
            }
        }
        for (i, cmd) in CbCommand::ALL.iter().enumerate() {
            if btn_rect(i).contains(px, py) {
                self.command(*cmd);
                return true;
            }
        }
        false
    }

    fn detail_title(&self) -> String {
        match self.selected_cb() {
            Some(r) => format!("Breaker {}", r.target()),
            None => String::from("EICAS and CBIC"),
        }
    }

    fn detail(&self) -> Vec<DetailSection> {
        let mut v = Vec::new();
        if let Some(r) = self.selected_cb() {
            let (t, _) = state_view(r.state);
            let st = if matches!(r.state, CbState::Tripped | CbState::Unknown) {
                Cell::fault(t)
            } else {
                Cell::new(t)
            };
            v.push(DetailSection {
                title: String::from("Selected breaker"),
                rows: vec![
                    (String::from("name"), Cell::new(&r.name)),
                    (String::from("kind"), Cell::new(format!("{:?}", r.kind))),
                    (String::from("state"), st),
                    (String::from("load"), Cell::new(format!("{:.1} A", r.load_a))),
                ],
            });
        }
        v.push(DetailSection {
            title: String::from("EICAS"),
            rows: self
                .messages()
                .into_iter()
                .map(|(l, t)| {
                    let cell = if matches!(l, EicasLevel::Warning) {
                        Cell::fault(t)
                    } else {
                        Cell::new(t)
                    };
                    (level_tag(l).to_string(), cell)
                })
                .collect(),
        });
        v
    }

    fn table(&self) -> Table {
        Table {
            title: String::from("CBIC breakers"),
            columns: vec![String::from("id"), String::from("name"), String::from("state")],
            rows: self
                .cbic
                .rows
                .iter()
                .map(|r| {
                    let (t, _) = state_view(r.state);
                    let st = if matches!(r.state, CbState::Tripped | CbState::Unknown) {
                        Cell::fault(t)
                    } else {
                        Cell::new(t)
                    };
                    vec![Cell::new(r.target()), Cell::new(&r.name), st]
                })
                .collect(),
        }
    }
}
