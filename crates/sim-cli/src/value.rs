use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Value>),
    Table(BTreeMap<String, Value>),
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Num(n) => Some(*n != 0.0),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn from_toml(v: &toml::Value) -> Value {
        match v {
            toml::Value::Boolean(b) => Value::Bool(*b),
            toml::Value::Integer(i) => Value::Num(*i as f64),
            toml::Value::Float(f) => Value::Num(*f),
            toml::Value::String(s) => Value::Str(s.clone()),
            toml::Value::Datetime(d) => Value::Str(d.to_string()),
            toml::Value::Array(a) => Value::List(a.iter().map(Value::from_toml).collect()),
            toml::Value::Table(t) => {
                Value::Table(t.iter().map(|(k, v)| (k.clone(), Value::from_toml(v))).collect())
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Num(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Table(t) => {
                write!(f, "{{")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}={v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}
