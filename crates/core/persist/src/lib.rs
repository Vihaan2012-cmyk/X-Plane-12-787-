mod document;
mod error;
mod gap;
mod migrate;
mod model;
mod storage;
mod value;

pub use document::{CURRENT_VERSION, Document, MAGIC, Section, fnv1a64, valid_name};
pub use error::PersistError;
pub use gap::{GapReport, apply_battery_gap, apply_part_gap, soak_toward};
pub use migrate::{MIGRATIONS, MigrationFn, migrate_to_current, migrate_with};
pub use model::{
    Airframe, BatteryRecord, BreakerRecord, BreakerState, Counter, FaultEntry, GapMode, Identity, PartRecord,
    SessionClock, Settings,
};
pub use storage::{DiskStorage, LoadSource, Loaded, MemStorage, SaveStore, Storage, decode_airframe};
pub use value::Value;
