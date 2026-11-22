#[derive(Clone, Debug, Default)]
pub struct Encoder {
    tokens: Vec<String>,
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next()? {
                '\\' => out.push('\\'),
                't' => out.push('\t'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                _ => return None,
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn str(&mut self, s: &str) -> &mut Self {
        self.tokens.push(escape(s));
        self
    }

    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.tokens.push(v.to_string());
        self
    }

    pub fn f64(&mut self, v: f64) -> &mut Self {
        self.tokens.push(format!("{:016x}", v.to_bits()));
        self
    }

    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.tokens.push(if v { "1" } else { "0" }.to_string());
        self
    }

    pub fn finish(&self) -> String {
        self.tokens.join("\t")
    }
}

#[derive(Clone, Debug)]
pub struct Decoder<'a> {
    parts: std::str::Split<'a, char>,
}

impl<'a> Decoder<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            parts: text.split('\t'),
        }
    }

    pub fn str(&mut self) -> Option<String> {
        unescape(self.parts.next()?)
    }

    pub fn u64(&mut self) -> Option<u64> {
        self.parts.next()?.parse().ok()
    }

    pub fn f64(&mut self) -> Option<f64> {
        u64::from_str_radix(self.parts.next()?, 16)
            .ok()
            .map(f64::from_bits)
    }

    pub fn bool(&mut self) -> Option<bool> {
        match self.parts.next()? {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }

    pub fn rest(&mut self) -> Vec<&'a str> {
        self.parts.by_ref().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_all_types() {
        let mut e = Encoder::new();
        e.str("a\tb\nc\\d\re").u64(77).f64(-0.1).bool(true).f64(f64::NAN);
        let text = e.finish();
        assert!(!text.contains('\n'));
        let mut d = Decoder::new(&text);
        assert_eq!(d.str().unwrap(), "a\tb\nc\\d\re");
        assert_eq!(d.u64().unwrap(), 77);
        assert_eq!(d.f64().unwrap().to_bits(), (-0.1f64).to_bits());
        assert!(d.bool().unwrap());
        assert!(d.f64().unwrap().is_nan());
        assert!(d.u64().is_none());
    }

    #[test]
    fn bad_escape_rejected() {
        let mut d = Decoder::new("\\q");
        assert!(d.str().is_none());
    }

    #[test]
    fn bad_bool_rejected() {
        let mut d = Decoder::new("2");
        assert!(d.bool().is_none());
    }

    #[test]
    fn rest_collects_remaining() {
        let mut d = Decoder::new("a\tb\tc");
        assert_eq!(d.str().unwrap(), "a");
        assert_eq!(d.rest(), vec!["b", "c"]);
    }
}
