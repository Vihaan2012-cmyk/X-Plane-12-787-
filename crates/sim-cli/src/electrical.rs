use crate::target::{Target, unbound};
use crate::value::Value;
use std::path::Path;
use sys_electrical::system::{Electrical, SignalValue};

pub struct ElectricalTarget {
    sys: Electrical,
}

impl ElectricalTarget {
    pub fn new(data_dir: &Path) -> Result<ElectricalTarget, String> {
        Ok(ElectricalTarget {
            sys: Electrical::new(data_dir)?,
        })
    }
}

fn scalar_text(value: Option<&Value>) -> Option<String> {
    value.map(|v| v.to_string())
}

impl Target for ElectricalTarget {
    fn name(&self) -> &str {
        "electrical"
    }

    fn apply(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        value: Option<&Value>,
    ) -> Result<(), String> {
        let text = scalar_text(value);
        let action = if actor == "sim" && action == "initial" {
            "set"
        } else {
            action
        };
        self.sys
            .action(actor, action, target, text.as_deref())
            .map_err(unbound)
    }

    fn read(&self, signal: &str) -> Option<Value> {
        let name = signal.strip_prefix("elec.").unwrap_or(signal);
        for n in [signal, name] {
            if let Some(v) = self.sys.value(n) {
                return Some(match v {
                    SignalValue::Bool(b) => Value::Bool(b),
                    SignalValue::Num(x) => Value::Num(x),
                    SignalValue::Text(s) => Value::Str(s),
                });
            }
        }
        None
    }

    fn step(&mut self, dt_us: u64) {
        self.sys.step_idle(dt_us);
    }

    fn adaptive(&self) -> bool {
        true
    }

    fn set_fine(&mut self, fine: bool) {
        self.sys.set_fine_stepping(fine);
    }
}
