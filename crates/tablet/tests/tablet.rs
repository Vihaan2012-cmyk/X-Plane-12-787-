use render::{Color, hash_canvas};
use tablet::{DemoElectricalPage, NAV, Page, Tablet};

fn demo_tablet() -> Tablet {
    let mut t = Tablet::new();
    t.register(Box::new(DemoElectricalPage::new()));
    t.set_sim_time(2843.112);
    t.set_phase("cruise");
    t
}

#[test]
fn renders_demo_png() {
    let t = demo_tablet();
    let c = t.render();
    assert_eq!((c.width(), c.height()), (1600, 960));
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/agents/render/tablet-demo.png");
    render::write_png(&path, c.width(), c.height(), c.rgba()).unwrap();
    assert!(std::fs::metadata(&path).unwrap().len() > 100_000);
    assert_eq!(c.pixel(5, 5), Color::PANEL2);
    assert_eq!(c.pixel(1100, 650), Color::BG);
}

#[test]
fn red_only_for_faults() {
    let c = demo_tablet().render();
    let red = c
        .rgba()
        .chunks_exact(4)
        .filter(|p| p[0] == 0xe0 && p[1] == 0x55)
        .count();
    assert!(red > 50);
    let mut t = demo_tablet();
    t.select("engines");
    let c2 = t.render();
    let red2 = c2
        .rgba()
        .chunks_exact(4)
        .filter(|p| p[0] > 0x90 && p[1] < 0x70 && p[2] < 0x70)
        .count();
    assert_eq!(red2, 0);
}

#[test]
fn render_is_deterministic() {
    assert_eq!(
        hash_canvas(&demo_tablet().render()),
        hash_canvas(&demo_tablet().render())
    );
}

#[test]
fn nav_lists_every_group_and_selects_on_click() {
    let names: Vec<&str> = NAV.iter().map(|g| g.name).collect();
    assert_eq!(names, ["POWER", "FLIGHT", "ENVIRONMENT", "AVIONICS", "CABIN"]);
    assert_eq!(
        NAV[0].items.iter().map(|i| i.label).collect::<Vec<_>>(),
        [
            "Electrical",
            "  overhead panel",
            "  synoptic",
            "  EICAS and CBIC",
            "  power panels",
            "  RPDUs",
            "  breaker",
            "  protection",
            "Engines",
            "APU",
            "Fuel",
            "Hydraulics"
        ]
    );
    let mut t = demo_tablet();
    assert_eq!(t.selected(), "electrical");
    let before = hash_canvas(&t.render());
    assert!(t.click(40.0, 38.0 + 10.0 + 22.0 + 23.0 * 10.0 + 5.0));
    assert_eq!(t.selected(), "fuel");
    assert_ne!(hash_canvas(&t.render()), before);
    assert!(!t.click(40.0, 5.0));
}

#[test]
fn diagram_click_selects_breaker() {
    let mut t = demo_tablet();
    let d = t.layout().diagram();
    let before = hash_canvas(&t.render());
    assert!(t.click(d.x + 230.0, d.y + 150.0));
    assert_ne!(hash_canvas(&t.render()), before);
    assert!(!t.click(d.x + 5.0, d.y + 5.0));
}

#[test]
fn page_trait_and_layout() {
    let p = DemoElectricalPage::new();
    assert_eq!(p.id(), "electrical");
    assert_eq!(p.title(), "Electrical");
    assert_eq!(p.tripped_count(), 1);
    assert_eq!(p.selected_breaker(), "TRU R CB");
    assert_eq!(p.table().rows.len(), 6);
    let l = Tablet::new().layout();
    assert_eq!(l.diagram().w, 1000.0);
    assert_eq!(l.bottom().h, 250.0);
    assert_eq!(l.side().x, 1170.0);
}

#[test]
fn register_replaces_same_id() {
    let mut t = demo_tablet();
    t.register(Box::new(DemoElectricalPage::new()));
    assert_eq!(t.page_ids(), ["electrical"]);
}
