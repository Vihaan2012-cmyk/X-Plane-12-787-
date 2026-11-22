use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistError {
    Io(String),
    NotFound(String),
    BadHeader,
    UnsupportedVersion(u32),
    Truncated,
    Trailing,
    Syntax { line: usize, msg: String },
    SectionChecksum(String),
    FileChecksum,
    SectionCount,
    DuplicateSection(String),
    DuplicateKey(String),
    InvalidName(String),
    Missing(String),
    WrongType(String),
    Migration(String),
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(m) => write!(f, "io error: {m}"),
            Self::NotFound(m) => write!(f, "not found: {m}"),
            Self::BadHeader => write!(f, "bad header"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported schema version {v}"),
            Self::Truncated => write!(f, "file truncated"),
            Self::Trailing => write!(f, "data after end marker"),
            Self::Syntax { line, msg } => write!(f, "syntax error at line {line}: {msg}"),
            Self::SectionChecksum(s) => write!(f, "checksum mismatch in section {s}"),
            Self::FileChecksum => write!(f, "file checksum mismatch"),
            Self::SectionCount => write!(f, "section count mismatch"),
            Self::DuplicateSection(s) => write!(f, "duplicate section {s}"),
            Self::DuplicateKey(s) => write!(f, "duplicate key {s}"),
            Self::InvalidName(s) => write!(f, "invalid name {s:?}"),
            Self::Missing(s) => write!(f, "missing key {s}"),
            Self::WrongType(s) => write!(f, "wrong value type for {s}"),
            Self::Migration(s) => write!(f, "migration failed: {s}"),
        }
    }
}

impl std::error::Error for PersistError {}
