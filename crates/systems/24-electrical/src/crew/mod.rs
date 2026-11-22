pub mod eicas;
pub mod inputs;
pub mod panel;
pub mod switch;
pub mod synoptic;

pub use eicas::{EicasElec, EicasLine, Level, MessageDef, MsgId};
pub use inputs::{CrewInputs, FlightPhase, classify_phase};
pub use panel::{ApuSelector, OverheadPanel, PanelLamps, PanelOutputs};
pub use switch::{Lamp, LampColor, Switch, SwitchKind};
pub use synoptic::{Colour, ElementState, SynopticPage, build_synoptic};
