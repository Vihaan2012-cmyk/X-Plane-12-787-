use render::*;

fn base() -> Canvas {
    let mut c = Canvas::new(120, 80);
    c.clear(Color::BG);
    c
}

fn count_lit(c: &Canvas) -> usize {
    c.rgba().chunks_exact(4).filter(|p| p[0] != 0x12).count()
}

#[test]
fn fill_rect_exact_and_clipped() {
    let mut c = base();
    c.fill_rect(10.0, 10.0, 20.0, 10.0, Color::WHITE);
    assert_eq!(c.pixel(10, 10), Color::WHITE);
    assert_eq!(c.pixel(29, 19), Color::WHITE);
    assert_eq!(c.pixel(30, 19), Color::BG);
    assert_eq!(c.pixel(9, 10), Color::BG);
    c.save();
    c.clip_rect(Rect::new(50.0, 50.0, 10.0, 10.0));
    c.fill_rect(0.0, 0.0, 120.0, 80.0, Color::BAD);
    c.restore();
    assert_eq!(c.pixel(55, 55), Color::BAD);
    assert_eq!(c.pixel(49, 55), Color::BG);
    assert_eq!(c.pixel(60, 55), Color::BG);
    c.fill_rect(0.0, 0.0, 2.0, 2.0, Color::WHITE);
    assert_eq!(c.pixel(1, 1), Color::WHITE);
}

#[test]
fn half_pixel_coverage_is_antialiased() {
    let mut c = base();
    c.fill_rect(10.0, 10.0, 10.5, 5.0, Color::WHITE);
    let edge = c.pixel(20, 12);
    assert!(edge.r > 0x12 + 60 && edge.r < 255 - 60, "{edge:?}");
}

#[test]
fn diagonal_line_has_partial_pixels() {
    let mut c = base();
    c.line(5.0, 5.0, 100.0, 40.0, &Stroke::new(Color::WHITE, 1.5));
    let partial = c
        .rgba()
        .chunks_exact(4)
        .filter(|p| p[0] > 0x20 && p[0] < 0xf0)
        .count();
    assert!(partial > 20);
    assert!(count_lit(&c) > 100);
}

#[test]
fn transform_stack() {
    let mut c = base();
    c.save();
    c.translate(50.0, 40.0);
    c.scale(2.0, 2.0);
    c.fill_rect(0.0, 0.0, 5.0, 5.0, Color::WHITE);
    c.restore();
    assert_eq!(c.pixel(55, 45), Color::WHITE);
    assert_eq!(c.pixel(61, 51), Color::BG);
    assert_eq!(c.transform(), Affine::IDENTITY);
    c.save();
    c.translate(60.0, 40.0);
    c.rotate(std::f32::consts::FRAC_PI_2);
    c.fill_rect(0.0, 0.0, 10.0, 4.0, Color::WHITE);
    c.restore();
    assert_eq!(c.pixel(58, 45), Color::WHITE);
    assert_eq!(c.pixel(65, 41), Color::BG);
}

#[test]
fn circle_area_close_to_pi_r2() {
    let mut c = Canvas::new(100, 100);
    c.fill_circle(50.0, 50.0, 30.0, Color::WHITE);
    let area: f32 = c.rgba().chunks_exact(4).map(|p| p[3] as f32 / 255.0).sum();
    let expect = std::f32::consts::PI * 900.0;
    assert!((area - expect).abs() / expect < 0.01, "{area}");
}

#[test]
fn ellipse_rounded_arc_polygon() {
    let mut c = base();
    c.fill_ellipse(30.0, 30.0, 20.0, 10.0, Color::WHITE);
    assert_eq!(c.pixel(30, 30), Color::WHITE);
    assert_eq!(c.pixel(30, 15), Color::BG);
    let mut c = base();
    c.fill_rounded_rect(10.0, 10.0, 40.0, 30.0, 10.0, Color::WHITE);
    assert_eq!(c.pixel(30, 25), Color::WHITE);
    assert_eq!(c.pixel(10, 10), Color::BG);
    let mut c = base();
    c.arc(
        60.0,
        40.0,
        20.0,
        0.0,
        std::f32::consts::PI,
        &Stroke::new(Color::WHITE, 2.0),
    );
    assert!(c.pixel(60, 60).r > 0xc0);
    assert_eq!(c.pixel(60, 20), Color::BG);
    let mut c = base();
    c.fill_polygon(&[(10.0, 10.0), (50.0, 10.0), (30.0, 40.0)], Color::WHITE);
    assert_eq!(c.pixel(30, 15), Color::WHITE);
    assert_eq!(c.pixel(12, 35), Color::BG);
}

