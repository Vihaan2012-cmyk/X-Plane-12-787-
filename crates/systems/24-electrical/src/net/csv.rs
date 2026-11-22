use std::path::Path;

pub struct Table {
    head: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn parse(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            ',' => row.push(std::mem::take(&mut field)),
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            _ => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

impl Table {
    pub fn load(dir: &Path, file: &str) -> Result<Table, String> {
        let path = dir.join(file);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut rows = parse(&text);
        if rows.is_empty() {
            return Err(format!("{file}: empty"));
        }
        let head = rows.remove(0);
        Ok(Table { head, rows })
    }

    pub fn col(&self, row: &[String], name: &str) -> String {
        match self.head.iter().position(|h| h == name) {
            Some(i) => row.get(i).map(|s| s.trim().to_string()).unwrap_or_default(),
            None => String::new(),
        }
    }
}
