#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub part_number: String,
    pub serial_number: String,
    pub software_pn: String,
    pub config_pn: String,
}

impl Identity {
    pub fn new(part_number: &str, serial_number: &str, software_pn: &str, config_pn: &str) -> Self {
        Self {
            part_number: part_number.to_string(),
            serial_number: serial_number.to_string(),
            software_pn: software_pn.to_string(),
            config_pn: config_pn.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityMismatch {
    PartNumber { found: String },
    SoftwarePn { found: String, expected: String },
    ConfigPn { found: String, expected: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expected {
    pub part_numbers: Vec<String>,
    pub software_pn: String,
    pub config_pn: String,
}

impl Expected {
    pub fn matching(identity: &Identity) -> Self {
        Self {
            part_numbers: vec![identity.part_number.clone()],
            software_pn: identity.software_pn.clone(),
            config_pn: identity.config_pn.clone(),
        }
    }

    pub fn check(&self, identity: &Identity) -> Vec<IdentityMismatch> {
        let mut out = Vec::new();
        if !self.part_numbers.is_empty() && !self.part_numbers.contains(&identity.part_number) {
            out.push(IdentityMismatch::PartNumber {
                found: identity.part_number.clone(),
            });
        }
        if self.software_pn != identity.software_pn {
            out.push(IdentityMismatch::SoftwarePn {
                found: identity.software_pn.clone(),
                expected: self.software_pn.clone(),
            });
        }
        if self.config_pn != identity.config_pn {
            out.push(IdentityMismatch::ConfigPn {
                found: identity.config_pn.clone(),
                expected: self.config_pn.clone(),
            });
        }
        out
    }
}
