use super::Electrical;
use crate::crew::{EicasLine, Level};
use crate::net::Snapshot;

impl Electrical {
    pub fn net_snapshot(&self) -> Option<&Snapshot> {
        self.plant.snapshot()
    }

    pub fn eicas_lines(&self) -> Vec<EicasLine> {
        self.eicas.list()
    }

    pub fn eicas_level_rank(level: Level) -> u8 {
        match level {
            Level::Warning => 0,
            Level::Caution => 1,
            Level::Advisory => 2,
            Level::Status => 3,
            Level::Memo => 4,
        }
    }
}
