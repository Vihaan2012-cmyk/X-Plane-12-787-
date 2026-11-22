# 42 — Common Core System (CCS) and Common Data Network (CDN)

Status: built (steps 1–7, see Build status). Remaining items are under Open data. Sources: `ata42.txt`, `tat_ata42.txt` (TAT ATA 42, 102 pages), `ata45.txt`, `stm.txt` 1570–2492, `lot_fcom.txt`, AD 2020-06-14; facts in `refs/facts/facts_ccs.verified.md`. **(verify)** = not confirmed. `APPROX` = engineered value in code.

## Depth target
| Layer | Depth |
|---|---|
| Power | 4 PCMs on real buses (L: CIB via TCB, R: DC28 R via ECB, + HBB each). 18 V switch-over, 11 s HBB hold-up, save-to-NVM, then shutdown. Driven by the electrical model, not a flag. |
| Modules | 36 cabinet LRMs (16 GPM, 4 PCM, 4 ACS, 4 FOX, + 2 cabinets) + 6 ARS + 21 RDC, each a serialised LRU on `shared-computer`: power, boot, BITE, NVM, heat, wear, tolerances. |
| Network | Every ARINC 664 P7 frame simulated: real bytes, VL, sequence number, CRC, A and B copies, BAG shaping, policing, store-and-forward hops, port queues, redundancy management. Latency, jitter, drops and skew are emergent. |
| Encoding | Every parameter packed to bits from one ICD file (BNR/BCD/discrete/float + status). Quantisation, saturation, bit-flip effects real. |
| RDC | 14 IFZs each (10 CAN/digital, 4 analog), 120 analog/discrete, 6 A429 Tx, 10 CAN; master/checker; program pins pick the config table. One IFZ fault kills only that zone. |
| GPM | Dual-lane lockstep with compare; RTOS partition scheduler with a real major frame; 6 IS partitions; health monitor; overrun = partition fault; overtemp stop. Hosted apps run as partitions with real time windows. |
| Software | Uninhibited / inhibited power-up with documented timings, CCSHM, Time Manager (message age), data load mode, configuration checks, 51-day bug with persistent counter. |
| Crew / maint | CCR RESET L/R (P5), GND TEST switch, EICAS status messages, 11 CCS/CDN maintenance pages, network inspector on the EFB. |
| Not modelled (stop rule) | PHY signalling and light, real CPU instructions, real RTOS code, Boeing's proprietary ICD and VL table (ours is engineered from our own data flows). |

## Scope
- **In 42:** CCR cabinets, PCM, GPM platform, ACS, FOX, ARS, RDC, CDN links, RTOS and the 6 IS partitions, CCR reset, CCS maintenance pages.
- **Hosted elsewhere:** each hosted function's logic lives in its own system crate and runs inside a GPM partition we provide. CMCF and ODLF (ATA 45) and the graphics generators (displays) are separate plans.

## Hardware
- **Cabinets:** 2 CCRs, 16 slots each (ata42:805). Contents per cabinet: 8 GPM, 2 PCM, 2 ACS, 2 FOX, plus 2 graphics generators that belong to displays (ata42:227–236).
  - Location: L in the E1 rack, R on top of the nose wheel well box (ata42:441, 306). Program pins set L or R (ata42:805).
  - Backplane: 2 power buses, one per PCM, shared by all modules (ata42:305).
  - Modules of one type are interchangeable (ata42:804).
- **PCM:**
  - Each PCM has a 28 VDC primary and a battery secondary input (ata42:912). L CCR primary is CIB via TCB, R CCR primary is DC28 R via ECB (ata42:497).
  - Output 12.5 VDC to all LRMs and to the other PCM (ata42:683). One PCM can carry the whole cabinet (ata42:702). Modules on PCM 1 primary take PCM 2 as secondary (ata42:483).
  - Below 18 VDC on the main DC bus it switches to the HBB. HBB hold-up is 11 s, then power is removed (ata42:491, 498, 708).
  - Before removal it tells hosted functions to save SDRAM data to NVM (ata42:489).
  - Its clock runs from the main battery when the aircraft is unpowered (ata42:865).
  - It talks to two GPMs over I2C. Power management apps in GPM L4 and R6 pick the PCM input (ata42:698, 918).
  - Inputs: position discretes, fan speed, aux valve position (ata42:310).
