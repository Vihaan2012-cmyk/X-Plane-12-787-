use crate::target::Target;
use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct AppliedAction {
    pub t_us: u64,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub value: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub action: String,
    pub target: String,
    pub signal: String,
    pub value: Value,
}

#[derive(Debug, Default)]
pub struct MockTarget {
    signals: BTreeMap<String, Value>,
    facts: BTreeMap<String, Value>,
    rules: Vec<Rule>,
    applied: Vec<AppliedAction>,
    steps: Vec<u64>,
    t_us: u64,
}

impl MockTarget {
    pub fn new() -> MockTarget {
        MockTarget::default()
    }

    pub fn with_signal(mut self, name: &str, v: Value) -> MockTarget {
        self.signals.insert(name.to_string(), v);
        self
    }

    pub fn with_fact(mut self, quantity: &str, v: Value) -> MockTarget {
        self.facts.insert(quantity.to_string(), v);
        self
    }

    pub fn with_rule(mut self, action: &str, target: &str, signal: &str, value: Value) -> MockTarget {
        self.rules.push(Rule {
            action: action.to_string(),
            target: target.to_string(),
            signal: signal.to_string(),
            value,
        });
        self
    }

    pub fn applied(&self) -> &[AppliedAction] {
        &self.applied
    }

    pub fn step_sizes(&self) -> &[u64] {
        &self.steps
    }

    pub fn time_us(&self) -> u64 {
        self.t_us
    }
}

impl Target for MockTarget {
    fn name(&self) -> &str {
        "mock"
    }

    fn apply(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        value: Option<&Value>,
    ) -> Result<(), String> {
        self.applied.push(AppliedAction {
            t_us: self.t_us,
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            value: value.cloned(),
        });
        if actor == "fault" && action == "reject" {
            return Err(format!("rejected {target}"));
        }
        for r in &self.rules {
            if r.action == action && r.target == target {
                self.signals.insert(r.signal.clone(), r.value.clone());
            }
        }
        if actor == "sim"
            && action == "initial"
            && let Some(v) = value
        {
            self.signals.insert(format!("initial.{target}"), v.clone());
        }
        Ok(())
    }

    fn read(&self, signal: &str) -> Option<Value> {
        if signal == "mock.time_us" {
            return Some(Value::Num(self.t_us as f64));
        }
        self.signals.get(signal).cloned()
    }

    fn step(&mut self, dt_us: u64) {
        self.t_us += dt_us;
        self.steps.push(dt_us);
    }

    fn fact(&self, quantity: &str, _condition: &str) -> Option<Value> {
        self.facts.get(quantity).cloned()
    }
}
