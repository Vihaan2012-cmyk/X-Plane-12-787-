use render::{Anchor, Canvas, Color, Stroke, TextStyle};

pub fn tone(live: bool, fault: bool) -> Color {
    if fault {
        Color::BAD
    } else if live {
        Color::WHITE
    } else {
        Color::FAINT
    }
}

pub fn wire_stroke(live: bool, fault: bool) -> Stroke {
    let s = Stroke::new(tone(live, fault), if live { 1.6 } else { 1.0 });
    if live || fault { s } else { s.dashed(&[4.0, 3.0]) }
}

pub fn put(c: &mut Canvas, x: f32, y: f32, s: &str, size: f32, color: Color) {
    c.text(x, y, s, size, color);
}

pub fn put_mid(c: &mut Canvas, x: f32, y: f32, s: &str, size: f32, color: Color) {
    c.draw_text(x, y, s, &TextStyle::new(size, color).anchor(Anchor::Middle));
}

pub fn put_end(c: &mut Canvas, x: f32, y: f32, s: &str, size: f32, color: Color) {
    c.draw_text(x, y, s, &TextStyle::new(size, color).anchor(Anchor::End));
}

pub fn symbol(c: &mut Canvas, x: f32, y: f32, closed: bool, live: bool, fault: bool) {
    let col = tone(live || closed, fault);
    if closed {
        c.fill_rect(x - 4.5, y - 4.5, 9.0, 9.0, col);
    } else {
        c.fill_rect(x - 4.5, y - 4.5, 9.0, 9.0, Color::BG);
        c.stroke_rect(x - 4.5, y - 4.5, 9.0, 9.0, &Stroke::new(col, 1.2));
    }
}

pub fn volts(v: f64) -> String {
    if v.abs() >= 100.0 {
        format!("{:.0}V", v)
    } else {
        format!("{:.1}V", v)
    }
}

pub fn amps(a: f64) -> String {
    if a.abs() >= 100.0 {
        format!("{:.0} A", a)
    } else {
        format!("{:.1} A", a)
    }
}

pub fn frame(c: &mut Canvas, x: f32, y: f32, w: f32, h: f32, color: Color) {
    c.stroke_rect(x, y, w, h, &Stroke::new(color, 1.2));
}