- **GPM:**
  - 2 × 800 MHz CPUs, dual-lane; keeps running on one CPU fault (ata42:1003, 1007).
  - Memory: 256 MB SDRAM, 128 MB PROM, 32 MB NVM, 64 Mb maintenance flash, 128 Mb program flash, 32 Mb buffer NVM (ata42:994–1011).
  - Two ASICs compare the A and B frames (ata42:986).
  - Internal temperature sensor; stops on overtemp, and the other GPMs log it (ata42:966, 980).
  - Integrity < 1e-10 undetected corruption per flight hour (ata42:1080).
- **ACS:** one A and one B per cabinet. 24 ports: 20 copper at 100 Mbps, 4 copper at 10 Mbps for external LRUs (ata42:1132, 1137, 1155).
  - Connects GPMs, GGs, P300/P400, FCE, RPDUs and some RDCs (ata42:714).
  - Filters, polices and routes from its configuration table (ata42:1114).
- **FOX:** one A and one B per cabinet. Electrical to fibre at 100 Mbps, no software (ata42:1177, 1182, 718).
  - Links to the same-channel FOX in the other cabinet (ata42:1179).
  - EEPROM holds part number, serial, elapsed time and faults (ata42:1183).
  - Fibre bend radius ≥ 1.5 in, so a tighter bend is a real attenuation fault (ata42:1191).
- **ARS:** 6 units, 3 A and 3 B; 2 forward, 4 aft (stm:1783, 1899).
  - 24 ports: 20 copper at 10 Mbps, 4 fibre at 100 Mbps (ata42:1260).
  - 8.0 lb, 5 connectors (ata42:1230, 1234).
  - ARS 5 sits on the left side of the E1 rack (ata42:416). Others from the Locator **(verify)**.
- **RDC:** 21 units (ata42:1390).
  - Per RDC: 120 analog/discrete, 6 A429 Tx, 10 CAN (ata42:1287–1296). Analog types include current, frequency, voltage, potentiometer, LVDT, RVDT, fire detection and RTD (ata42:157, 1290).
  - 14 IFZs (10 CAN/digital, 4 analog). Two gateway processors link the IFZs over two SPI networks (ata42:1322, 1338, 1345).
  - Dual master/checker main processors, high-integrity end system, A and B channels, no fibre (ata42:1075, 1309, 1313, 1380).
  - 28 VDC power with primary and secondary sources; RDC 21 primary is CIB via P300 (ata42:498, 572).
  - Same OFP in all RDCs. Program pins give the position, which selects the config table (ata42:1302, 1309).
  - Numbers are 1–19, 21 and 23 (mmel87:15790).
  - **Power and location** for 20 of 21 come from the TAT power tables (p29, p30, p33), in `aircraft/42-ccs/rdcs.csv`:
    - Primary: the instrument bus (CIB left, FIB right).
      - RDCs 1, 7, 9, 11 (left) and 2, 8, 10, 12 (right) through P300/P400 TCBs.
      - RDCs 3, 13 (left) and 4, 14 (right) through ECBs.
      - RDCs 5, 15, 17 through RPDU 81; RDCs 6, 16, 18, 19, 23 through RPDU 82.
    - Secondary: DC28 L or R through an RPDU SSPC (RPDUs 71–76, 21/22, 31–34, 41/42, 81/82).
    - Locations come from the breaker names: flight deck, forward E/E bay 1 and 2, aft E/E bay, and crown at doors 1–4.
    - RDC 21: primary is CIB via P300 (ata42:498); secondary and location **(verify)**.
  - RDCs 1/3 and 2/4 carry the left and right thrust-reverser lock proximity sensors (mmel87:15730).
