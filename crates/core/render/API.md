# b789-render (lib `render`)

CPU 2D renderer, no dependencies, deterministic. RGBA8, straight alpha, source-over blending. Anti-aliasing by 6 sub-scanlines with exact horizontal coverage.

## Types
- `Color { r, g, b, a: u8 }`: `rgb`, `rgba`, `hex(0xRRGGBB)`, `with_alpha`, `lerp`. Consts: `BLACK WHITE TRANSPARENT BG PANEL PANEL2 LINE DIM FAINT BAD` (inspector palette; `BAD` = #e05555, faults only).
- `page` module (`render::page`, re-exported at the root): `trait Page` (`id`, `title`, `draw(&self, &mut Canvas, area: Rect)`, `click(&mut self, x, y) -> bool` area-relative, defaults `detail_title`, `detail() -> Vec<DetailSection>`, `table() -> Table`), `Cell { text, fault }` (`new`, `fault`), `DetailSection { title, rows: Vec<(String, Cell)> }`, `Table { title, columns, rows }`. Systems implement `Page`; the tablet hosts them.
- `Rect { x, y, w, h }`: `new`, `right`, `bottom`, `contains`, `inset`, `intersect`.
- `Affine`: `IDENTITY`, `translate`, `scale`, `rotate`, `then`, `apply`, `mean_scale`.
- `Stroke { color, width, cap, join, dash, dash_offset }`: `new(color, width)`, `.dashed(&[on, off, ..])`, `.with_cap(Cap)`, `.with_join(Join)`, `.with_dash_offset`.
- `Cap { Butt, Round, Square }`, `Join { Miter, Round, Bevel }`, `FillRule { NonZero, EvenOdd }`.
- `Anchor { Start, Middle, End }`, `TextStyle { size, color, anchor, weight }`: `new(size, color)`, `.anchor(..)`, `.weight(..)`.
- `GoldenError { expected, actual }`.

## Canvas
- `Canvas::new(w, h)` (transparent), `width`, `height`, `rgba()`, `rgba_mut()`, `into_rgba()`, `pixel(x, y)`, `set_pixel(x, y, color)`, `clear(color)`, `draw_canvas(&src, dx, dy)`.
- State: `save`, `restore` (transform and clip), `translate`, `scale`, `rotate(rad)`, `reset_transform`, `transform`, `clip_rect(Rect)` (intersects, device-space bounding box), `reset_clip`, `clip_bounds`.
- Fill: `fill_rect`, `fill_rounded_rect`, `fill_polygon`, `fill_polygon_rule`, `fill_circle`, `fill_ellipse`, `fill_pie`.
- Stroke: `line`, `polyline`, `stroke_polygon`, `stroke_path(pts, closed, &Stroke)`, `stroke_rect`, `stroke_rounded_rect`, `stroke_circle`, `stroke_ellipse`, `arc(cx, cy, r, start, end, &Stroke)`.
- Text: `text(x, y, str, size, color)`, `draw_text(x, y, str, &TextStyle)`. `y` is the baseline.

## Text
- Built-in single-stroke vector font, printable ASCII, monospace. Advance = 5/8 of size, cap height = 6/8 of size, descenders to 8/8. Non-ASCII draws as `?`.
- `measure_text(str, size) -> (width, height)`, `glyph_strokes(char) -> Vec<Vec<(f32, f32)>>` (grid units, x 0..4, y 0..8, baseline at 6).

## Export and golden
- `encode_png(w, h, rgba) -> Vec<u8>` (stored-deflate, no deps), `write_png(path, w, h, rgba)`, `decode_png_rgba(bytes)` (reads this encoder's output only).
- `hash_bytes(&[u8]) -> u64` (FNV-1a), `hash_canvas(&Canvas) -> u64`, `check_golden(&Canvas, expected) -> Result<(), GoldenError>`.
- `sin_f`, `cos_f`: platform-independent trig used by the renderer.
