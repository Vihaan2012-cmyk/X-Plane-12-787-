use super::*;
use crate::crew::{CrewInputs, build_synoptic};
use render::page::Page;
use render::{Canvas, Color, Rect, hash_canvas, write_png};

fn shot(page: &dyn Page, name: &str) -> u64 {
    let mut c = Canvas::new(1000, 672);
    c.clear(Color::BG);
    page.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    let dir = "e:/X-Plane 787/target/agents/uipages/shots";
    let _ = std::fs::create_dir_all(dir);
    write_png(format!("{dir}/{name}.png"), 1000, 672, c.rgba()).unwrap();
    hash_canvas(&c)
}

fn lit_state() -> OverheadState {
    let mut panel = crate::crew::OverheadPanel::new();
    panel.set_normal();
    panel.set_apu_selector(crate::crew::ApuSelector::On);
    panel.gen_ctrl[1].press();
    let inp = CrewInputs {
        ext_fwd_inserted: [true, false],
        ext_fwd_quality_ok: [true, false],
        vfsg_lop: [false, true, false, false],
        engine_running: [true, true],
        ..CrewInputs::default()
    };
    let (_, lamps) = panel.update(&inp);
    let mut s = OverheadState::from_panel(&panel, &lamps);
    s.apu_fault = true;
    s.plugged = [true, false, false];
    s
}

fn busy_synoptic() -> SynopticPage {
    let inp = CrewInputs {
        engine_running: [true, true],
        gen_ctrl_on: [true; 4],
        gcb_closed: [true, true, true, false],
        ac_bus_energized: [true, true, true, false],
        ext_fwd_inserted: [false, true],
        ext_fwd_quality_ok: [false, true],
        engine_start_in_progress: [false, true],
        drive_disconnected: [false, true, false, false],
        main_batt_volts: 27.6,
        main_batt_current_a: -12.0,
        apu_batt_volts: 28.1,
        apu_batt_current_a: 3.0,
        load_shed: vec![String::from("CABIN LOADS"), String::from("GALLEY LOADS")],
        ..CrewInputs::default()
    };
    SynopticPage::new(build_synoptic(&inp, [true; 4], [true; 4]))
}

fn busy_eicas() -> EicasPage {
    let mut d = InspectorData::demo();
    d.eicas = vec![
        EicasMessage {
            level: EicasLevel::Status,
            text: "ELEC L1 BUS ISLN".to_string(),
        },
        EicasMessage {
            level: EicasLevel::Warning,
            text: "ELEC GEN SYS FAIL".to_string(),
        },
        EicasMessage {
            level: EicasLevel::Caution,
            text: "ELEC AC BUS L1".to_string(),
        },
        EicasMessage {
            level: EicasLevel::Advisory,
            text: "ELEC GEN OFF L1".to_string(),
        },
        EicasMessage {
            level: EicasLevel::Advisory,
            text: "MAIN BATTERY DISCH".to_string(),
        },
    ];
    let mut p = EicasPage::new(d);
    p.select_id(2125831);
    p
}

const GOLD: [(&str, u64); 6] = [
    ("overhead_normal", 0xaf8513240fbc0c77),
    ("overhead_lit", 0x3f8a1269c212a5c0),
    ("synoptic_idle", 0xe841beff97ca638b),
    ("synoptic_busy", 0x95b60bf54efb1cfb),
    ("eicas_empty", 0x485a104fdb70772),
    ("eicas_busy", 0xe80371841fa55856),
];

fn hashes() -> Vec<u64> {
    vec![
        shot(&OverheadPage::new(OverheadState::demo()), "overhead_normal"),
        shot(&OverheadPage::new(lit_state()), "overhead_lit"),
        shot(&SynopticPage::demo(), "synoptic_idle"),
        shot(&busy_synoptic(), "synoptic_busy"),
        shot(&EicasPage::new(InspectorData::default()), "eicas_empty"),
        shot(&busy_eicas(), "eicas_busy"),
    ]
}

#[test]
fn new_pages_render_and_match_golden() {
    let h = hashes();
    let mut bad = Vec::new();
    for (i, (n, g)) in GOLD.iter().enumerate() {
        if *g != h[i] {
            bad.push(format!("(\"{}\", {:#x}),", n, h[i]));
        }
    }
    assert!(bad.is_empty(), "golden mismatch: {}", bad.join(" "));
}

