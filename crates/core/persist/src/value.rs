#[derive(Clone, Debug)]
pub enum Value {
    F64(f64),
    U64(u64),
    I64(i64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::F64(a), Self::F64(b)) => a.to_bits() == b.to_bits(),
            (Self::U64(a), Self::U64(b)) => a == b,
            (Self::I64(a), Self::I64(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            (Self::Bytes(a), Self::Bytes(b)) => a == b,
            (Self::Array(a), Self::Array(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::F64(_) => "f64",
            Self::U64(_) => "u64",
            Self::I64(_) => "i64",
            Self::Bool(_) => "bool",
            Self::Str(_) => "str",
            Self::Bytes(_) => "bytes",
            Self::Array(_) => "array",
        }
    }

    pub fn encode(&self) -> String {
        let mut out = String::new();
        self.encode_into(&mut out);
        out
    }

    fn encode_into(&self, out: &mut String) {
        match self {
            Self::F64(v) => {
                out.push_str(&format!("f:{:016x}({:?})", v.to_bits(), v));
            }
            Self::U64(v) => out.push_str(&format!("u:{v}")),
            Self::I64(v) => out.push_str(&format!("i:{v}")),
            Self::Bool(v) => out.push_str(if *v { "b:1" } else { "b:0" }),
            Self::Str(s) => {
                out.push_str("s:\"");
                for c in s.chars() {
                    match c {
                        '\\' => out.push_str("\\\\"),
                        '"' => out.push_str("\\\""),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Self::Bytes(b) => {
                out.push_str("x:");
                for byte in b {
                    out.push_str(&format!("{byte:02x}"));
                }
            }
            Self::Array(items) => {
                out.push_str("a:[");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.encode_into(out);
                }
                out.push(']');
            }
        }
    }

    pub fn decode(text: &str) -> Result<Value, String> {
        let mut p = Parser {
            chars: text.chars().collect(),
            pos: 0,
        };
        let v = p.value(0)?;
        if p.pos != p.chars.len() {
            return Err("trailing characters after value".to_string());
        }
        Ok(v)
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected {c:?} at {}", self.pos))
        }
    }

    fn take_while(&mut self, f: impl Fn(char) -> bool) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if f(c) {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > 32 {
            return Err("array nesting too deep".to_string());
        }
        let tag = self.peek().ok_or("empty value")?;
        self.pos += 1;
        self.expect(':')?;
        match tag {
            'f' => {
                let hex = self.take_while(|c| c.is_ascii_hexdigit());
                if hex.len() != 16 || hex.chars().any(|c| c.is_ascii_uppercase()) {
                    return Err("f64 needs 16 lowercase hex digits".to_string());
                }
                let bits = u64::from_str_radix(&hex, 16).map_err(|e| e.to_string())?;
                if self.peek() == Some('(') {
                    self.take_while(|c| c != ')');
                    self.expect(')')?;
                }
                Ok(Value::F64(f64::from_bits(bits)))
            }
            'u' => {
                let d = self.take_while(|c| c.is_ascii_digit());
                d.parse::<u64>().map(Value::U64).map_err(|e| e.to_string())
            }
            'i' => {
                let neg = self.peek() == Some('-');
                if neg {
                    self.pos += 1;
                }
                let d = self.take_while(|c| c.is_ascii_digit());
                let text = if neg { format!("-{d}") } else { d };
                text.parse::<i64>().map(Value::I64).map_err(|e| e.to_string())
            }
            'b' => match self.peek() {
                Some('1') => {
                    self.pos += 1;
                    Ok(Value::Bool(true))
                }
                Some('0') => {
                    self.pos += 1;
                    Ok(Value::Bool(false))
                }
                _ => Err("bool must be 0 or 1".to_string()),
            },
            's' => self.string().map(Value::Str),
            'x' => {
                let hex = self.take_while(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
                if !hex.len().is_multiple_of(2) {
                    return Err("odd number of hex digits in bytes".to_string());
                }
                let mut out = Vec::with_capacity(hex.len() / 2);
                for pair in hex.as_bytes().chunks(2) {
                    let s = std::str::from_utf8(pair).map_err(|e| e.to_string())?;
                    out.push(u8::from_str_radix(s, 16).map_err(|e| e.to_string())?);
                }
                Ok(Value::Bytes(out))
            }
            'a' => {
                self.expect('[')?;
                let mut items = Vec::new();
                if self.peek() == Some(']') {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                loop {
                    items.push(self.value(depth + 1)?);
                    match self.peek() {
                        Some(',') => self.pos += 1,
                        Some(']') => {
                            self.pos += 1;
                            break;
                        }
                        _ => return Err("expected , or ] in array".to_string()),
                    }
                }
                Ok(Value::Array(items))
            }
            other => Err(format!("unknown type tag {other:?}")),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            let c = self.peek().ok_or("unterminated string")?;
            self.pos += 1;
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let e = self.peek().ok_or("unterminated escape")?;
                    self.pos += 1;
                    match e {
                        '\\' => out.push('\\'),
                        '"' => out.push('"'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            self.expect('{')?;
                            let hex = self.take_while(|c| c.is_ascii_hexdigit());
                            self.expect('}')?;
                            let code = u32::from_str_radix(&hex, 16).map_err(|e| e.to_string())?;
                            out.push(char::from_u32(code).ok_or("invalid unicode escape")?);
                        }
                        _ => return Err(format!("bad escape \\{e}")),
                    }
                }
                c if c.is_control() => return Err("raw control character in string".to_string()),
                c => out.push(c),
            }
        }
    }
}
