use crate::error::PersistError;
use crate::value::Value;
use std::collections::BTreeMap;

pub const MAGIC: &str = "B789PERSIST";
pub const CURRENT_VERSION: u32 = 2;

pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-' | '/' | ':'))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Section {
    pub entries: BTreeMap<String, Value>,
}

impl Section {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.entries.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.entries.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    fn need(&self, key: &str) -> Result<&Value, PersistError> {
        self.entries
            .get(key)
            .ok_or_else(|| PersistError::Missing(key.to_string()))
    }

    pub fn f64(&self, key: &str) -> Result<f64, PersistError> {
        match self.need(key)? {
            Value::F64(v) => Ok(*v),
            _ => Err(PersistError::WrongType(key.to_string())),
        }
    }

    pub fn u64(&self, key: &str) -> Result<u64, PersistError> {
        match self.need(key)? {
            Value::U64(v) => Ok(*v),
            _ => Err(PersistError::WrongType(key.to_string())),
        }
    }

    pub fn i64(&self, key: &str) -> Result<i64, PersistError> {
        match self.need(key)? {
            Value::I64(v) => Ok(*v),
            _ => Err(PersistError::WrongType(key.to_string())),
        }
    }

    pub fn bool(&self, key: &str) -> Result<bool, PersistError> {
        match self.need(key)? {
            Value::Bool(v) => Ok(*v),
            _ => Err(PersistError::WrongType(key.to_string())),
        }
    }

    pub fn str(&self, key: &str) -> Result<&str, PersistError> {
        match self.need(key)? {
            Value::Str(v) => Ok(v),
            _ => Err(PersistError::WrongType(key.to_string())),
        }
    }

    pub fn bytes(&self, key: &str) -> Result<&[u8], PersistError> {
        match self.need(key)? {
            Value::Bytes(v) => Ok(v),
            _ => Err(PersistError::WrongType(key.to_string())),
        }
    }

    pub fn array(&self, key: &str) -> Result<&[Value], PersistError> {
        match self.need(key)? {
            Value::Array(v) => Ok(v),
            _ => Err(PersistError::WrongType(key.to_string())),
        }
    }

    pub fn f64_array(&self, key: &str) -> Result<Vec<f64>, PersistError> {
        self.array(key)?
            .iter()
            .map(|v| match v {
                Value::F64(x) => Ok(*x),
                _ => Err(PersistError::WrongType(key.to_string())),
            })
            .collect()
    }

    pub fn f64_or(&self, key: &str, default: f64) -> Result<f64, PersistError> {
        if self.contains(key) {
            self.f64(key)
        } else {
            Ok(default)
        }
    }