#[test]
fn new_pages_are_deterministic() {
    assert_eq!(hashes(), hashes());
}

fn pressed(target: &str) -> Vec<PanelAction> {
    let mut p = OverheadPage::new(OverheadState::demo());
    let mut c = Canvas::new(1000, 672);
    p.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    let (x, y) = p.hit_center(target).unwrap();
    assert!(p.click(x, y));
    p.take_actions()
}

#[test]
fn overhead_click_queues_action() {
    let a = pressed("BATTERY");
    assert_eq!(a, vec![PanelAction::new("crew", "press", "BATTERY", Some("OFF"))]);
    let a = pressed("GEN_CTRL_R2");
    assert_eq!(
        a,
        vec![PanelAction::new("crew", "press", "GEN_CTRL_R2", Some("OFF"))]
    );
    let a = pressed("APU_SELECTOR_START");
    assert_eq!(
        a,
        vec![PanelAction::new("crew", "select", "APU_SELECTOR", Some("START"))]
    );
    let a = pressed("EXT_PWR_FWD_R");
    assert_eq!(a, vec![PanelAction::new("crew", "press", "EXT_PWR_FWD_R", None)]);
    let a = pressed("APU_GEN_L");
    assert_eq!(a[0].target, "APU_GEN_L");
    let a = pressed("CART_AFT");
    assert_eq!(a[0].actor, "ground");
    assert_eq!(a[0].action, "connect");
    assert_eq!(a[0].target, "EXT_PWR_AFT");
}

#[test]
fn drive_disc_needs_guard_lift_first() {
    let mut p = OverheadPage::new(OverheadState::demo());
    let mut c = Canvas::new(1000, 672);
    p.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    let (x, y) = p.hit_center("DRIVE_DISC_L2").unwrap();
    assert!(p.click(x, y));
    assert!(p.take_actions().is_empty());
    assert!(p.guard_open(1));
    assert!(p.click(x, y));
    assert_eq!(
        p.take_actions(),
        vec![PanelAction::new("crew", "press", "DRIVE_DISC_L2", None)]
    );
    assert!(!p.guard_open(1));
}

#[test]
fn click_outside_switches_is_ignored() {
    let mut p = OverheadPage::new(OverheadState::demo());
    let mut c = Canvas::new(1000, 672);
    p.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    assert!(!p.click(700.0, 640.0));
    assert!(p.take_actions().is_empty());
}

#[test]
fn cbic_buttons_queue_actions() {
    let mut p = EicasPage::new(InspectorData::default());
    let mut c = Canvas::new(1000, 672);
    p.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    let (x, y) = p.row_center(2);
    assert!(p.click(x, y));
    assert_eq!(p.selected_cb().unwrap().id, 2125831);
    let (x, y) = p.button_center(CbCommand::Close);
    assert!(p.click(x, y));
    let (x, y) = p.button_center(CbCommand::Lock);
    assert!(p.click(x, y));
    assert_eq!(
        p.take_actions(),
        vec![
            PanelAction::new("crew", "cbic_close", "CE2125831", None),
            PanelAction::new("crew", "cbic_lock", "CE2125831", None),
        ]
    );
}

#[test]
fn tcb_is_indication_only() {
    let mut p = EicasPage::new(InspectorData::default());
    assert!(p.select_id(3000101));
    assert!(!p.command(CbCommand::Open));
    assert!(p.take_actions().is_empty());
    p.select_id(1471831);
    p.select_next();
    assert!(p.command(CbCommand::Open));
    assert_eq!(p.take_actions()[0].action, "cbic_open");
}

#[test]
fn from_system_is_live() {
    let root = std::path::Path::new("e:/X-Plane 787/aircraft/24-electrical");
    let mut sys = crate::system::Electrical::new(root).unwrap();
    sys.step_idle(1_000_000);
    let d = InspectorData::from_system(&sys);
    assert!(d.sim_time_s > 0.5);
    assert!(!d.rpdus.is_empty());
    let csv = std::fs::read_to_string(root.join("breakers.csv")).unwrap();
    let before = CbicView::from_system(&sys).rows.len();
    let n = seed_cbic(&mut sys, &csv, 40, 10);
    let v = CbicView::from_system(&sys);
    assert_eq!(v.rows.len(), before + n);
    assert!(!v.rows.is_empty());
}