#[test]
fn stroke_polygon_leaves_interior() {
    let mut c = base();
    c.stroke_rect(20.0, 20.0, 40.0, 30.0, &Stroke::new(Color::WHITE, 2.0));
    assert!(c.pixel(40, 20).r > 0xc0);
    assert_eq!(c.pixel(40, 35), Color::BG);
    let mut c = base();
    c.stroke_rounded_rect(20.0, 20.0, 40.0, 30.0, 6.0, &Stroke::new(Color::WHITE, 2.0));
    assert_eq!(c.pixel(40, 35), Color::BG);
}

#[test]
fn dashed_line_has_gaps() {
    let mut c = base();
    c.line(
        10.0,
        40.0,
        110.0,
        40.0,
        &Stroke::new(Color::WHITE, 2.0).dashed(&[10.0, 10.0]),
    );
    let on = (10..110).filter(|&x| c.pixel(x, 40).r > 0xc0).count();
    assert!((45..=55).contains(&on), "{on}");
    assert_eq!(c.pixel(25, 40), Color::BG);
    assert_eq!(c.pixel(15, 40), Color::WHITE);
}

#[test]
fn line_widths_scale() {
    let mut a = base();
    a.line(10.0, 40.0, 110.0, 40.0, &Stroke::new(Color::WHITE, 1.0));
    let mut b = base();
    b.line(10.0, 40.0, 110.0, 40.0, &Stroke::new(Color::WHITE, 4.0));
    assert!(count_lit(&b) > count_lit(&a) * 2 - 1);
}

#[test]
fn text_measure_and_anchor() {
    let (w, h) = measure_text("ABC", 16.0);
    assert!((w - 28.0).abs() < 0.01 && h == 16.0);
    assert_eq!(measure_text("", 16.0).0, 0.0);
    let mut s = base();
    s.draw_text(
        60.0,
        40.0,
        "HELLO",
        &TextStyle::new(16.0, Color::WHITE).anchor(Anchor::Start),
    );
    let mut m = base();
    m.draw_text(
        60.0,
        40.0,
        "HELLO",
        &TextStyle::new(16.0, Color::WHITE).anchor(Anchor::Middle),
    );
    let mut e = base();
    e.draw_text(
        60.0,
        40.0,
        "HELLO",
        &TextStyle::new(16.0, Color::WHITE).anchor(Anchor::End),
    );
    let span = |c: &Canvas| {
        let xs: Vec<usize> = (0..120)
            .filter(|&x| (0..80).any(|y| c.pixel(x, y) != Color::BG))
            .collect();
        (xs[0], xs[xs.len() - 1])
    };
    let (s0, s1) = span(&s);
    let (m0, m1) = span(&m);
    let (e0, e1) = span(&e);
    assert!(s0 >= 59 && s1 > 80);
    assert!(m0 < 50 && m1 > 70);
    assert!(e1 <= 62 && e0 < 40);
}

#[test]
fn every_printable_glyph_draws() {
    for ch in ' '..='~' {
        let mut c = Canvas::new(30, 30);
        c.text(5.0, 20.0, &ch.to_string(), 16.0, Color::WHITE);
        let lit = c.rgba().chunks_exact(4).any(|p| p[3] > 0);
        assert_eq!(lit, ch != ' ', "glyph {ch:?}");
    }
    assert!(!glyph_strokes('A').is_empty());
    assert_eq!(glyph_strokes('\u{e9}'), glyph_strokes('?'));
}

