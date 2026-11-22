# API: inspector (b789-sys-electrical, feature `inspector`)

Sim-tablet pages for ATA 24, drawn with `render`. Pages implement `render::page::Page`; the tablet only hosts them. Virtual drawing space is 1000x672, scaled to the given area; `click` takes area-relative pixels.

## Data
- `InspectorData { sim_time_s, buses, contactors, sources, converters, batteries, rpdus, eicas }`, lookups `bus`, `contactor`, `source`, `converter`, `battery`, `rpdu`, `bus_live`, `closed`.
- `BusView { name, voltage_v, live, freq_hz, load_a, fault }`, `ContactorView { name, closed, current_a, fault, fault_text }`, `SourceView { name, online, voltage_v, freq_hz, load_a, rpm, status, fault }`, `ConverterView`, `BatteryView`.
- `RpduView { id, location, input_v, input_a, powered, channels }`, `ChannelView { channel, rating_a, load, state, current_a, voltage_out_v, i2t_pct, arc_events, instant_trip_a, temp_c, trips_life, wire, bundle, wire_temp_c, history, trace }`, `ChannelState { Off, On, Shed, Tripped, TrippedArc }`.
- `EicasMessage { level, text }`, `EicasLevel { Warning, Caution, Advisory, Status }` (warning draws red).
- `InspectorData::from_net_snapshot(&net::Snapshot)`: uses only bus `name/voltage_v/live/freq_hz`, contactor `name/closed/current_a`, `battery_current_a`, `converged`. Aliases `EXT_L EPC` to `L EPC`, `L1 TRUC` to `L TRU RLY`. Derives VFSG/ASG/EXT_L_FWD sources, TRU_L/ATU_L converters, BATT_MAIN. Missing names draw dead.
- `InspectorData::demo()`: synthetic cruise state (used by tests, one failed-open contactor, one arc-tripped channel).

## Names the pages look up
- Bus ids: `L1 L2 LTB RTB R1 R2 HVDC_L1/L2/R1/R2 AC115_L/R BB DC28_L/R CIB FIB HBB_MAIN HBB_APU`.
- Contactor names as in `aircraft/24-electrical/contactors.csv` (40 drawn: GCB, BTB, APB, ATRUC, ATUC, EPC, BSB, BTB 115, TRU RLY, RCB, BB RLY, DCT, CIT/FIT, MBR, BATT RLY, SPUB, SPUC, LAEPC).
- Source/converter/battery ids: `VFSG_L1.. ASG_L/R EXT_AFT EXT_L_FWD EXT_R_FWD RAT_GEN ATRU_* ATU_L/R TRU_L/C1/C2/R CMSC_* SPU BATT_MAIN BATT_APU`.

## Pages (all `new(InspectorData)`, `set_data`)
- `SingleLinePage` (id `electrical`): 4 VFSG, 2 ASG, RAT, 3 ext, LTB/RTB ring with 6 BTBs, ATRUs/HVDC/CMSCs, ATUs/115 VAC, TRUs/28 VDC, CIB/FIB/BB/HBBs, batteries, SPU, EICAS box. Live white, dead grey dashed, fault red. Click selects a contactor (`Selection::Contactor`) or bus; detail shows state/current/both sides, table lists every feeder (selected, faulted, closed first). `select_contactor`, `selection`.
- `PanelPage` (id `electrical.panels`): P100/P150/P200/P300/P400/P500/P600/P700/P800/P49 tiles with contents, buses, units, contactors. `select(id)`, `selected_panel()`.
- `RpduPage` (id `electrical.rpdu`): RPDU picker plus grid of every channel (rating, load, state, current). `select_rpdu`, `select_channel`, `rpdu_id`, `selected_channel`.
- `BreakerPage::new(data, rpdu_id, channel)` (id `electrical.breaker`): state chain, I2t trip curve (log-log, instant trip, operating point), 120 s current trace; detail has telemetry, wire and load; table is history. `select(rpdu, channel)`.

## Tests
- `inspector::tests` render each page to `target/agents/inspector/*.png` and pin FNV golden hashes; update `GOLD` in `src/inspector/tests.rs` when the drawing changes on purpose.

## Left
- Zoom navigation between pages (bus to panel, RPDU to breaker) is by the host calling `select_*`; no cross-page click links yet.
- Wire gauge/length/temperature per feeder in the table waits for the full-topology net snapshot.
