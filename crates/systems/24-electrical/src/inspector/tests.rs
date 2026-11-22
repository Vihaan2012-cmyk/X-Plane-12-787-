use super::*;
use render::page::Page;
use render::{Canvas, Color, Rect, hash_canvas, write_png};

fn shot(page: &dyn Page, name: &str) -> u64 {
    let mut c = Canvas::new(1000, 672);
    c.clear(Color::BG);
    page.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    let dir = "e:/X-Plane 787/target/agents/inspector";
    let _ = std::fs::create_dir_all(dir);
    write_png(format!("{dir}/{name}.png"), 1000, 672, c.rgba()).unwrap();
    hash_canvas(&c)
}

const GOLD: [(&str, u64); 5] = [
    ("single_line", 0xb7afb36e392225b3),
    ("single_line_selected", 0xf3bece73d5cf6bd3),
    ("panels", 0x104a68409c4dacc3),
    ("rpdu", 0xe8f01e84130274bb),
    ("breaker", 0x8df707bc2e7547aa),
];

fn hashes() -> Vec<u64> {
    let d = InspectorData::demo();
    let mut sl = SingleLinePage::new(d.clone());
    let a = shot(&sl, "single_line");
    sl.select_contactor("R TRU RLY");
    let b = shot(&sl, "single_line_selected");
    let p = shot(&PanelPage::new(d.clone()), "panels");
    let mut r = RpduPage::new(d.clone());
    r.select_rpdu("RPDU 75");
    r.select_channel(12);
    let rr = shot(&r, "rpdu");
    let br = shot(&BreakerPage::new(d, "RPDU 75", 12), "breaker");
    vec![a, b, p, rr, br]
}

#[test]
fn pages_render_and_match_golden() {
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
fn rendering_is_deterministic() {
    assert_eq!(hashes(), hashes());
}

#[test]
fn single_line_click_selects_contactor() {
    let mut p = SingleLinePage::new(InspectorData::demo());
    let mut c = Canvas::new(1000, 672);
    p.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    assert!(p.click(85.0, 82.0));
    assert_eq!(p.selection(), &Selection::Contactor("L1 GCB".to_string()));
    assert_eq!(p.detail_title(), "Contactor L1 GCB");
    assert_eq!(p.table().rows[0][0].text, "L1 GCB");
    assert!(p.click(60.0, 150.0));
    assert_eq!(p.selection(), &Selection::Bus("L1".to_string()));
    assert!(p.click(500.0, 640.0));
    assert_eq!(p.selection(), &Selection::None);
}

#[test]
fn faulted_contactor_is_red_in_table_and_detail() {
    let mut p = SingleLinePage::new(InspectorData::demo());
    p.select_contactor("R TRU RLY");
    assert!(p.table().rows[0][3].fault);
    assert!(p.detail()[0].rows[0].1.fault);
}

#[test]
fn rpdu_page_selects_channels() {
    let mut r = RpduPage::new(InspectorData::demo());
    let mut c = Canvas::new(1000, 672);
    r.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
    assert!(r.select_channel(19));
    assert_eq!(r.selected_channel(), Some(19));
    assert!(r.table().rows[0][3].fault);
    assert!(r.click(20.0, 70.0));
    assert_eq!(r.selected_channel(), Some(1));
    assert!(r.click(100.0, 15.0));
    assert_eq!(r.rpdu_id(), "RPDU 71");
}

#[test]
fn panel_page_lists_panel_contactors() {
    let mut p = PanelPage::new(InspectorData::demo());
    assert!(p.select("P100"));
    assert_eq!(p.table().rows.len(), 8);
    assert!(!p.select("P999"));
}

#[test]
fn breaker_trip_curve_is_monotonic() {
    assert!(breaker::trip_time_s(2.0, 8.0) > breaker::trip_time_s(5.0, 8.0));
    assert!(breaker::trip_time_s(9.0, 8.0) < 0.1);
}

#[test]
fn from_net_snapshot_is_tolerant() {
    let mut net = crate::net::ElectricalNet::new();
    net.set_machine_rpm("VFSG_L1", 7200.0);
    net.set_contactor("L1 GCB", true);
    let mut snap = None;
    for _ in 0..20 {
        snap = net.step(10_000).ok();
    }
    let d = InspectorData::from_net_snapshot(&snap.unwrap());
    assert!(d.bus_live("L1"));
    assert!(d.source("VFSG_L1").unwrap().online);
    let p = SingleLinePage::new(d);
    let mut c = Canvas::new(1000, 672);
    p.draw(&mut c, Rect::new(0.0, 0.0, 1000.0, 672.0));
}
