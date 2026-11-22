use crate::model::Op;
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Unbound,
    Warn,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::Unbound => "UNBOUND",
            Status::Warn => "WARN",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Pass,
    Fail(String),
    Unbound,
}

pub fn evaluate(op: Op, expected: Option<&Value>, tol: f64, actual: Option<&Value>) -> Verdict {
    let Some(actual) = actual else {
        return Verdict::Unbound;
    };
    match op {
        Op::True | Op::Present => match actual.as_bool() {
            Some(true) => Verdict::Pass,
            Some(false) => Verdict::Fail(format!("expected true, got {actual}")),
            None => Verdict::Fail(format!("expected true, got non-boolean {actual}")),
        },
        Op::False | Op::Absent => match actual.as_bool() {
            Some(false) => Verdict::Pass,
            Some(true) => Verdict::Fail(format!("expected false, got {actual}")),
            None => Verdict::Fail(format!("expected false, got non-boolean {actual}")),
        },
        Op::Eq => compare_eq(expected, tol, actual),
        Op::Approx | Op::Within => compare_band(expected, tol, actual),
        Op::Lt => compare_order(expected, actual, true),
        Op::Gt => compare_order(expected, actual, false),
    }
}

fn compare_eq(expected: Option<&Value>, tol: f64, actual: &Value) -> Verdict {
    let Some(expected) = expected else {
        return Verdict::Fail("no expected value".to_string());
    };
    match (expected, actual) {
        (Value::Str(a), Value::Str(b)) => {
            if a == b {
                Verdict::Pass
            } else {
                Verdict::Fail(format!("expected {a}, got {b}"))
            }
        }
        (Value::Bool(a), Value::Bool(b)) => {
            if a == b {
                Verdict::Pass
            } else {
                Verdict::Fail(format!("expected {a}, got {b}"))
            }
        }
        (e, a) => match (e.as_f64(), a.as_f64()) {
            (Some(e), Some(a)) => {
                if (a - e).abs() <= tol {
                    Verdict::Pass
                } else {
                    Verdict::Fail(format!("expected {e} (tol {tol}), got {a}"))
                }
            }
            _ => Verdict::Fail(format!("type mismatch: expected {e}, got {a}")),
        },
    }
}

fn compare_band(expected: Option<&Value>, tol: f64, actual: &Value) -> Verdict {
    match (expected.and_then(Value::as_f64), actual.as_f64()) {
        (Some(e), Some(a)) => {
            if (a - e).abs() <= tol {
                Verdict::Pass
            } else {
                Verdict::Fail(format!("expected {e} +/- {tol}, got {a}"))
            }
        }
        _ => Verdict::Fail(format!("non-numeric comparison: got {actual}")),
    }
}

fn compare_order(expected: Option<&Value>, actual: &Value, less: bool) -> Verdict {
    match (expected.and_then(Value::as_f64), actual.as_f64()) {
        (Some(e), Some(a)) => {
            let ok = if less { a < e } else { a > e };
            if ok {
                Verdict::Pass
            } else {
                Verdict::Fail(format!("expected {} {e}, got {a}", if less { "<" } else { ">" }))
            }
        }
        _ => Verdict::Fail(format!("non-numeric comparison: got {actual}")),
    }
}

pub fn to_status(verdict: &Verdict, approx: bool) -> Status {
    match verdict {
        Verdict::Pass => Status::Pass,
        Verdict::Unbound => Status::Unbound,
        Verdict::Fail(_) => {
            if approx {
                Status::Warn
            } else {
                Status::Fail
            }
        }
    }
}