- **Links:** fibre at 100 Mbps; copper at 100 Mbps inside cabinets, 10 Mbps outside; anything faster than 10 Mbps outside a cabinet is fibre (ata42:698, 706, 712, 719).
- **Every module, port, cable and connector** is an entity with temperature, wear, tolerance, failures and zone.

## Power and boot (links to 24)
- **Power-up modes** (ata42:1594–1621):
  - **Uninhibited:** on the ground with external power or the battery selected.
    - Full PBIT, up to 3 min. L CCR start takes 2–3 min. Default displays appear in 25–45 s and hosted functions are up in 45 s.
    - On battery only, just the L CCR is powered (ata42:1600). The R CCR starts when the BPCU enables the main DC loads (ata42:1602).
    - Battery first then external power can take up to 6 min for both (ata42:1611).
  - **Inhibited:** a CCR reset in the air or with a fuel cutoff switch in RUN.
    - Partial PBIT. Displays in 60 s, hosted functions in 55–75 s (ata42:1601, 1646–1647).
  - Network switches, FOX modules and RDCs always start in under 45 s (ata42:1621).
  - **Start-up complete:** default displays, stick shaker cycles, 1 kHz tone (ata42:1614, 1619).
  - **Video evidence** (two real-aircraft clips, both edited, so used for order only, not timing):
    1. Battery ON, then every display stays dark for a long period.
    2. The captain inboard display comes up first with EICAS, and the lower center MFD with system and maintenance pages, still on battery (L CCR only).
    3. After APU start and the APU generators come online, the PFDs, NDs and right-side displays follow (R CCR).
    4. The APU controller waits for the CCS applications before it starts the APU, as lot_fcom:83232 says.
  - **Conflict:** LOT FCOM gives about 1 min for a CCR restart (lot_fcom:64437). Use the ATA 42 times as the bounds, with the FCOM value as the inhibited case **(verify by video)**.
- **APU start** waits for CCS applications, then 15 s after the L CCR is online (lot_fcom:83232, 12104). This replaces today's electrical rig shortcut.
- **QRH Loss of All Displays** (qrh:8049–8091):
  - L CCR RESET, then wait 1 min: four forward displays come back, and the lower display and both HUDs stay blank.
  - If not, reset both, then wait 1 min.
  - On loss of all displays: autothrottle is inoperative, LNAV/VNAV/FLCH are unusable, and window heat and wing anti-ice may be lost.
  - This is a test target, and supports the ~1 min inhibited restart.
- **CCR RESET** (P5, guarded, hard-wired to the onside PCMs): a ground discrete removes module power immediately, skipping the 11 s shutdown and NVM save (ata42:1675, 1709–1710).
  - ARS and RDC are not affected (ata42:1675).
  - Using it during maintenance can corrupt a GPM: modelled as a probability of NVM corruption when reset mid-write (ata42:1679).
- **Electrical changes:** the `CCS_HOLD_US` shortcut in the BPCU moves here. The CCR, RDC and ARS loads become real loads drawn per mode (boot, run, fan).

## Cooling
- Normal air comes from forward equipment cooling (ata42:931).
- **LPS** is a hosted app. If supply pressure is low, it has the PCM run the cabinet fan and close the duct valve, a two-position door (ata42:930, 942, 943).
- **AAF** monitors cabinet temperature. On overtemp it blocks the inlet and runs the fan. If that fails on the ground, the PCM removes LRM power (ata42:952, 963, 930).
- Messages: `CCR CABINET COOLING L/R` for a fan or valve failure (ata42:919); AUX COOLING shows SEE CMCF on the PCM page (ata42:913).
- Module heat goes into the forward E/E zone. Each GPM's temperature drives derating and its overtemp stop.

