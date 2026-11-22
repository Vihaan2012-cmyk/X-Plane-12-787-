use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum NvmValue {
    U64(u64),
    I64(i64),
    F64(f64),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Minor,
    Major,
    Critical,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FaultRecord {
    pub time_us: u64,
    pub code: u32,
    pub severity: Severity,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NvmError {
    WornOut,
    Corrupt,
}

#[derive(Clone, Debug)]
struct NvmEntry {
    value: NvmValue,
    writes: u32,
}

#[derive(Clone, Debug)]
pub struct Nvm {
    entries: BTreeMap<String, NvmEntry>,
    endurance: u32,
    total_writes: u64,
    faults: Vec<FaultRecord>,
    fault_capacity: usize,
    corrupt: bool,
}

impl Nvm {
    pub fn new(endurance: u32, fault_capacity: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            endurance,
            total_writes: 0,
            faults: Vec::new(),
            fault_capacity: fault_capacity.max(1),
            corrupt: false,
        }
    }

    pub fn write(&mut self, key: &str, value: NvmValue) -> Result<(), NvmError> {
        if self.corrupt {
            return Err(NvmError::Corrupt);
        }
        let entry = self.entries.entry(key.to_string()).or_insert(NvmEntry {
            value: value.clone(),
            writes: 0,
        });
        if entry.writes >= self.endurance {
            return Err(NvmError::WornOut);
        }
        entry.value = value;
        entry.writes += 1;
        self.total_writes += 1;
        Ok(())
    }

    pub fn read(&self, key: &str) -> Option<&NvmValue> {
        if self.corrupt {
            return None;
        }
        self.entries.get(key).map(|e| &e.value)
    }

    pub fn read_u64(&self, key: &str) -> Option<u64> {
        match self.read(key) {
            Some(NvmValue::U64(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn increment(&mut self, key: &str) -> Result<u64, NvmError> {
        let next = self.read_u64(key).unwrap_or(0) + 1;
        self.write(key, NvmValue::U64(next))?;
        Ok(next)
    }

    pub fn write_count(&self, key: &str) -> u32 {
        self.entries.get(key).map_or(0, |e| e.writes)
    }

    pub fn total_writes(&self) -> u64 {
        self.total_writes
    }

    pub fn wear_fraction(&self, key: &str) -> f64 {
        self.write_count(key) as f64 / self.endurance.max(1) as f64
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    pub fn log_fault(&mut self, time_us: u64, code: u32, severity: Severity, text: &str) {
        if self.faults.len() >= self.fault_capacity {
            self.faults.remove(0);
        }
        self.faults.push(FaultRecord {
            time_us,
            code,
            severity,
            text: text.to_string(),
        });
    }

    pub fn faults(&self) -> &[FaultRecord] {
        &self.faults
    }

    pub fn fault_count(&self, code: u32) -> usize {
        self.faults.iter().filter(|f| f.code == code).count()
    }

    pub fn clear_faults(&mut self) {
        self.faults.clear();
    }

    pub fn set_corrupt(&mut self, corrupt: bool) {
        self.corrupt = corrupt;
    }

    pub fn is_corrupt(&self) -> bool {
        self.corrupt
    }
}
