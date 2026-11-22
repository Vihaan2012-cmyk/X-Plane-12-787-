use super::data::{EicasLevel, EicasMessage, InspectorData};
use crate::control::cbic::{BreakerKind, CbState};
use crate::crew::Level;
use crate::system::Electrical;

fn level(l: Level) -> EicasLevel {
    match l {
        Level::Warning => EicasLevel::Warning,
        Level::Caution => EicasLevel::Caution,
        Level::Advisory => EicasLevel::Advisory,
        Level::Status | Level::Memo => EicasLevel::Status,
    }
}

impl InspectorData {
    pub fn from_system(sys: &Electrical) -> InspectorData {
        let mut data = match sys.net_snapshot() {
            Some(s) => InspectorData::from_net_snapshot(s),
            None => InspectorData::default(),
        };
        data.sim_time_s = sys.now_us() as f64 / 1e6;
        data.eicas = sys
            .eicas_lines()
            .into_iter()
            .map(|l| EicasMessage {
                level: level(l.level),
                text: l.name,
            })
            .collect();
        data.protections = sys.protection_rows();
        data.rpdus = sys.pdu().inspector_rpdus();
        if data.rpdus.is_empty() {
            data.rpdus = InspectorData::demo().rpdus;
        }
        data
    }
}

pub fn seed_cbic(sys: &mut Electrical, breakers_csv: &str, ecb_limit: usize, tcb_limit: usize) -> usize {
    let mut ecb = 0;
    let mut tcb = 0;
    let mut added = 0;
    for line in breakers_csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 3 {
            continue;
        }
        let Some(id) = f[0].get(2..).and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let kind = match f[2] {
            "ECB" if ecb < ecb_limit => {
                ecb += 1;
                BreakerKind::Ecb
            }
            "TCB" if tcb < tcb_limit => {
                tcb += 1;
                BreakerKind::Tcb
            }
            _ => continue,
        };
        if sys.cbic().app.breaker(id).is_none() {
            sys.cbic_mut().app.add_breaker(id, f[1], kind, CbState::Closed);
            added += 1;
        }
    }
    added
}