## Network (ARINC 664 P7, frame by frame)
- **VL table:** generated from `docs/systems/data-flows.md`.
  - One VL per data set: source, destinations, BAG 1–128 ms, max frame size, priority. Static routes through the ACS/ARS/FOX tables.
  - The real table isn't public, so ours is marked `APPROX`.
- **End system:**
  - **Transmit:** shaped to the BAG with real jitter; every frame goes out on A and B with a sequence number and CRC.
  - **Receive:** first-valid-wins redundancy management, with a sequence check and a maximum A/B skew.
  - **Ports:** sampling ports hold the value and its age; queuing ports are FIFOs that can overflow.
- **Switch:**
  - Store-and-forward: each hop costs frame bits ÷ link rate plus switch latency.
  - Two-priority output queues.
  - Per-VL policing on BAG, jitter and size. Violations and unknown VLs are dropped and counted.
- **Links:** a bit error rate from each link's optical or electrical margin (wear, bends, corrosion). CRC catches most errors; an undetected one lands in the payload.
- **Cost:** event-driven on the discrete-event clock. Budget under 1 ms of CPU per ms of sim at full load **(measure)**.

## Data movement (rule 10)
- Systems exchange data only as CDN frames, A429 words, CAN frames, or discrete and analog wires.
- **Example path:** sensor → wire → RDC IFZ → RDC end system → ARS → FOX → ACS → GPM partition → ACS → GG → display.
- **A429:** label, SDI, data, SSM, parity, at 12.5 or 100 kbit/s, with real word rates.
- **CAN:** arbitration, error frames, bus-off.
- **Discretes and analog:** wire voltage from the wiring model. The RDC ADC adds resolution and noise. An open wire reads as an open wire.
- **Enforced:** function crates see only their ports. An audit test fails the build if anything else reads model state.
- **ICD file:** `aircraft/42-ccs/icd/*.toml` defines every message, parameter, encoding, offset, rate and VL. Encoders and decoders are generated from it. Each box carries an ICD version, and a wrong load decodes the right bytes into wrong values.

## GPM software
- **RTOS:** a partition scheduler runs each app in its window and rate from the configuration table. Apps talk only through ports (ata42:1073–1074). The BSP includes low-level BIT (ata42:1090).
- **6 IS partitions** (ata42:1092):
  - **CCRSM:** support and maintenance, including auxiliary airflow.
  - **CCSHM:** consolidated health view and system BIT through RDC test channels. Raises EICAS status messages for platform failures and configuration mismatches (ata42:331–336). If CCSHM itself fails, `CCS HEALTH MANAGEMENT` appears after a delay (ata42:339).
  - **Data Load:** runs only in data-load mode, over TFTP and ARINC 615A-2 on the CDN, and can reload most items including itself (ata42:338–343).
  - **Time Manager:** distributes the time reference used for message age checks (ata42:341).
  - **FIDO:** memory monitoring.
  - **File System:** volatile and non-volatile files across power cycles (ata42:348).
- **Hosted apps:** 34 applications mapped to L1–R8 in `aircraft/42-ccs/hosted-apps.csv`, read from the TAT ATA 42 allocation graphic (p60).
  - Most apps run in both cabinets; single-instance apps are cabin air temperature (L4), ACMF (R1) and nitrogen generation (R4).
  - Load is uneven: L1 and R1 carry most utilities; L2, L3, R2 and R3 carry alerting and displays; L5 and R5 carry thrust management and FMF.
  - Power management is in L4 and R6 (ata42:698), which matches electrical power distribution in L1 L4 L8 R1 R6.
  - **GPM loss:** exactly its apps drop, or their other instances carry on, per the table.
- **Health monitor:** an overrun is a partition fault and the partition restarts. Lane miscompare puts the GPM on one lane, or stops it on a second fault.
- **51-day bug** (AD 2020-06-14): the stale-data monitor counter persists across sessions. After 51 days powered, a replaying switch makes stale flight-critical data show as valid.

