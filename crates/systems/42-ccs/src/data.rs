use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct RdcDef {
    pub id: String,
    pub location: String,
    pub pri_bus: String,
    pub pri_source: String,
    pub sec_bus: String,
    pub sec_source: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CcsData {
    pub hosted: BTreeMap<String, Vec<String>>,
    pub rdcs: Vec<RdcDef>,
}

fn rows(path: &Path) -> Result<Vec<Vec<String>>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|c| c.trim().to_string()).collect())
        .collect())
}

impl CcsData {
    pub fn load(dir: &Path) -> Result<CcsData, String> {
        let mut hosted = BTreeMap::new();
        for r in rows(&dir.join("hosted-apps.csv"))? {
            if r.len() < 2 {
                continue;
            }
            let gpms = r[1].split_whitespace().map(|s| s.to_string()).collect();
            hosted.insert(r[0].clone(), gpms);
        }
        let mut rdcs = Vec::new();
        for r in rows(&dir.join("rdcs.csv"))? {
            if r.len() < 9 {
                continue;
            }
            rdcs.push(RdcDef {
                id: r[0].clone(),
                location: r[1].clone(),
                pri_bus: r[2].clone(),
                pri_source: r[3].clone(),
                sec_bus: r[6].clone(),
                sec_source: r[7].clone(),
            });
        }
        Ok(CcsData { hosted, rdcs })
    }
}