    fn body(&self) -> Result<String, PersistError> {
        let mut out = String::new();
        for (k, v) in &self.entries {
            if !valid_name(k) {
                return Err(PersistError::InvalidName(k.clone()));
            }
            out.push_str(k);
            out.push_str(" = ");
            out.push_str(&v.encode());
            out.push('\n');
        }
        Ok(out)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub version: u32,
    pub sections: BTreeMap<String, Section>,
}

impl Document {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            sections: BTreeMap::new(),
        }
    }

    pub fn current() -> Self {
        Self::new(CURRENT_VERSION)
    }

    pub fn section_mut(&mut self, name: &str) -> &mut Section {
        self.sections.entry(name.to_string()).or_default()
    }

    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.get(name)
    }

    pub fn to_text(&self) -> Result<String, PersistError> {
        let mut out = format!("{MAGIC} v{}\n", self.version);
        for (name, section) in &self.sections {
            if !valid_name(name) {
                return Err(PersistError::InvalidName(name.clone()));
            }
            let body = section.body()?;
            out.push_str(&format!("[{name}] crc={:016x}\n", fnv1a64(body.as_bytes())));
            out.push_str(&body);
        }
        let crc = fnv1a64(out.as_bytes());
        out.push_str(&format!(
            "[end] sections={} crc={crc:016x}\n",
            self.sections.len()
        ));
        Ok(out)
    }

    pub fn from_text(text: &str) -> Result<Document, PersistError> {
        let mut lines = text.split('\n').enumerate();
        let (_, header) = lines.next().ok_or(PersistError::BadHeader)?;
        let version = header
            .strip_prefix(MAGIC)
            .and_then(|r| r.strip_prefix(" v"))
            .and_then(|r| r.parse::<u32>().ok())
            .ok_or(PersistError::BadHeader)?;
        if version == 0 || version > CURRENT_VERSION {
            return Err(PersistError::UnsupportedVersion(version));
        }
        let mut doc = Document::new(version);
        let mut current: Option<(String, u64, String)> = None;
        let mut end: Option<(usize, u64)> = None;
        let mut order: Vec<(String, u64, String)> = Vec::new();
        let mut finished_at: Option<usize> = None;
        for (idx, line) in lines {
            let lineno = idx + 1;
            if let Some(pos) = finished_at {
                if line.is_empty() && pos + 1 == lineno {
                    continue;
                }
                return Err(PersistError::Trailing);
            }
            if let Some(rest) = line.strip_prefix("[end] ") {
                if let Some(c) = current.take() {
                    order.push(c);
                }
                let (count, crc) = parse_end(rest, lineno)?;
                end = Some((count, crc));
                finished_at = Some(lineno);
                continue;
            }
            if line.starts_with('[') {
                if let Some(c) = current.take() {
                    order.push(c);
                }
                let close = line.find("] crc=").ok_or_else(|| PersistError::Syntax {
                    line: lineno,
                    msg: "bad section header".to_string(),
                })?;
                let name = &line[1..close];
                if !valid_name(name) {
                    return Err(PersistError::InvalidName(name.to_string()));
                }
                let crc = parse_hex16(&line[close + 6..]).ok_or_else(|| PersistError::Syntax {
                    line: lineno,
                    msg: "bad section checksum".to_string(),
                })?;
                if doc.sections.contains_key(name) || order.iter().any(|(n, _, _)| n == name) {
                    return Err(PersistError::DuplicateSection(name.to_string()));
                }
                current = Some((name.to_string(), crc, String::new()));
                continue;
            }
            let Some((_, _, body)) = current.as_mut() else {
                if line.is_empty() {
                    continue;
                }
                return Err(PersistError::Syntax {
                    line: lineno,
                    msg: "entry outside section".to_string(),
                });
            };
            body.push_str(line);
            body.push('\n');
        }
        let Some((count, file_crc)) = end else {
            return Err(PersistError::Truncated);
        };
        for (name, crc, body) in &order {
            if fnv1a64(body.as_bytes()) != *crc {
                return Err(PersistError::SectionChecksum(name.clone()));
            }
        }
        let end_marker = text.rfind("[end] ").ok_or(PersistError::Truncated)?;
        if fnv1a64(&text.as_bytes()[..end_marker]) != file_crc {
            return Err(PersistError::FileChecksum);
        }
        if count != order.len() {
            return Err(PersistError::SectionCount);
        }
        for (name, _, body) in order {
            let mut section = Section::new();
            for (i, line) in body.lines().enumerate() {
                let (key, val) = line.split_once(" = ").ok_or_else(|| PersistError::Syntax {
                    line: i + 1,
                    msg: format!("bad entry in section {name}"),
                })?;
                if !valid_name(key) {
                    return Err(PersistError::InvalidName(key.to_string()));
                }
                let value = Value::decode(val).map_err(|msg| PersistError::Syntax {
                    line: i + 1,
                    msg: format!("{name}.{key}: {msg}"),
                })?;
                if section.entries.insert(key.to_string(), value).is_some() {
                    return Err(PersistError::DuplicateKey(format!("{name}.{key}")));
                }
            }
            doc.sections.insert(name, section);
        }
        Ok(doc)
    }
}

fn parse_hex16(s: &str) -> Option<u64> {
    if s.len() != 16 || !s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return None;
    }
    u64::from_str_radix(s, 16).ok()
}

fn parse_end(rest: &str, line: usize) -> Result<(usize, u64), PersistError> {
    let bad = || PersistError::Syntax {
        line,
        msg: "bad end marker".to_string(),
    };
    let rest = rest.strip_prefix("sections=").ok_or_else(bad)?;
    let (count, crc) = rest.split_once(" crc=").ok_or_else(bad)?;
    let count = count.parse::<usize>().map_err(|_| bad())?;
    let crc = parse_hex16(crc).ok_or_else(bad)?;
    Ok((count, crc))
}
