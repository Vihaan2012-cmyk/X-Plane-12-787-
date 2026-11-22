use crate::codec::{Decoder, Encoder};
use crate::common::{PartCommon, PartId};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub trait Part {
    fn common(&self) -> &PartCommon;
    fn common_mut(&mut self) -> &mut PartCommon;
}

pub trait Persist: Sized {
    fn encode(&self, e: &mut Encoder);
    fn decode(d: &mut Decoder<'_>) -> Option<Self>;
}

impl Part for PartCommon {
    fn common(&self) -> &PartCommon {
        self
    }

    fn common_mut(&mut self) -> &mut PartCommon {
        self
    }
}

impl Persist for PartCommon {
    fn encode(&self, e: &mut Encoder) {
        PartCommon::encode(self, e);
    }

    fn decode(d: &mut Decoder<'_>) -> Option<Self> {
        PartCommon::decode(d)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreError {
    DuplicateName(String),
    UnknownId(PartId),
    Parse(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::DuplicateName(n) => write!(f, "duplicate part name {n}"),
            StoreError::UnknownId(id) => write!(f, "unknown part id {}", id.0),
            StoreError::Parse(m) => write!(f, "snapshot parse error: {m}"),
        }
    }
}

impl std::error::Error for StoreError {}

type Keys = (String, String, String);

#[derive(Clone, Debug)]
pub struct Store<T: Part> {
    slots: Vec<Option<T>>,
    names: BTreeMap<String, PartId>,
    kinds: BTreeMap<String, BTreeSet<PartId>>,
    zones: BTreeMap<String, BTreeSet<PartId>>,
    keys: BTreeMap<PartId, Keys>,
    dirty: BTreeSet<PartId>,
    asleep: BTreeSet<PartId>,
}

impl<T: Part> Default for Store<T> {
    fn default() -> Self {
        Self::new()
    }
}

const MAGIC: &str = "b789-store-v1";

impl<T: Part> Store<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            names: BTreeMap::new(),
            kinds: BTreeMap::new(),
            zones: BTreeMap::new(),
            keys: BTreeMap::new(),
            dirty: BTreeSet::new(),
            asleep: BTreeSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn next_id(&self) -> PartId {
        PartId(self.slots.len() as u32)
    }

    fn index(&mut self, id: PartId, keys: Keys) {
        self.names.insert(keys.0.clone(), id);
        self.kinds.entry(keys.1.clone()).or_default().insert(id);
        self.zones.entry(keys.2.clone()).or_default().insert(id);
        self.keys.insert(id, keys);
    }

    fn unindex(&mut self, id: PartId) {
        if let Some((name, kind, zone)) = self.keys.remove(&id) {
            self.names.remove(&name);
            if let Some(set) = self.kinds.get_mut(&kind) {
                set.remove(&id);
                if set.is_empty() {
                    self.kinds.remove(&kind);
                }
            }
            if let Some(set) = self.zones.get_mut(&zone) {
                set.remove(&id);
                if set.is_empty() {
                    self.zones.remove(&zone);
                }
            }
        }
    }

    fn keys_of(part: &T) -> Keys {
        let c = part.common();
        (c.name.clone(), c.kind.clone(), c.zone.clone())
    }

    pub fn insert(&mut self, part: T) -> Result<PartId, StoreError> {
        let keys = Self::keys_of(&part);
        if self.names.contains_key(&keys.0) {
            return Err(StoreError::DuplicateName(keys.0));
        }
        let id = self.next_id();
        self.slots.push(Some(part));
        self.index(id, keys);
        self.dirty.insert(id);
        Ok(id)
    }

    pub fn remove(&mut self, id: PartId) -> Option<T> {
        let part = self.slots.get_mut(id.0 as usize)?.take()?;
        self.unindex(id);
        self.dirty.remove(&id);
        self.asleep.remove(&id);
        Some(part)
    }

    pub fn contains(&self, id: PartId) -> bool {
        matches!(self.slots.get(id.0 as usize), Some(Some(_)))
    }

    pub fn get(&self, id: PartId) -> Option<&T> {
        self.slots.get(id.0 as usize)?.as_ref()
    }

    pub fn get_mut(&mut self, id: PartId) -> Option<&mut T> {
        if !self.contains(id) {
            return None;
        }
        self.dirty.insert(id);
        self.asleep.remove(&id);
        self.slots.get_mut(id.0 as usize)?.as_mut()
    }

