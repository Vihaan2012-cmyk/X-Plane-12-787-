# API: net (b789-sys-electrical, feature `net`)

Full aircraft topology built from `aircraft/24-electrical/*.csv` (own CSV parser, quotes and embedded newlines handled).

## ElectricalNet
- `from_data(dir: &Path) -> Result<ElectricalNet, String>`; `new()` = `from_data(default_data_dir())`, panics on bad data. `default_data_dir()` is `<manifest>/../../../aircraft/24-electrical`.
- Name lists (data order, `&[String]`): `contactor_names()`, `bus_names()` (sorted), `machine_names()`, `load_names()`, `sspc_ids()`. `node_count()`.
- `set_contactor(name, closed) -> bool`: `name` is the contactors.csv `name` column (empty name -> id; duplicate -> `"<name> <id>"`). false if unknown.
- `set_machine_rpm(name, rpm)`, `set_machine_field(name, field_pu)` (1.0 = feed-forward nominal; scales exciter current). Machines: VFSG_L1 VFSG_L2 VFSG_R1 VFSG_R2 ASG_L ASG_R RAT.
- `set_external_power(receptacle, connected)`: "L FWD", "R FWD", "AFT" (115 V 400 Hz source).
- `set_load(name, demand_w)`: constant-power demand (AC balanced over 3 phases). For an RPDU channel load the demand only flows while its SSPC is closed.
- `set_sspc(id, closed)`: ids from `sspc_ids()`.
- `step(dt_us) -> Result<Snapshot, String>`: steps machines and batteries, writes Thevenin values, solves, then re-solves up to 6 passes until the diode switches settle.
- `battery()` (main), `battery_apu()`.

## Snapshot (all names are `String`)
- `buses: Vec<BusReading {name, voltage_v, live, freq_hz}>` (AC: phase-to-neutral rms; DC: volts; HVDC buses are one node at +270 V rail, the +/- pair is not split).
- `contactors: Vec<ContactorReading {name, closed, current_a}>` (inert contactors with no connection in the data report 0 A).
- `machines: Vec<MachineReading {name, freq_hz, terminal_v, current_a, field_pu}>`.
- `batteries: Vec<BatteryReading {name, voltage_v, current_a, soc, temp_k}>` (BATT_MAIN, BATT_APU).
- `loads: Vec<LoadReading {name, voltage_v, current_a, power_w}>`; `sspcs: Vec<SspcReading {id, rpdu, voltage_v, current_a}>`; `battery_current_a` (main), `converged`.
- Lookups: `bus`, `contactor`, `machine`, `battery`, `load`, `sspc`.

## Topology (from data)
- Buses: buses.csv (L1 L2 R1 R2 LTB RTB BB AC115_L AC115_R HVDC_x4 DC28_L DC28_R CIB FIB HBB_MAIN HBB_APU) plus ATU_L/ATU_R (235 V side) plus `RPDU nn DC` (17) and `RPDU nn AC` (15, not 81/82).
- Contactors: all 101 rows of contactors.csv, plus `AC FEED RPDU 31/32/41` from breakers.csv ELCF rows and a synthetic `AC FEED RPDU 33` (APPROX, no row in the data). Rows with blank connects (GNR, GEC, SC, ram fan, CAC, galley, window heat) are inert state-only contactors. Default open; `CIT`/`FIT` (type NC) default closed.
- Power path:
  - VFSG -> `Lx/Rx GCB` -> L1 L2 R1 R2; ASG -> `L/R APB` -> LTB/RTB; ring per BTBs (L1 BTB L1-RTB, L2 BTB L2-LTB, L3 BTB L1-L2, R1 BTB R1-LTB, R2 BTB R2-RTB, R3 BTB R1-R2).
  - Ext FWD L/R -> `L/R EPC` -> AC115_L/R; each ATU is two autotransformers (down 235->115 via `ATUC` side, up 115->235 for ground power) behind direction diodes; `L/R BSB` joins ATU 115 side to AC115 bus, `L/R ATUC` joins L1/R1 to ATU 235 side; `L/R BTB 115` tie the 115 buses.
  - Ext AFT -> `LAEPC` -> ATRU_L2 115 V input only (second rectifier on HVDC_L2).
  - ATRU_x (235 V, 270 V, 97%) behind `x ATRUC` -> HVDC_x; TRU L/R (`L/R TRU RLY`, from L1/R1) -> DC28_L/R; TRU_C1 (`C1 TRU RELAY` from BB, `C1 TRU ISO R` from L2) -> CIB; TRU_C2 hard-wired from BB -> FIB (no relay in data).
  - `L DCT`/`R DCT` tie DC28_L/R; `CIT`/`FIT` tie CIB/FIB; `MBR` HBB_MAIN-CIB.
  - BB: R2 -> `BB RLY` -> mid node -> `BB ISO RLY` -> BB (series, APPROX); RAT -> `RCB` -> BB.
  - Batteries: 8 cells x 50 Ah each, hard-wired to their hot buses through a battery diode module (diode switch: conducts only while battery EMF > bus voltage), so hot buses are live with every contactor open. No charger model.
  - SPU: `SPU contactor` (HBB_APU -> SPU input, constant-power load = output/0.85) and `SPUB`; source 135 V behind a diode onto HVDC_R2, enabled only while the SPU contactor is closed and the APU bus > 18 V.
  - Rectifier / ATU / battery / SPU outputs go through internal diode switches (not contactors): closed when the inner EMF exceeds the far bus voltage; this isolates dead converters.
  - Dead machines and external sources use R = 1e6 ohm so a closed GCB on a dead source does not short the bus.
- RPDUs (rpdus.csv): per RPDU a DC node (main feed SSPC from DC28_L for odd / DC28_R for even, standby feed SSPC from CIB / FIB for 71 73 81 / 72 74 82) and an AC node (`AC FEED RPDU n` contactor from AC115_L odd / AC115_R even). Feed SSPC ids are the breaker ids (CE...); default closed.
- SSPC channels: one per loads.csv row with an `RPDU nn` unit (id `RPDUnn.kk`, default open); load name = loads.csv `load` (duplicates get ` @ RPDU nn` or ` #k`). AC if voltage text has 115/235/"v ac" and the RPDU has an AC node, else 28 V DC. Demand = `draw` if "NNN W", else APPROX default 60 W AC / 15 W DC (flagged: all RPDU channel loads are APPROX unless draw is stated).
- Lumped loads `LOAD_<bus>` on L1 L2 R1 R2 BB AC115_L/R DC28_L/R CIB FIB HBB_MAIN HBB_APU (defaults: DC28_L 560 W, HBB_MAIN 280 W, else 0) and 8 CMSC loads (conversion.csv ids) on HVDC buses at 270 V (default 0 W).

## Counts (current data)
- 53 buses, 105 contactors, 7 machines, 149 loads, 151 SSPCs (128 channels + 23 feeds), 93 solver nodes.
- Full-network step about 1.3 ms (release-like test profile, 3 solves when diodes change).

## Not done
- GCU/BPCU logic (controllers close contactors), GATU, EBPSU, backup bus alternate from L1, battery charger, +/- 270 V rail split, real contactor dynamics, ELCU/SPDU per-card limits.
- Generator excitation is open-loop feed-forward x `field_pu`.
