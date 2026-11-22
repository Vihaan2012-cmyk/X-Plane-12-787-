#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtSupply {
    pub connected: bool,
    pub v_ll: f64,
    pub hz: f64,
    pub kva: f64,
    pub phase_seq_abc: bool,
    pub pin_ef_ok: bool,
}

impl Default for ExtSupply {
    fn default() -> Self {
        Self {
            connected: false,
            v_ll: 115.0,
            hz: 400.0,
            kva: 90.0,
            phase_seq_abc: true,
            pin_ef_ok: true,
        }
    }
}

impl ExtSupply {
    pub fn cart() -> Self {
        Self {
            connected: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrewCommand {
    Battery(bool),
    ApuGen(usize, bool),
    GenCtrl(usize, bool),
    GenCtrlCycle(usize),
    ExtPwrFwd(usize),
    ExtPwrAft,
    DriveDisc(usize),
    GroundService,
    ApuStart,
    CabinUtil(bool),
    IfeSeats(bool),
    LampTest(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElectricalInputs {
    pub crew: Vec<CrewCommand>,
    pub engine_rpm: [f64; 2],
    pub engine_running: [bool; 2],
    pub apu_rpm: f64,
    pub apu_running: bool,
    pub ext: [ExtSupply; 3],
    pub on_ground: bool,
    pub ground_speed_kt: f64,
    pub radio_alt_ft: f64,
    pub rat_deployed: bool,
    pub rat_rpm: f64,
    pub ambient_c: f64,
    pub doors_open: bool,
    pub alt_ft: f64,
    pub apu_battery_connected: bool,
    pub ext_load_kva: [f64; 3],
    pub cbic_session: Option<u32>,
    pub galley_power: bool,
    pub cargo_heat: bool,
}

impl Default for ElectricalInputs {
    fn default() -> Self {
        Self {
            crew: Vec::new(),
            engine_rpm: [0.0; 2],
            engine_running: [false; 2],
            apu_rpm: 0.0,
            apu_running: false,
            ext: [ExtSupply::default(); 3],
            on_ground: true,
            ground_speed_kt: 0.0,
            radio_alt_ft: 0.0,
            rat_deployed: false,
            rat_rpm: 0.0,
            ambient_c: 15.0,
            doors_open: false,
            alt_ft: 0.0,
            apu_battery_connected: true,
            ext_load_kva: [0.0; 3],
            cbic_session: None,
            galley_power: false,
            cargo_heat: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Faults {
    pub gen_loss: [bool; 4],
    pub asg_loss: [bool; 2],
    pub bpcu_fail: [bool; 2],
    pub bus_fail: [bool; 4],
    pub cbic_tcb_rejected: bool,
}
