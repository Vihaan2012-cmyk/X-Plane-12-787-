mod breaker;
mod data;
mod demo;
mod eicas;
mod gfx;
mod label;
mod live;
mod overhead;
mod panel;
mod protection;
mod rpdu;
mod single_line;
mod synoptic;
mod topo;
mod ui;

pub use breaker::BreakerPage;
pub use data::{
    BatteryView, BusView, ChannelState, ChannelView, ContactorView, ConverterView, EicasLevel, EicasMessage,
    InspectorData, ProtectionRow, RpduView, SourceView,
};
pub use eicas::{CbCommand, CbRow, CbicView, EicasPage};
pub use live::seed_cbic;
pub use overhead::{OverheadPage, OverheadState};
pub use panel::PanelPage;
pub use protection::ProtectionPage;
pub use rpdu::RpduPage;
pub use single_line::{Selection, SingleLinePage};
pub use synoptic::SynopticPage;
pub use ui::PanelAction;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ui_tests;