## Crew and maintenance interface
- **Controls:** CCR RESET L/R (P5); GND TEST switch with NORM / ENABLE / DATA LOAD-ENABLE and a 2.5 s delay leaving the last two (ata45:239, 301).
- **EICAS status messages:** `CCR FIBER OPTIC XLTR`, `CCR CABINET COOLING L/R`, `CCS HEALTH MANAGEMENT`, plus ACS, ARS, RDC, GPM and PCM messages (ata42:980, 1199) **(verify exact list from FCOM)**.
- **Maintenance pages:** 11 pages (stm:2315–2326; ata42:1726–1824).
  - Pages 2–3: L and R CCR.
  - Pages 4–6: DCAS.
  - Page 7: RDCs.
  - Page 8: the 6 switches.
  - Pages 9–11: CDN LRUs, connection status only.
  - Display rules: NORMAL in white, FAULT in amber, SEE CMCF in amber box. PCM MASTER is shown blank.
- **Network inspector (EFB):** frame log with decoded or hex payload; per-VL rate, latency, jitter, drops, skew and age; per-port counters and queue depth; per-GPM partition load and health; live topology.

## Dispatch (MMEL, `aircraft/42-ccs/mmel.csv`)
- GPM 8L or 8R may be inoperative if the other works; one HF is lost unless CN-AA39640 is incorporated.
- RDCs: 1 of 1/3, 1 of 2/4, 15 of 16 (5–18, 21, 23), and 0 of 1 (19), each with the `CCS RDC` status message.
- No other CCS item is dispatchable: any other GPM, PCM, ACS, ARS or FOX fault is no-go.

## Failures (0–1, on parts)
- **PCM:** degraded output, input lost, hold-up loss, clock battery.
- **GPM:** one CPU lane, both lanes, RAM bits, NVM corruption, partition crash, health-monitor fault, temperature sensor.
- **ACS / ARS:** one port, the whole switch, corrupt config table, replaying queue, stuck queue.
- **FOX:** optical power.
- **Fibre:** open, bend, dirt, intermittent.
- **Copper and connectors:** open, intermittent, corrosion.
- **RDC:** one IFZ, a gateway processor, a master/checker lane, program pin, end system.
- **End system:** babbling, stuck sequence number.
- **Cooling:** fan, valve, LPS sensor.
- **Software:** wrong load.

## Temperature, wear, tolerances
- **Temperature:** every module temperature is driven by its power and the cooling air. Overtemp stops a GPM; on the ground the cabinet shuts down. Thresholds **(verify)**, `APPROX` until then.
- **Wear:** connector and fibre margin falls with cycles and age; PSU capacitors age per `shared-computer`; fan bearings.
- **Tolerances:** each airframe gets its own oscillator ppm, link margins and boot-time spread.

## Tests
1. Cold battery power-up: only the L CCR boots. Displays in 25–45 s, hosted functions in 45 s. With external power the R CCR follows once the BPCU enables the main DC loads.
2. Inhibited restart in the air (CCR RESET): displays in 60 s, functions in 55–75 s, no NVM save.
3. Main DC drops below 18 V: HBB hold-up for 11 s, NVM save, then off. A return inside 11 s means no reboot.
4. One PCM lost: the cabinet stays up. Both lost: the cabinet goes down.
5. ACS-A fails: everything continues on B, and the status message and maintenance page show it.
6. Intermittent fibre: CRC count rises, A/B redundancy hides it, counters show it.
7. Babbling RDC: policing drops the excess, and other VLs keep their latency bound.
8. Queue overflow: stale sampling ports, and consumers flag the data invalid.
9. One RDC IFZ fails: only that zone's signals go invalid.
10. A GPM is lost: exactly its hosted apps degrade or fail over, per the table.
11. Cooling: duct valve fails closed, then overtemp, then GPM stop; on the ground the cabinet shuts down.
12. 51 days powered plus a replaying switch: stale data is displayed as valid.
13. Every VL stays inside its latency bound at full load. CPU budget met.
14. APU start waits for CCS plus 15 s.

