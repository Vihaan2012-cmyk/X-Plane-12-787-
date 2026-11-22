use crate::{Canvas, Rect};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub text: String,
    pub fault: bool,
}

impl Cell {
    pub fn new(text: impl Into<String>) -> Cell {
        Cell {
            text: text.into(),
            fault: false,
        }
    }

    pub fn fault(text: impl Into<String>) -> Cell {
        Cell {
            text: text.into(),
            fault: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DetailSection {
    pub title: String,
    pub rows: Vec<(String, Cell)>,
}

pub trait Page {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn draw(&self, canvas: &mut Canvas, area: Rect);
    fn click(&mut self, x: f32, y: f32) -> bool;
    fn detail_title(&self) -> String {
        self.title().to_string()
    }
    fn detail(&self) -> Vec<DetailSection> {
        Vec::new()
    }
    fn table(&self) -> Table {
        Table::default()
    }
}