    pub fn id_of(&self, name: &str) -> Option<PartId> {
        self.names.get(name).copied()
    }

    pub fn by_name(&self, name: &str) -> Option<&T> {
        self.get(self.id_of(name)?)
    }

    pub fn by_name_mut(&mut self, name: &str) -> Option<&mut T> {
        let id = self.id_of(name)?;
        self.get_mut(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = PartId> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_some())
            .map(|(i, _)| PartId(i as u32))
    }

    pub fn iter(&self) -> impl Iterator<Item = (PartId, &T)> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|p| (PartId(i as u32), p)))
    }

    pub fn ids_of_kind(&self, kind: &str) -> Vec<PartId> {
        self.kinds
            .get(kind)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn ids_in_zone(&self, zone: &str) -> Vec<PartId> {
        self.zones
            .get(zone)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn kind_names(&self) -> Vec<&str> {
        self.kinds.keys().map(String::as_str).collect()
    }

    pub fn zone_names(&self) -> Vec<&str> {
        self.zones.keys().map(String::as_str).collect()
    }

    pub fn rekey(&mut self, id: PartId) -> Result<(), StoreError> {
        let new_keys = match self.get(id) {
            Some(p) => Self::keys_of(p),
            None => return Err(StoreError::UnknownId(id)),
        };
        if let Some(existing) = self.names.get(&new_keys.0)
            && *existing != id
        {
            return Err(StoreError::DuplicateName(new_keys.0));
        }
        if self.keys.get(&id) == Some(&new_keys) {
            return Ok(());
        }
        self.unindex(id);
        self.index(id, new_keys);
        self.dirty.insert(id);
        Ok(())
    }

    pub fn mark_dirty(&mut self, id: PartId) {
        if self.contains(id) {
            self.dirty.insert(id);
        }
    }

    pub fn is_dirty(&self, id: PartId) -> bool {
        self.dirty.contains(&id)
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    pub fn take_dirty(&mut self) -> Vec<PartId> {
        std::mem::take(&mut self.dirty).into_iter().collect()
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    pub fn sleep(&mut self, id: PartId) {
        if self.contains(id) {
            self.asleep.insert(id);
        }
    }

    pub fn wake(&mut self, id: PartId) {
        self.asleep.remove(&id);
    }

    pub fn is_asleep(&self, id: PartId) -> bool {
        self.asleep.contains(&id)
    }

    pub fn wake_zone(&mut self, zone: &str) -> usize {
        let ids = self.ids_in_zone(zone);
        let mut n = 0;
        for id in ids {
            if self.asleep.remove(&id) {
                n += 1;
            }
        }
        n
    }

    pub fn wake_kind(&mut self, kind: &str) -> usize {
        let ids = self.ids_of_kind(kind);
        let mut n = 0;
        for id in ids {
            if self.asleep.remove(&id) {
                n += 1;
            }
        }
        n
    }

    pub fn wake_all(&mut self) {
        self.asleep.clear();
    }

    pub fn awake_ids(&self) -> Vec<PartId> {
        self.ids().filter(|id| !self.asleep.contains(id)).collect()
    }

    pub fn awake_count(&self) -> usize {
        self.len() - self.asleep.len()
    }

    pub fn sleeping_count(&self) -> usize {
        self.asleep.len()
    }
}

impl<T: Part + Persist> Store<T> {
    pub fn snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str(MAGIC);
        out.push('\n');
        out.push_str(&format!("next\t{}\n", self.slots.len()));
        for (id, part) in self.iter() {
            let mut e = Encoder::new();
            e.u64(u64::from(id.0))
                .bool(self.dirty.contains(&id))
                .bool(self.asleep.contains(&id));
            part.encode(&mut e);
            out.push_str("part\t");
            out.push_str(&e.finish());
            out.push('\n');
        }
        out.push_str(&format!("end\t{}\n", self.len()));
        out
    }

    pub fn restore(text: &str) -> Result<Self, StoreError> {
        let err = |m: &str| StoreError::Parse(m.to_string());
        let mut lines = text.lines();
        if lines.next() != Some(MAGIC) {
            return Err(err("bad magic"));
        }
        let next_line = lines.next().ok_or_else(|| err("missing next"))?;
        let next: usize = next_line
            .strip_prefix("next\t")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| err("bad next"))?;
        let mut store = Store::new();
        store.slots.resize_with(next, || None);
        let mut ended = false;
        for line in lines {
            if let Some(n) = line.strip_prefix("end\t") {
                let n: usize = n.parse().map_err(|_| err("bad end"))?;
                if n != store.len() {
                    return Err(err("count mismatch"));
                }
                ended = true;
                break;
            }
            let body = line.strip_prefix("part\t").ok_or_else(|| err("bad line"))?;
            let mut d = Decoder::new(body);
            let id = d.u64().ok_or_else(|| err("bad id"))?;
            let dirty = d.bool().ok_or_else(|| err("bad dirty"))?;
            let asleep = d.bool().ok_or_else(|| err("bad asleep"))?;
            let part = T::decode(&mut d).ok_or_else(|| err("bad part"))?;
            let id = PartId(u32::try_from(id).map_err(|_| err("id range"))?);
            if id.0 as usize >= next || store.slots[id.0 as usize].is_some() {
                return Err(err("bad slot"));
            }
            let keys = Self::keys_of(&part);
            if store.names.contains_key(&keys.0) {
                return Err(StoreError::DuplicateName(keys.0));
            }
            store.slots[id.0 as usize] = Some(part);
            store.index(id, keys);
            if dirty {
                store.dirty.insert(id);
            }
            if asleep {
                store.asleep.insert(id);
            }
        }
        if !ended {
            return Err(err("truncated"));
        }
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Grade;

    fn part(name: &str, kind: &str, zone: &str) -> PartCommon {
        PartCommon::new(name, kind, zone)
            .with_serial(&format!("SN-{name}"))
            .with_grade(Grade::Derived, "rule:test")
    }

    fn filled() -> Store<PartCommon> {
        let mut s = Store::new();
        s.insert(part("a", "relay", "119")).unwrap();
        s.insert(part("b", "relay", "120")).unwrap();
        s.insert(part("c", "bat", "119")).unwrap();
        s
    }

    #[test]
    fn insert_get_and_ids_stable() {
        let s = filled();
        assert_eq!(s.len(), 3);
        assert_eq!(s.get(PartId(0)).unwrap().name, "a");
        assert_eq!(s.get(PartId(2)).unwrap().name, "c");
        assert!(s.get(PartId(3)).is_none());
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut s = filled();
        let e = s.insert(part("a", "x", "y")).unwrap_err();
        assert_eq!(e, StoreError::DuplicateName("a".to_string()));
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn lookup_by_name() {
        let s = filled();
        assert_eq!(s.id_of("b"), Some(PartId(1)));
        assert_eq!(s.by_name("c").unwrap().kind, "bat");
        assert!(s.by_name("zz").is_none());
    }

    #[test]
    fn iteration_in_id_order() {
        let s = filled();
        let ids: Vec<u32> = s.ids().map(|i| i.0).collect();
        assert_eq!(ids, vec![0, 1, 2]);
        let names: Vec<&str> = s.iter().map(|(_, p)| p.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn kind_and_zone_index() {
        let s = filled();
        assert_eq!(s.ids_of_kind("relay"), vec![PartId(0), PartId(1)]);
        assert_eq!(s.ids_in_zone("119"), vec![PartId(0), PartId(2)]);
        assert!(s.ids_of_kind("none").is_empty());
        assert_eq!(s.kind_names(), vec!["bat", "relay"]);
        assert_eq!(s.zone_names(), vec!["119", "120"]);
    }

    #[test]
    fn remove_keeps_ids_stable_and_never_reuses() {
        let mut s = filled();
        let removed = s.remove(PartId(1)).unwrap();
        assert_eq!(removed.name, "b");
        assert!(s.get(PartId(1)).is_none());
        assert_eq!(s.id_of("b"), None);
        assert_eq!(s.ids_of_kind("relay"), vec![PartId(0)]);
        let id = s.insert(part("d", "relay", "120")).unwrap();
        assert_eq!(id, PartId(3));
        assert_eq!(s.get(PartId(2)).unwrap().name, "c");
        assert!(s.remove(PartId(1)).is_none());
        assert!(!s.zone_names().is_empty());
    }

    #[test]
    fn rekey_updates_indexes() {
        let mut s = filled();
        s.get_mut(PartId(0)).unwrap().zone = "200".to_string();
        s.get_mut(PartId(0)).unwrap().name = "a2".to_string();
        s.rekey(PartId(0)).unwrap();
        assert_eq!(s.id_of("a2"), Some(PartId(0)));
        assert_eq!(s.id_of("a"), None);
        assert_eq!(s.ids_in_zone("200"), vec![PartId(0)]);
        assert_eq!(s.ids_in_zone("119"), vec![PartId(2)]);
    }

    #[test]
    fn rekey_duplicate_rejected() {
        let mut s = filled();
        s.get_mut(PartId(0)).unwrap().name = "b".to_string();
        assert_eq!(
            s.rekey(PartId(0)),
            Err(StoreError::DuplicateName("b".to_string()))
        );
        assert_eq!(s.rekey(PartId(9)), Err(StoreError::UnknownId(PartId(9))));
    }

    #[test]
    fn dirty_tracking() {
        let mut s = filled();
        assert_eq!(s.dirty_count(), 3);
        let d = s.take_dirty();
        assert_eq!(d, vec![PartId(0), PartId(1), PartId(2)]);
        assert_eq!(s.dirty_count(), 0);
        s.get_mut(PartId(1)).unwrap().add_wear(0.1);
        assert!(s.is_dirty(PartId(1)));
        assert!(!s.is_dirty(PartId(0)));
        s.get(PartId(0));
        assert!(!s.is_dirty(PartId(0)));
        s.mark_dirty(PartId(0));
        assert_eq!(s.take_dirty(), vec![PartId(0), PartId(1)]);
        s.mark_dirty(PartId(77));
        assert_eq!(s.dirty_count(), 0);
    }

    #[test]
    fn sleep_and_wake() {
        let mut s = filled();
        s.sleep(PartId(0));
        s.sleep(PartId(2));
        assert!(s.is_asleep(PartId(0)));
        assert_eq!(s.awake_ids(), vec![PartId(1)]);
        assert_eq!(s.awake_count(), 1);
        assert_eq!(s.sleeping_count(), 2);
        s.get_mut(PartId(0)).unwrap();
        assert!(!s.is_asleep(PartId(0)));
        assert_eq!(s.wake_zone("119"), 1);
        assert_eq!(s.awake_count(), 3);
        s.sleep(PartId(0));
        s.sleep(PartId(1));
        assert_eq!(s.wake_kind("relay"), 2);
        s.sleep(PartId(2));
        s.wake_all();
        assert_eq!(s.sleeping_count(), 0);
        s.sleep(PartId(50));
        assert_eq!(s.sleeping_count(), 0);
    }

    #[test]
    fn snapshot_restore_round_trip() {
        let mut s = filled();
        s.get_mut(PartId(2)).unwrap().draw_tolerance("cap", 0.02, 3.0);
        s.get_mut(PartId(2)).unwrap().temperature_k = 311.25;
        s.remove(PartId(1));
        s.clear_dirty();
        s.mark_dirty(PartId(2));
        s.sleep(PartId(0));
        let text = s.snapshot();
        let r = Store::<PartCommon>::restore(&text).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r.next_id(), PartId(3));
        for (id, p) in s.iter() {
            assert_eq!(r.get(id).unwrap(), p);
        }
        assert!(r.is_asleep(PartId(0)));
        assert!(r.is_dirty(PartId(2)));
        assert!(!r.is_dirty(PartId(0)));
        assert_eq!(r.ids_in_zone("119"), vec![PartId(0), PartId(2)]);
        assert_eq!(r.snapshot(), text);
    }

    #[test]
    fn restore_rejects_corruption() {
        let s = filled();
        let text = s.snapshot();
        assert!(Store::<PartCommon>::restore("junk").is_err());
        let truncated: String = text.lines().take(3).collect::<Vec<_>>().join("\n");
        assert!(Store::<PartCommon>::restore(&truncated).is_err());
        let bad = text.replace("end\t3", "end\t4");
        assert!(Store::<PartCommon>::restore(&bad).is_err());
        let dup = text.replacen("part\t1\t", "part\t0\t", 1);
        assert!(Store::<PartCommon>::restore(&dup).is_err());
    }

    #[test]
    fn snapshot_deterministic() {
        assert_eq!(filled().snapshot(), filled().snapshot());
    }

    #[test]
    fn empty_store_round_trip() {
        let s = Store::<PartCommon>::new();
        let r = Store::<PartCommon>::restore(&s.snapshot()).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn error_display() {
        assert!(StoreError::UnknownId(PartId(4)).to_string().contains('4'));
    }
}
