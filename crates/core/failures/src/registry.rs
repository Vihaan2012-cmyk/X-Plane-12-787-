use crate::failure::FailureMode;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateMode { part_type: String, mode: String },
}

#[derive(Clone, Debug, Default)]
pub struct Registry {
    types: BTreeMap<String, BTreeMap<String, FailureMode>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, part_type: &str, mode: FailureMode) -> Result<(), RegistryError> {
        let modes = self.types.entry(part_type.to_string()).or_default();
        if modes.contains_key(&mode.name) {
            return Err(RegistryError::DuplicateMode {
                part_type: part_type.to_string(),
                mode: mode.name,
            });
        }
        modes.insert(mode.name.clone(), mode);
        Ok(())
    }

    pub fn add(&mut self, part_type: &str, mode: &str, observables: &[&str]) -> Result<(), RegistryError> {
        let mut m = FailureMode::new(mode);
        for o in observables {
            m = m.observable_via(o);
        }
        self.register(part_type, m)
    }

    pub fn supports(&self, part_type: &str, mode: &str) -> bool {
        self.types.get(part_type).is_some_and(|m| m.contains_key(mode))
    }

    pub fn mode(&self, part_type: &str, mode: &str) -> Option<&FailureMode> {
        self.types.get(part_type)?.get(mode)
    }

    pub fn modes_for(&self, part_type: &str) -> Vec<&FailureMode> {
        self.types
            .get(part_type)
            .map(|m| m.values().collect())
            .unwrap_or_default()
    }

    pub fn part_types(&self) -> Vec<&str> {
        self.types.keys().map(String::as_str).collect()
    }

    pub fn total_modes(&self) -> usize {
        self.types.values().map(BTreeMap::len).sum()
    }

    pub fn all(&self) -> Vec<(&str, &FailureMode)> {
        let mut out = Vec::new();
        for (t, modes) in &self.types {
            for m in modes.values() {
                out.push((t.as_str(), m));
            }
        }
        out
    }

    pub fn unobservable(&self) -> Vec<(String, String)> {
        self.all()
            .into_iter()
            .filter(|(_, m)| !m.is_observable())
            .map(|(t, m)| (t.to_string(), m.name.clone()))
            .collect()
    }

    pub fn audit(&self) -> Result<(), Vec<String>> {
        let bad: Vec<String> = self
            .unobservable()
            .into_iter()
            .map(|(t, m)| format!("{t}/{m}: no observable"))
            .collect();
        if bad.is_empty() { Ok(()) } else { Err(bad) }
    }

    pub fn audit_known_observables(&self, known: &[&str]) -> Result<(), Vec<String>> {
        let mut bad = Vec::new();
        for (t, m) in self.all() {
            if !m.is_observable() {
                bad.push(format!("{t}/{}: no observable", m.name));
            }
            for o in &m.observables {
                if !known.contains(&o.as_str()) {
                    bad.push(format!("{t}/{}: unknown observable {o}", m.name));
                }
            }
        }
        if bad.is_empty() { Ok(()) } else { Err(bad) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Registry {
        let mut r = Registry::new();
        r.add("relay", "stuck-open", &["contact-voltage"]).unwrap();
        r.add("relay", "stuck-closed", &["contact-voltage", "bite"])
            .unwrap();
        r.add("battery", "cell-short", &["terminal-voltage"]).unwrap();
        r
    }

    #[test]
    fn supports_and_lists() {
        let r = sample();
        assert!(r.supports("relay", "stuck-open"));
        assert!(!r.supports("relay", "cell-short"));
        assert!(!r.supports("nothing", "x"));
        assert_eq!(r.part_types(), vec!["battery", "relay"]);
        assert_eq!(r.total_modes(), 3);
        let names: Vec<&str> = r.modes_for("relay").iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, vec!["stuck-closed", "stuck-open"]);
        assert!(r.modes_for("zzz").is_empty());
        assert_eq!(r.mode("relay", "stuck-closed").unwrap().observables.len(), 2);
    }

    #[test]
    fn duplicate_rejected() {
        let mut r = sample();
        let e = r.add("relay", "stuck-open", &["x"]).unwrap_err();
        assert_eq!(
            e,
            RegistryError::DuplicateMode {
                part_type: "relay".to_string(),
                mode: "stuck-open".to_string()
            }
        );
    }

    #[test]
    fn audit_flags_unobservable() {
        let mut r = sample();
        assert!(r.audit().is_ok());
        r.add("relay", "silent", &[]).unwrap();
        assert_eq!(
            r.unobservable(),
            vec![("relay".to_string(), "silent".to_string())]
        );
        let e = r.audit().unwrap_err();
        assert_eq!(e, vec!["relay/silent: no observable".to_string()]);
    }

    #[test]
    fn audit_known_observables() {
        let r = sample();
        assert!(
            r.audit_known_observables(&["contact-voltage", "bite", "terminal-voltage"])
                .is_ok()
        );
        let e = r
            .audit_known_observables(&["contact-voltage", "terminal-voltage"])
            .unwrap_err();
        assert_eq!(e.len(), 1);
        assert!(e[0].contains("bite"));
    }

    #[test]
    fn all_in_deterministic_order() {
        let r = sample();
        let order: Vec<String> = r.all().iter().map(|(t, m)| format!("{t}/{}", m.name)).collect();
        assert_eq!(
            order,
            vec!["battery/cell-short", "relay/stuck-closed", "relay/stuck-open"]
        );
    }
}
