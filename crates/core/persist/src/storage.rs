use crate::document::Document;
use crate::error::PersistError;
use crate::migrate::migrate_to_current;
use crate::model::Airframe;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub trait Storage {
    fn read(&self, path: &str) -> Result<Option<Vec<u8>>, PersistError>;
    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), PersistError>;
    fn rename(&mut self, from: &str, to: &str) -> Result<(), PersistError>;
    fn copy(&mut self, from: &str, to: &str) -> Result<(), PersistError>;
    fn remove(&mut self, path: &str) -> Result<(), PersistError>;
}

fn io(e: std::io::Error) -> PersistError {
    PersistError::Io(e.to_string())
}

#[derive(Clone, Debug)]
pub struct DiskStorage {
    pub root: PathBuf,
}

impl DiskStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Storage for DiskStorage {
    fn read(&self, path: &str) -> Result<Option<Vec<u8>>, PersistError> {
        match std::fs::read(self.root.join(path)) {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io(e)),
        }
    }

    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), PersistError> {
        use std::io::Write;
        std::fs::create_dir_all(&self.root).map_err(io)?;
        let mut f = std::fs::File::create(self.root.join(path)).map_err(io)?;
        f.write_all(data).map_err(io)?;
        f.sync_all().map_err(io)
    }

    fn rename(&mut self, from: &str, to: &str) -> Result<(), PersistError> {
        std::fs::rename(self.root.join(from), self.root.join(to)).map_err(io)
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), PersistError> {
        std::fs::copy(self.root.join(from), self.root.join(to))
            .map(|_| ())
            .map_err(io)
    }

    fn remove(&mut self, path: &str) -> Result<(), PersistError> {
        match std::fs::remove_file(self.root.join(path)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io(e)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemStorage {
    pub files: BTreeMap<String, Vec<u8>>,
    pub ops: u64,
    pub fail_at: Option<u64>,
    pub torn_write: bool,
}

impl MemStorage {
    pub fn new() -> Self {
        Self::default()
    }

    fn tick(&mut self) -> Result<(), PersistError> {
        self.ops += 1;
        if self.fail_at == Some(self.ops) {
            return Err(PersistError::Io("injected failure".to_string()));
        }
        Ok(())
    }
}

impl Storage for MemStorage {
    fn read(&self, path: &str) -> Result<Option<Vec<u8>>, PersistError> {
        Ok(self.files.get(path).cloned())
    }

    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), PersistError> {
        if let Err(e) = self.tick() {
            if self.torn_write {
                self.files
                    .insert(path.to_string(), data[..data.len() / 2].to_vec());
            }
            return Err(e);
        }
        self.files.insert(path.to_string(), data.to_vec());
        Ok(())
    }

    fn rename(&mut self, from: &str, to: &str) -> Result<(), PersistError> {
        self.tick()?;
        let data = self
            .files
            .remove(from)
            .ok_or_else(|| PersistError::NotFound(from.to_string()))?;
        self.files.insert(to.to_string(), data);
        Ok(())
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), PersistError> {
        self.tick()?;
        let data = self
            .files
            .get(from)
            .cloned()
            .ok_or_else(|| PersistError::NotFound(from.to_string()))?;
        self.files.insert(to.to_string(), data);
        Ok(())
    }

    fn remove(&mut self, path: &str) -> Result<(), PersistError> {
        self.tick()?;
        self.files.remove(path);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadSource {
    Main,
    Previous,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    pub airframe: Airframe,
    pub source: LoadSource,
    pub migrated_steps: u32,
    pub primary_error: Option<PersistError>,
}

pub fn decode_airframe(bytes: &[u8]) -> Result<(Airframe, u32), PersistError> {
    let text = std::str::from_utf8(bytes).map_err(|_| PersistError::Syntax {
        line: 0,
        msg: "invalid utf-8".to_string(),
    })?;
    let mut doc = Document::from_text(text)?;
    let steps = migrate_to_current(&mut doc)?;
    Ok((Airframe::from_document(&doc)?, steps))
}

#[derive(Clone, Debug)]
pub struct SaveStore<S: Storage> {
    pub storage: S,
    pub name: String,
}

impl<S: Storage> SaveStore<S> {
    pub fn new(storage: S, name: &str) -> Self {
        Self {
            storage,
            name: name.to_string(),
        }
    }

    pub fn main_path(&self) -> String {
        format!("{}.b789", self.name)
    }

    pub fn prev_path(&self) -> String {
        format!("{}.b789.prev", self.name)
    }

    pub fn tmp_path(&self) -> String {
        format!("{}.b789.tmp", self.name)
    }

    pub fn save(&mut self, airframe: &Airframe) -> Result<(), PersistError> {
        let text = airframe.to_document().to_text()?;
        self.save_text(text.as_bytes())
    }

    pub fn save_text(&mut self, bytes: &[u8]) -> Result<(), PersistError> {
        let r = self.save_inner(bytes);
        if r.is_err() {
            let tmp = self.tmp_path();
            let _ = self.storage.remove(&tmp);
        }
        r
    }

    fn save_inner(&mut self, bytes: &[u8]) -> Result<(), PersistError> {
        let (tmp, main, prev) = (self.tmp_path(), self.main_path(), self.prev_path());
        self.storage.write(&tmp, bytes)?;
        let back = self
            .storage
            .read(&tmp)?
            .ok_or_else(|| PersistError::NotFound(tmp.clone()))?;
        if back != bytes {
            return Err(PersistError::Io("verify after write failed".to_string()));
        }
        if let Some(existing) = self.storage.read(&main)?
            && decode_airframe(&existing).is_ok()
        {
            self.storage.copy(&main, &prev)?;
        }
        self.storage.rename(&tmp, &main)
    }

    pub fn load(&self) -> Result<Option<Loaded>, PersistError> {
        let main = self.storage.read(&self.main_path())?;
        let prev = self.storage.read(&self.prev_path())?;
        let primary_error = match &main {
            Some(bytes) => match decode_airframe(bytes) {
                Ok((airframe, migrated_steps)) => {
                    return Ok(Some(Loaded {
                        airframe,
                        source: LoadSource::Main,
                        migrated_steps,
                        primary_error: None,
                    }));
                }
                Err(e) => Some(e),
            },
            None => None,
        };
        match prev {
            Some(bytes) => match decode_airframe(&bytes) {
                Ok((airframe, migrated_steps)) => Ok(Some(Loaded {
                    airframe,
                    source: LoadSource::Previous,
                    migrated_steps,
                    primary_error,
                })),
                Err(e) => Err(primary_error.unwrap_or(e)),
            },
            None => match primary_error {
                Some(e) => Err(e),
                None => Ok(None),
            },
        }
    }
}
