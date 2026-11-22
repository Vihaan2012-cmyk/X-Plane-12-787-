mod canvas;
mod color;
mod font;
mod golden;
mod math;
pub mod page;
mod png;
mod raster;
mod text;

pub use canvas::{Canvas, Cap, Join, Stroke};
pub use color::Color;
pub use font::{Anchor, OutlineFace, TextStyle, glyph_strokes, install_face, measure_text};
pub use golden::{GoldenError, check_golden, hash_bytes, hash_canvas};
pub use math::{Affine, Rect, cos_f, sin_f};
pub use page::{Cell, DetailSection, Page, Table};
pub use png::{decode_png_rgba, encode_png, write_png};
pub use raster::FillRule;
pub use text::{ellipsize, fit_size};
