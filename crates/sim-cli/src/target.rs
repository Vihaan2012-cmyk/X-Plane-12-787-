use crate::value::Value;

pub trait Target {
    fn name(&self) -> &str;
    fn apply(&mut self, actor: &str, action: &str, target: &str, value: Option<&Value>)
    -> Result<(), String>;
    fn read(&self, signal: &str) -> Option<Value>;
    fn step(&mut self, dt_us: u64);
    fn adaptive(&self) -> bool {
        false
    }
    fn set_fine(&mut self, _fine: bool) {}
    fn fact(&self, quantity: &str, _condition: &str) -> Option<Value> {
        self.read(quantity)
    }
}

pub const UNBOUND_PREFIX: &str = "unbound:";

pub fn unbound(msg: impl AsRef<str>) -> String {
    format!("{UNBOUND_PREFIX} {}", msg.as_ref())
}
