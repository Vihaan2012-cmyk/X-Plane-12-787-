use crate::model::load_scenario;
use std::path::{Path, PathBuf};
use sys_electrical::crew::Level;
use sys_electrical::inspector::{
    BreakerPage, EicasLevel, EicasMessage, InspectorData, PanelPage, RpduPage, SingleLinePage,
};
use sys_electrical::system::Electrical;

fn level(l: Level) -> EicasLevel {
    match l {
        Level::Warning => EicasLevel::Warning,
        Level::Caution => EicasLevel::Caution,
        Level::Advisory => EicasLevel::Advisory,
        Level::Status | Level::Memo => EicasLevel::Status,
    }
}

pub fn live_data(
    data_dir: &Path,
    scenario: Option<&Path>,
    settle_s: f64,
) -> Result<(InspectorData, String), String> {
    let mut sys = Electrical::new(data_dir)?;
    let mut now = 0u64;
    let mut title = String::from("cold and dark");
    if let Some(p) = scenario {
        let sc = load_scenario(p)?;
        title = sc.title.clone();
        for (k, v) in &sc.initial {
            let text = v.to_string();
            let _ = sys.action("sim", "set", k, Some(&text));
        }
        let mut steps = sc.steps.clone();
        steps.sort_by_key(|s| s.t_us);
        for s in &steps {
            if s.t_us > now {
                sys.step_idle(s.t_us - now);
                now = s.t_us;
            }
            let text = s.value.as_ref().map(|v| v.to_string());
            let _ = sys.action(&s.actor, &s.action, &s.target, text.as_deref());
        }
    }
    let settle = (settle_s.max(0.0) * 1e6) as u64;
    sys.step_idle(settle.max(1_000));
    now += settle.max(1_000);
    let snap = sys.net_snapshot().ok_or("no network solution yet")?;
    let mut data = InspectorData::from_net_snapshot(snap);
    data.sim_time_s = now as f64 / 1e6;
    data.eicas = sys
        .eicas_lines()
        .into_iter()
        .map(|l| EicasMessage {
            level: level(l.level),
            text: l.name,
        })
        .collect();
    Ok((data, title))
}

pub fn render_viewer(
    out_dir: &Path,
    data_dir: &Path,
    scenario: Option<&Path>,
) -> Result<Vec<PathBuf>, String> {
    let (mut data, title) = live_data(data_dir, scenario, 5.0)?;
    let demo = InspectorData::demo();
    if data.rpdus.is_empty() {
        data.rpdus = demo.rpdus.clone();
    }
    std::fs::create_dir_all(out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;
    let (rpdu_id, channel) = data
        .rpdus
        .first()
        .map(|r| (r.id.clone(), r.channels.first().map(|c| c.channel).unwrap_or(1)))
        .unwrap_or_else(|| (String::from("RPDU 71"), 1));
    let mut tablet = tablet::Tablet::new();
    tablet.register(Box::new(SingleLinePage::new(data.clone())));
    tablet.register(Box::new(PanelPage::new(data.clone())));
    tablet.register(Box::new(RpduPage::new(data.clone())));
    tablet.register(Box::new(BreakerPage::new(data.clone(), &rpdu_id, channel)));
    tablet.set_sim_time(data.sim_time_s);
    tablet.set_view(&title);
    let mut written = Vec::new();
    for (id, file) in [
        ("electrical", "viewer-single-line.png"),
        ("electrical.panels", "viewer-panels.png"),
        ("electrical.rpdu", "viewer-rpdu.png"),
        ("electrical.breaker", "viewer-breaker.png"),
    ] {
        tablet.select(id);
        let canvas = tablet.render();
        let path = out_dir.join(file);
        render::write_png(&path, canvas.width(), canvas.height(), canvas.rgba())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}