#[test]
fn png_roundtrip() {
    let mut c = base();
    c.fill_circle(40.0, 40.0, 20.0, Color::BAD);
    c.text(5.0, 15.0, "PNG 123", 12.0, Color::WHITE);
    let bytes = encode_png(c.width(), c.height(), c.rgba());
    assert_eq!(&bytes[1..4], b"PNG");
    let (w, h, px) = decode_png_rgba(&bytes).unwrap();
    assert_eq!((w, h), (120, 80));
    assert_eq!(px, c.rgba());
    let mut big = Canvas::new(300, 300);
    big.clear(Color::WHITE);
    let b = encode_png(300, 300, big.rgba());
    assert_eq!(decode_png_rgba(&b).unwrap().2, big.rgba());
}

#[test]
fn write_png_file() {
    let c = base();
    let path = std::env::temp_dir().join("b789-render-test.png");
    write_png(&path, c.width(), c.height(), c.rgba()).unwrap();
    assert!(std::fs::metadata(&path).unwrap().len() > 50);
}

fn scene() -> Canvas {
    let mut c = Canvas::new(160, 100);
    c.clear(Color::BG);
    c.fill_rect(5.0, 5.0, 30.0, 20.0, Color::DIM);
    c.stroke_rounded_rect(40.0, 5.0, 40.0, 20.0, 5.0, &Stroke::new(Color::WHITE, 1.5));
    c.fill_circle(110.0, 20.0, 12.0, Color::BAD);
    c.line(
        5.0,
        40.0,
        150.0,
        60.0,
        &Stroke::new(Color::WHITE, 2.0).dashed(&[6.0, 3.0]),
    );
    c.arc(40.0, 80.0, 15.0, 0.5, 4.0, &Stroke::new(Color::WHITE, 1.0));
    c.text(70.0, 90.0, "Golden 787 [OK]", 12.0, Color::WHITE);
    c
}

fn text_canvas() -> Canvas {
    let mut c = Canvas::new(200, 30);
    c.clear(Color::BG);
    c.text(4.0, 20.0, "ABC xyz 0123 !?@#", 14.0, Color::WHITE);
    c
}

#[test]
fn golden_is_deterministic_and_sensitive() {
    let h = hash_canvas(&scene());
    assert_eq!(h, hash_canvas(&scene()));
    assert!(check_golden(&scene(), h).is_ok());
    let mut other = scene();
    other.set_pixel(1, 1, Color::WHITE);
    assert_ne!(hash_canvas(&other), h);
    let err = check_golden(&other, h).unwrap_err();
    assert_eq!(err.expected, h);
    assert!(err.to_string().contains("mismatch"));
}

#[test]
fn golden_scene_pinned() {
    assert_eq!(hash_canvas(&scene()), PINNED_SCENE);
}

#[test]
fn golden_text_pinned() {
    assert_eq!(hash_canvas(&text_canvas()), PINNED_TEXT);
}

#[test]
fn sin_cos_accurate() {
    for i in -200..200 {
        let x = i as f32 * 0.37;
        assert!((sin_f(x) - x.sin()).abs() < 1e-4);
        assert!((cos_f(x) - x.cos()).abs() < 1e-4);
    }
}

const PINNED_SCENE: u64 = 15262825661083421608;
const PINNED_TEXT: u64 = 10480863374162615052;

struct NullPage;

impl render::page::Page for NullPage {
    fn id(&self) -> &str {
        "null"
    }
    fn title(&self) -> &str {
        "Null"
    }
    fn draw(&self, canvas: &mut Canvas, area: Rect) {
        canvas.fill_rect(area.x, area.y, area.w, area.h, Color::BG);
    }
    fn click(&mut self, _x: f32, _y: f32) -> bool {
        false
    }
}

#[test]
fn page_trait_defaults() {
    use render::page::Page;
    let mut p = NullPage;
    assert_eq!(p.detail_title(), "Null");
    assert!(p.detail().is_empty());
    assert!(p.table().rows.is_empty());
    assert!(!p.click(1.0, 1.0));
    let mut c = Canvas::new(8, 8);
    p.draw(&mut c, Rect::new(0.0, 0.0, 8.0, 8.0));
    assert_eq!(c.pixel(3, 3), Color::BG);
    assert!(render::Cell::fault("x").fault);
}