## Build order
1. **Platform:** cabinets, PCMs and power/boot on the electrical buses, cooling, CCR RESET. Electrical's `CCS_HOLD_US` is removed.
2. **CDN:** end systems, switches, links, the VL generator, and the ICD codegen.
3. **RDCs:** IFZs, I/O types, pins and config tables.
4. **GPMs:** scheduler, health monitor, IS partitions, and the hosting API for system crates.
5. **Interfaces:** EICAS status messages, maintenance pages, EFB network inspector.
6. **Electrical on the CDN:** move electrical's internal paths (EICAS, synoptic, CBIC, load management to the RPDUs, BPCU data) onto real frames.
7. **Failures and tests,** including the 51-day case.

## Build status
| Step | State | Notes |
|---|---|---|
| 1 Platform | Done | PCM power, 18 V switch-over, 11 s HBB hold-up with NVM save, reset hard-off, start modes, cooling LPS/AAF, RDC power from TAT tables |
| 2 CDN | Done | Frame-level A/B network: BAG shaping with jitter, policing, store-and-forward, 2-priority queues, CRC, BER, integrity and redundancy checks, sequence numbers; quiet-network fast path when nothing changes or fails |
| 3 RDC | Done | 21 RDCs; 14 IFZs carried as validity bits in each RDC frame; IFZ fault drops only that zone |
| 4 GPM | Done | 6 IS partitions plus hosted apps per GPM in a 25 ms major frame (`APPROX`); crash, restart, 3 restarts in 60 s holds the partition; dual lanes; data-load mode gated by GND TEST |
| 5 Interfaces | Done | Crew status messages through CCSHM; `CCS HEALTH MANAGEMENT` when every CCSHM is lost; CCS/CDN maintenance pages 1–3 and 7–11 (4–6 DCAS belong to displays) |
| 6 Electrical on CDN | Done | Electrical EICAS state goes from the BPCUs (P300/P400) as a dataset to crew alerting on L2/L3/L7/R2/R3; CAS shows it only while its data is valid |
| 7 Failures and tests | Done | 9 part types, 35 modes (693 failure points), every mode visible (audit test), every link open visible; aircraft stress with invariants and determinism |

### Behaviours worth knowing
- **Message age uses the source timestamp in the payload**, not reception time. A replaying switch can deliver old frames that pass the integrity check, so the age monitor catches them; after 51 days of power it does not (AD 2020-06-14).
- **CCR power is constrained by the battery-only bound:** discharge ≥ 30 A for 20 s inhibits discharge (eps24:21949). Boot 220 W, run 250 W `APPROX`; electrical's 220 W CCS stand-in (`STANDIN_CCS_L`) is zeroed when CCS is coupled.
- **Source corruption before CRC** gives plausible but wrong data: a corrupted BPCU frame shows a false ELEC message.

### Engineered (`APPROX`)
- ARS placement and fibre ring (FOX L→ARS 5/6, FOX R→ARS 1/2, ARS 1–3, 5–3, 2–4, 6–4); RDC→ARS by location; RDC 1/2 and gateway RPDUs on ACS 10 Mb ports.
- VL table generated from hosted apps, RDCs, BPCUs and gateway RPDUs (RDC 16 ms, apps and BPCU 32 ms, time 128 ms).
- Switch latency 20 µs, queue 64 frames, max skew 2 ms, stale after 3 BAG + 2 ms, major frame 25 ms, partition restart 2 s.
- Cabinet thermal constants and overtemp thresholds.

## Open data
- ARS positions, RDC 21 secondary and location, and the sensor-to-RDC map (Locator, ATA books).
- Exact CCS EICAS status messages (FCOM).
- Boot sub-step timings (video).
- Overtemp thresholds.
