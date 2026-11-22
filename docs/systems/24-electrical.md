# 24 — Electrical: generation, start, conversion, distribution, every circuit breaker

Status: full plan, from the verified facts plus six research tables (`elec2_*`). Every number cites its source or is marked **(verify)**. `APPROX` marks anything engineered because no source states it.

Citation keys (file:line):
- `E` = eps24.txt (Hamilton Sundstrand EPGSS/EPS course)
- `A` = ata24.txt (ATA 24 book)
- `S` = stm.txt
- `F` = lot_fcom.txt (LOT FCOM Rev 12)
- `Q` = qrh.txt
- `A28`, `A29`, `A38`, `A42` (written `ata42` in text) = the ATA books
- `P` = figure page of 24eps.pdf (e.g. P:955)
- `B` = elec_breakers.verified.md (breaker/contactor ID table, by ID)
- `L` = elec_loads.verified.md (load to power path)
- `C` = elec_conditions.verified.md (what works in each mode / failure)
- `R1`..`R8` = refs/facts/elec2_* tables (R1 = r1a conflicts, R1b = r1b contactor IDs, R2 sources, R3 conversion, R4 topology/BPCU, R5 batteries/backup, R6 distribution units, R7 crew interface, R8 timing/loads); line cites inside each

## Scope
- Sources, start, conversion, every bus, every contactor, BPCU/GCU/ELCU/RPDU/SPDU logic, CBIC, load management, batteries, RAT, ESIC crew interface.
- Not here: what each load does (owned by its system). Here we only switch, protect and measure it.

## Hardware: sources
| Source | Count | Rating / data | Cite |
|---|---|---|---|
| VFSG (engine) | 4 (2 per engine, IP-spool gearbox) | 250 kVA, 235 VAC 3φ wye, 360–800 Hz, PF 0.85–1.0 lag, 6-pole brushless | E:9636, E:9648, E:9646 |
| VFSG overload | | 125% = 312.5 kVA 5 min; 175% = 437.5 kVA 5 s | E:9688, E:9689 |
| VFSG start duty | | 300 ft-lb per VFSG; 3 min at 100% then 30 min cooldown at 50% torque; inhibited in flight | E:9756, E:9782, E:9776 |
| VFSG motoring | | 150 ft-lb maintenance | E:9745 |
| VFSG online speed | | ground idle 370 Hz = 7400 rpm (shaft rpm = 20 × POR Hz) | E:9761, E:17599 |
| VFSG PMGs | 2 stators | one for GCU regulator, one for flight-control power; PMG = 3× POR Hz (1080–2400 Hz) | E:2808, E:18072 |
| VFSG mass | | 207.9 lb | E:9680 |
| VFSG shaft shear | | 79,500 in-lb/rad | E:10144 |
| VFSG oil | 2 independent systems per engine | normal 40–129 °C; cooled by air/oil + fuel/oil exchangers; EEC air valve (open at ground idle, closed in flight, spring to open) | E:10267, E:2839, E:2847, E:1419 |
| Oil level sensor (ROLS) | per VFSG | under-service only, active with generator stopped, read ≥ every 2.5 min | E:10203, E:4787 |
| Drive disconnect | per VFSG | solenoid + solder-pellet thermal disconnect at 199 ±3 °C (390 ±5 °F) | E:12512, A:4051 |
| ASG (APU) | 2 | 225 kVA, 230/400 V 3φ; PMG = 4.5× POR (360–441 Hz) | E:2962, E:12564, E:18152 |
| RAT generator | 1 | 230 VAC variable freq, 10 kVA max, 10 s from deploy to power; own PMG | S:8183, S:8185, A:5152 |
| RAT pump | | 10 gpm at 4850 psi (10–19 gpm at 2700–4850) | A29:3069, A29:3065 |
| External power | 3 (fwd L, fwd R, aft) | 115 VAC 3φ 400 Hz; fwd max 90 kVA each; aft rated 188 A | A:975, A:1283, E:4900 |
| Main battery | 1 | Li-ion 28 V, 50 Ah end of life (A, E) vs 65 Ah (S) — conflict; nominal 29.6 V | E:1423, A:544, S:7501, S:6721 |
| APU battery | 1 | same type; three 45 s start attempts; 10 s hold-up of hydraulic CMSC 28 V | E:21589, E:21591 |
| WELS emergency lighting packs | | discharge ~15 min, recharge ≥ 2 h | A:589 |
| Fuel shutoff module | 1 | capacitor store, 24 V, 3 valve cycles, 1 h hold-up | A28:2681, A28:2694 |

## Hardware: conversion
| Unit | Count / location | Data | Cite |
|---|---|---|---|
| ATRU | 4: L1, L2 in P700; R1, R2 in P800 (aft EE bay), PECS liquid cooled | 235 VAC → ±270 VDC; 787-9: 155 kVA continuous, 165 kVA 10 min/60 min, 187.5 kVA 5 min, 200 kVA 30 s, 245 kVA 5 s/60 min | A:2417, A:2503, R3:A:6143–6147 |
| ATRU I/O | | in 360–800 Hz (235 V), 380–430 Hz (115 V); out ±250/±280/±320 V (min/nom/max); 274 A nominal out | A:2450, A:2451, A:2468, A:2470 |
| ATRU cooling | | coolant ≥ 6.8 kg/min, ≤ 96 °C; ambient 15–70 °C; coolant interface 130 psi nominal | A:2458, A:2459, A:2453, A:2484 |
| ATRU per-unit | | 1 PU = 212 A (aft EP via L2: 188 A) | E:4925, E:4933 |
| L2 and R2 ATRU extra | | also take 115 VAC: L2 from aft EP, R2 from SPU (±135 VDC) | A:2437, A:2413 |
| ATU | 2: L in P500, R in P600 | 235 VAC → 115 VAC (down); on ground 115 → 235 (up) with fwd EP; 1 PU = 260 A | A:1408, A:1411, E:4945 |
| GATU | 1, left aft cargo sidewall | 235 → 115 VAC for aft galleys, via GATU contactor from L2 bus | A:1550, A:1560, A:1569 |
| TRU | 4: L, C1 in P500; C2, R in P600 | 235 VAC → 28 VDC; each feeds a 28 V bus plus an EBPSU | A:1455, A:1458, A:1463 |
| EBPSU | 4 (2 in P500: LI, RI; 2 in P600: LO, RO) | electric brake supplies from 28 V bus, main battery backup | A:1487, E:934, F:81060 |
| SPU | 1, rack E3 aft EE bay | APU battery 28 VDC → ~200 VAC φ-φ; 13.5 kW from 18–33 VDC; ≥ 80% efficient; 3 APU starts | A:2971, A:2974, A:2941, A:2966 |
| CMSC | 8: 4 in P700, 4 in P800 | 270 VDC → 3φ 0–235 VAC, 0–1400 Hz; 105 kW (ata24, 787-8/-9; eps24 says 110) with 150 kW 3 min; 52–53 kg; liquid cooled; 787-10 unit not interchangeable | A:6473, E:36424, E:36425, E:36419, A:6480 |
| CMSC modes | 6: MES, AES, EMP, CAC, RAF, NGS | MES 450 A to 340 Hz (3 min); AES 120 A/207 Hz; CAC 240 A/1409 Hz; NGS 135 A/1367 Hz; RAF 70 A/395 Hz; EMP 120 A/207 Hz (5500 rpm), 190 A/174 Hz (3300), 205 A/144 Hz (2700) | E:36686–36711 |
| RFMC | ram-fan motor controllers, P700/P800 | pack switch goes to 4 CAC CMSCs + 2 RFMCs | A:6196 |
| OVTPU | 4 (2 in P100, 2 in P200); 220 µF capacitors in the panel **(verify which unit)** | over-voltage transient protection, 1 per VFSG | A:4051, E:18036 |
| PECS | 2 liquid loops, 2 redundant pumps each | coolant 29 °C nominal; power held 10 min after APU/2 fwd EP off, 22 min with 1 fwd EP | A:2339, A:2344, A:2355, A:2357 |

CMSC allocation (787-9):
- L1 CAC CMSC: L1 CAC, L VFSG start (MES), left ASG start (AES), left RAF (L1 RFC is a 787-9/-10 contactor) (A:6183, A:5987).
- L2 CAC CMSC: can drive both ram fans at once (A:5958).
- R2 CAC CMSC / R2 EMP CMSC: right ASG start paths; BPCU rotates the APU start path daily (E:12219).
- Each CMSC runs one motor at a time; contactors in P700/P800 pick the motor (A:2546, A:2533).
- Pin-programmed identical boxes (E:36401). Which CMSC drives which hydraulic/NGS motor: **(verify)** per `aircraft/24-electrical/cmsc.*`.

## Hardware: buses
| Bus | Count | Source / fed through | Cite |
|---|---|---|---|
| Main 235 VAC | 4: L1, L2 (P100), R1, R2 (P200) | each VFSG its own bus (4-channel isolated) | A:4311, A:4495, E:475 |
| Tie buses | 2: LTB, RTB | each ASG its own tie bus; 6 buses in a ring | A:4272–4286 |
| Bus tie breakers (BTB) | 6: L1–L3 in P100, R1–R3 in P200 | normally open, GCU controlled | A:4337, A:3109, A:1529 |
| ±270 VDC (HVDC) | 4: L1, L2 (P700), R1, R2 (P800) | one ATRU each, ON above 205 V, OFF below 170 V | A:2417, E:5095 |
| 115 VAC | 2: L, R (P300/P400) | each from its ATU or fwd EP | A:989, A:1710 |
| 28 VDC main | 2: L, R | TRU L / TRU R | A:1719 |
| Instrument buses | 2: captain (P300), F/O (P400) | C1 TRU / C2 TRU; main battery via MBR; CIT/FIT tie | A:1719, E:21750 |
| Backup bus | 1, P400, 235 VAC | normal R2; alternate L1 or RAT; feeds C1 and C2 TRU + small ATU (window heat) | E:716, E:718, A:1587 |
| Main hot battery bus (HBB) | 1, P300 | main battery direct; few loads (fire bottles, RAT manual deploy, CCR memory) | E:21632, S:7503 |
| APU HBB | 1, P49 | APU battery direct | S:7505, E:30509 |
| Virtual ground service bus | | RPDUs from live buses; ~160 extra loads | A:5977, E:34472 |

HBB thresholds: UV fault < 24 V (15 s), OV fault > 35 V (60 s) (E:21826, E:21828).

Main HBB thermal-breaker loads (E:28222–28238):
| Load | TCB position |
|---|---|
| CCR cabinet L BAT / R BAT | D21 / D22 |
| FCE cabinet C1 BAT / C2 BAT | D30 / D29 |
| Engine fire bottle squibs L1 / L2 / R1 / R2 | D44 / D46 / D45 / D47 |
| RAT manual control 1 (deploy solenoid) | D48 |
| Attitude/heading ref unit battery L | D39 |
| Landing gear alternate extension motor (nose) | D31 |
| Tow lights, flight deck (ground power tow relay) | D103 |
| Ground crew call horn transfer power | D34 |
- Main battery wiring in P300: L13 charger input, L14 main battery, L19 charger output (E:28412).
- APU HBB (P49) relays: manual RAT deploy K3, APU inlet door, ground tow nav lights, SPU ATRUC interlock (E:30509).
- Towing power feeds: captain interphone, dome lights, aisle-stand floodlights, wing and tail position lights; no radios (S:7047, F:81137).
- On-ground battery-only powers: captain inboard DU, lower DU, MCP, CCDL, APU start capability, AHRU L/R, fueling panel, FMC, ACPs, CCR L, crew call horn, flight interphone, engine/APU fire detection, VHF L, misc lighting, TCPL, brakes (C, on-ground battery-only rows).
Wiring: 235 VAC loads use ELCCs only, no breakers (A:4601). About 40 primary 235 VAC loads (A:4614).

## Hardware: contactors and relays (IDs settled in R1b from table + figure images)
| Function | IDs | Panel |
|---|---|---|
| Generator control breaker GCB | L1 CK2422525, L2 CK2422526, R1 CK2422528, R2 CK2422529 (P5 controlled) | P100/P200 |
| Generator neutral relay GNR | L1 CK2415001, L2 CK2415002, R1 CK2415004, R2 CK2415005, LA CK2412003, RA CK2412006 (probable; CK2415006 is the ram-fan contactor RFCL) | P100/P200/P150 |
| APU breaker APB | L CK2422527, R CK2422530 | P150 |
| Bus tie breaker BTB | L1 CK2421501 (L1 bus to RTB), L2 CK2421502 (L2 bus to LTB), L3 CK2421523 (L1–L2), R1 CK2421503 (R1 to LTB), R2 CK2421504 (R2 to RTB), R3 CK2421524 (R1–R2) | P100/P200 |
| Start contactor SC | L1 CK2415501, L2 CK2415502, R1 CK2415503, R2 CK2415504; APU L CK2412501, R CK2412502 | P100/P200/P150 |
| Exciter relays GEC1/GEC2/GEC3 | L1 CK2415616/2415515/2415714; L2 CK2415619/2415518/2415717; R1 CK2415607/2415506/2415705; R2 CK2415610/2415509/2415708; APU GEC2 L CK2412521, R CK2412512; APU GEC3 L CK2412720, R CK2412711 | P100/P200/P150 |
| ATRU contactor ATRUC (300 A) | L1 CK2435501, L2 CK2435502, R1 CK2435503, R2 CK2435504; soft-start L1 CK2435505, L2 CK2435506, R1 CK2435507, R2 CK2435508 | P100/P200 |
| ATU contactor ATUC (150 A) | L CK2425501, R CK2425503; GATU contactor **(verify)** | P100/P200 |
| 115 VAC source/tie | L EPC CK2441301, R EPC CK2441302 (260 A), L BSB CK2422301, R BSB CK2422303, L BTB CK2422302, R BTB CK2422304 | P300/P400 |
| Aft EPC | LAEPC CK2441372 (430 A Type II, P700) | P700 |
| 28 V ties and TRU relays | L DCT CK2432807, R DCT CK2432808 (400 A), L TRU RLY CK2432506, R TRU RLY CK2432504, C1 TRU RELAY CK2432505, C1 TRU ISO CK2432501, BB ISO RLY CK2432502, BB RLY CK2432507 (P200, 15 A) | P300/P400/P200 |
| Instrument ties, battery | CIT CK2432803, FIT CK2432804 (1PST NC, 400 A), MBR CK2431801 (400 A, P300) | P300/P400 |
| RAT | RCB CK2433503 (65 A, P400) | P400 |
| CAC contactors | L1CC CK2415573, L2CC CK2415576, R1CC CK2415583, R2CC CK2415586 | P700/P800 |
| Ram fan contactors | RFCL CK2415006, R1RFC CK2415007, L2RFCL CK2415008, RFCR CK2415009, L2RFCR CK2415010, R2RFC CK2415011 | P700/P800 |
| Other HVDC | CNGC CK2415505, R2HD CK2415507 (CK2415500), HNGC, NGC | P800 |
| SPU | SPUB CK2431387 (P800), SPUC 28 V 400 A (APU HBB, P49) | P800/P49 |
(R1b:25263–25272, R1b:CK-rows; R4 device table; B.)

Topology (R4, P:466):
- L3 BTB joins L1–L2; R3 BTB joins R1–R2; L2 BTB joins L2 to LTB; R1 BTB joins R1 to LTB; L1 BTB joins L1 to RTB; R2 BTB joins R2 to RTB.
- Each main bus: own GCB from its VFSG and own ATRUC; L1/L2 feed L ATUC (P500 ATU), R1/R2 feed R ATUC (P600 ATU).
- R2 → BB RLY (P200) → backup bus; RAT via RCB; BB ISO RLY in P400.
- GCU bus monitoring: L1 GCU watches VFSG L1 feeder + right tie bus; L2 GCU L2 feeder + left tie bus; R1 GCU R1 + left tie bus; R2 GCU R2 + right tie bus; L AGCU / R AGCU their ASG feeder + their tie bus (E:13588–13594, E:13575–13584).
- AGCU failsafe locks out its tie bus via L2 BTB and R1 BTB (E:13580).

Source priority per bus (rank, ground/air; P:459–506):
| Bus | Order |
|---|---|
| L1 | VFSG L1 (1), ASG R (2), VFSG L2 (3), ASG L (4/5), VFSG R2 (5/4), VFSG R1 (6), L fwd EP (7 GO / 9 GH), R fwd EP (8 / 10) |
| L2 (air) | VFSG L2, ASG L, VFSG L1, VFSG R1, VFSG R2, ASG R, L fwd EP (7/9), R fwd EP (8/10) |
| R1 | VFSG R1 (1), ASG L (2), VFSG R2 (3), ASG R (4/5), VFSG L2 (5/4), VFSG L1 (6), L fwd EP (7/9), R fwd EP (8/10) |
| R2 (air) | VFSG R2, ASG R, VFSG R1, VFSG L1, VFSG L2, ASG L, L fwd EP (7/9), R fwd EP (8/10) |
| LTB | ASG L, VFSG L2, VFSG L1, VFSG R1, VFSG R2, ASG R, R fwd EP (7/9), L fwd EP (8/10) |
| RTB | ranks not fully read **(verify)**; ASG R first by mirror |
- With engine start on L+R fwd EP only, or all 3 EP only: left EP is the L2 source, LTB/RTB unpowered (P:473–474, P:491–498).

Contactor types (P:1047): Type I 3PST 150/260 A, Type II 3PST 430 A, Type III 3PST 65 A.
Fail-to-close: aux contact still open 150 ms (BB iso rly, BTBs, DC ties) or 190 ms (BSBs, TRU relays) after command (E:23081, E:23068). GCB command/state disagreement > 150 ms (eps24_1).
Dual-controlled relays: L TRU, R TRU, C1 TRU iso, C1 TRU (E:7793). EPC chatter: > 4 state changes in 5 s trips (E:23099).

## Hardware: panels
| Panel | Location | Contents | Cite |
|---|---|---|---|
| P100 / P200 | aft EE bay | L / R 235 VAC power; L1+L2 / R1+R2 GCUs, ELCU, 2 OVTPU, GCBs, BTBs, SC, GNR, GEC, 5 ELCU_P cards each | E:502, E:18036 |
| P150 | aft EE bay | APU: L and R AGCU, APB, APU SC, GNR, GEC contactors; no ELCU | eps24_1, S |
| P300 / P400 | fwd EE bay | 115 VAC + 28 VDC + backup bus; BPCU, ELCU, SPDU, TRU relays, TCBs, ELCCs; P300 also WH1 heater control | E:934, A:418 |
| P500 / P600 | | conversion: ATU, TRUs, EBPSUs | A:419, E:934, E:936 |
| P700 / P800 | aft EE bay | HVDC: ATRUs, CMSCs, RFMC, 6 contactors each | A:420, E:938, E:940 |
| P49 | aft EE bay below P150 | APU start/aux panel; APU HBB; manual RAT deploy and tow relays; TCBs (not monitored) | eps24_3, E:30509, E:6125 |
| ESIC | flight deck | P5 overhead panel (GEN CTRL, DRIVE, EXT PWR, BATTERY, etc.), MFD pages | A:135 |
- No TCBs in P100/P200. TCBs are in P300/P400, P500/P600, P49 (A:410, E:6126).
- RAT GCU in fwd EE bay (A:5144).

## Hardware: controllers
| Unit | Count | Data | Cite |
|---|---|---|---|
| Main GCU | 4 (2 in P100, 2 in P200) | per VFSG; 3-PWB LRM, 8 lb; A1 micro, A2 regulation, A3 PMG interface; forced-air cooled, backup fans; TTP/C to BPCU | E:13010–13029, E:12992 |
| AGCU | 2 (P150) | per ASG | eps24_1 |
| RAT GCU | 1 (fwd EE bay) | closes RCB at ≥ 380 Hz; 230 V ±4 V for 0–10 kVA; UF, OV, high-phase takeover | A:5141–5162 |
| GCU supply | | all excitation from PMG; also both hot battery buses (protection, not regulation) | E:12974, E:12986, E:12989 |
| GCU position | | 4 pin-program discretes; invalid → failsafe | E:13320 |
| BPCU | 2 (P300, P400), same PN | 3 PWB, convection + backup fan; TTP/C to GCU/SPDU/ELCU; CDN gateway; either takes over | A:1033, A:1069, A:1071, F:81108 |
| ELCU | 4 (P100, P200, P300, P400) | ELCU_C (TTP/C + 2 CAN) + up to 5 ELCU_P cards, 6 ELCMs each; P100/P200 active/backup control pair | A:4376–4379, E:31073, E:31286 |
| LPU | in power panels | loss removes both TTP/C buses → SPDU/ELCU loads default | E:30237 |
| ESIC/CBIC software | in CCR | hosted in GPMs; see below | E:33427 |

ELCU channel map (figure P:909; each ELCU_P card has 6 ELCMs, 18 channels per ELCU max):
| Card | Channels |
|---|---|
| P300 PL3.1 | AC feeders RPDU 21, 31, 41 (OC + DP); 3 spare |
| P300 PL3.2 | L DCT, L BTB, galley power AC-DR 2 (OC, DP), AC feeders RPDU 71, 73, 75 (OC) |
| P300 PL3.3 | AC feeder RPDU 33 (OC, DP), spare, window heat 1L fail-safe, C1 TRU ISO RLY, L TRU RLY, L BSB |
| P400 PR4.1 | AC feeds RPDU 22, 32, 42 (OC, DP), pitot probe heater (OC monitored), 2 spare |
| P400 PR4.2 | R DCT, R BTB, galley power AC-DR 1 (OC, DP), AC feeds RPDU 72, 74, 76 (OC) |
| P400 PR4.3 | AC feeds RPDU 34 and 92 (OC, DP), window heat 1R fail-safe, spare, R TRU RLY, R BSB |
| P100 PL1.1 | L1 ATRUC (OC, GFD, UC, DC), WIPS L2 (OC, DP), PECS pump L1, EMCU slat, FAN-MISC CLG, spare |
| P100 PL1.2 | L ATUC (OC, DP), lav/galley fan L, supply fan fwd equipment cooling L, upper recirc fan fwd, supply fan aft equipment cooling 1, fwd L VF boost pump |
| P100 PL1.3 | L2 ATRUC, WIPS L1, spare, PECS pump L2, ICS pump 1, fwd R VF boost pump |
| P100 PL1.4 | GATU (OC, DP), spoiler actuators 5 and 10, L stabilizer, lower recirc fan L, cargo refrigeration liquid pump 1 |
| P100 PL1.5 | ICS compressors 1 and 3, vent fan aft equipment cooling, cargo refrig liquid pump, vacuum blower L, boost fan FCAC |
| P200 PR2.1 | R1 ATRUC, WIPS R2 (P150), PECS pump R1, EMCU flap, 2 spare |
| P200 PR2.2 | R ATUC (OC, DP), vent fan fwd equipment cooling, lav/galley fan R, aft R VF boost pump, supply fans fwd/aft equipment cooling R |
| P200 PR2.3 | R2 ATRUC, WIPS R1, BB RLY, PECS pump R2, ICS pump 2, aft L VF boost pump |
| P200 PR2.4 | not used, spoiler actuators 4 and 11, R stabilizer, lower recirc fan R, spare |
| P200 PR2.5 | exhaust fan fwd cargo heat, cargo refrig compressor 2, ICS compressors, SCU 2 and 4 **(verify full list)** |
- Protection per channel: ATRUC = OC + ground fault (GFD) + UC + DC content; RPDU and WIPS feeders = OC + DP; VF boost pumps = OC + GFD (P:909).
- ELCU data to system: ECB and ESW state, load/ATE override, comms default, load current A/B/C, ESW fail short/open, load fail open, data valid (E:31459–31465).
- Path: BPCU → ELCU_C over PPDN → ELCU_P over CAN → ELCC; aux contacts back the same way (E:31461, E:31464, E:31468, E:31469).

## Hardware: RPDS (RPDU ×17)
| RPDU | Type | Location | Main DC / AC feed | Cite |
|---|---|---|---|---|
| 71 | Gateway | fwd EE bay (left) | L 28 V (P300) + L 115 VAC via ELCC; standby DC P300 SPDU slot 5 (capt instr) | P:955, E:33399 |
| 72 | Gateway | fwd EE bay (right) | R side; standby DC P400 SPDU slot 3 | P:955, E:33404 |
| 73, 75 | Standard | fwd EE bay (left) | L; 73 standby DC P300 SPDU slot 3 | P:955, E:33401 |
| 74, 76 | Standard | fwd EE bay (right) | R | P:955 |
| 81 | Gateway | aft EE bay (left) | L 28 V; no AC feed; capt instr secondary | P:955, A:1960 |
| 92 | Standard | aft EE bay (right) | R | P:955 |
| 82 | Gateway | aft cargo compartment | R 28 V; no AC feed; F/O instr secondary | P:955, A:1960 |
| 21, 31, 33, 41 | Standard | crown (left; 20s by door 2, 30s door 3, 40s door 4 galleys) | L | P:955, E:33082–33085 |
| 22, 32, 34, 42 | Standard | crown (right) | R | P:955 |
- Count: 17 = 4 gateway (71, 72, 81, 82) + 13 standard; 6 fwd EE, 2 aft EE, 8 crown, 1 aft of aft cargo door (R1, E:3649–3655). The "aft cargo" RPDU is 82 (P:955).
- Odd = left, even = right (E:33077).
- Gateway = ARINC 664 CDN + TTP/C; standard = TTP/C only; standard talk to a gateway in clusters of ≤ 4 (E:33283, E:32995).
- 8 slots: 1 and 8 micro modules; 2–7 power modules (≤ 6 AC12 or DC19 mix) (E:33198).
  - AC12: 12 SSPCs (3×2.5 A, 3×5 A, 3×7.5 A, 3×10 A), 1/2/3 phase (E:33167, E:33180).
  - DC19: 19 SSPCs (2×10 A, 5×5 A, 12×2.5 A) (E:33173).
  - DC1 feeds slots 2–4, DC2 feeds 5–7 (E:33179).
- Inputs: main 115 VAC, main 28 VDC (TRU, no battery), standby 28 VDC (instrument bus, battery backed) (E:32981–32987).
  - Standard RPDUs without secondary DC get micro-only backup 28 V from the opposite RPDU (E:33278). Examples: 22 from 21 slot 2, 21 from 22 slot 2, 41 from 42 slot 3, 76 from 73 slot 3, 92 from 81 slot 3 (E:33405–33415).
- Micros: active/standby, swap each power-up; supply 3 V comms, 5 V SSPC control (E:33660, E:32837).
- Gateway/standard failure: single channel loss no degradation; full loss → loads default (on/off/last) after 3 s; essential loads have redundant RPDU (E:3640, E:3650, E:3651).
- Gateway loss drops its whole cluster (E:3653). EICAS: ELEC RPDU xx CHANNEL, ELEC RPDU GATEWAY xx CHAN (E:3637).
- RPDUs outside the fwd EE bay have AC differential protection, done by the ELCUs; 15 DC lightning units, 11 DPCT modules (E:33609, A:1946).
- Special feeds: RPDU 22 (left) and RPDU 21 (right) give the drive-disconnect 28 V; RPDU 33 excites the left ASG (E:13363, E:12211).
- Load count: > 900 loads under 10 A on RPDUs; > 150 28 V loads under 50 A on SPDUs (E:32957).

## Hardware: SPDU ×2 (P300, P400)
- 18 slots: 16 power modules + 2 micros, same micro as RPDU (A:1861, A:1995).
- Cards: AC-1 (one 45 A SSPC, window heat anti-ice), AC-2 (two 22.5 A, anti-fog), DC-2 (two 50 A), DC-4 (three 35 A + one 15 A), DC-19 (as RPDU) (A:1871–1877).
- 28 VDC to RPDUs and local loads; 115 VAC only to window heaters (A:1909).
- BPCU is the SPDU's CDN gateway (A:1638). The "4 SPDUs" line is a summary slip: 2 SPDUs (R1 conflict 2).

SPDU outputs (P300; P400 mirrors with 72, 74, 82; E:32455–32546):
| Connector / slot | Feeds |
|---|---|
| J1, J17, J18, J21 (DC19 slots 2, 17) | 28 V relay power: pumps, motors, pitot heaters, fuel valve control, crew call horn, RDCs |
| J2 (slot 3) | standby DC for RPDU 73 (P400: 72); 50 A (E:28752) |
| J3 (slots 3, 4) | spare DC; APU DC pump on P300 only |
| J5 (DC4 slot 5) | C1 FCE cabinet (P400: C2); standby DC for RPDU 71 (P400: 74) and 81 (P400: 82), 35 A each |
| J6 (AC1 slots 6, 7) | phases B and A 115 VAC for primary left window heat |
| J7 (AC2 slot 8) | 115 VAC backup right fwd and right side window heat |
| J8 (DC2 slot 9) | main DC for RPDU 73 and 71 (P400: 74, 72), 50 A |
| J9 (slot 10) | secondary DC right CCR cabinet; main DC RPDU 75, 50 A |
| J10 (slot 11) | main DC RPDU 81 and 33, 50 A |
| J11 (slot 12) | main DC RPDU 41 and 21 + spare, 50 A |
| J15 (DC4 slot 15) | left FCE backup 35 A; main DC RPDU 31 35 A; flight-deck boost fan 15 A on J13 |
| J14, J16 | spare |
- Micros: active/standby, comms loss → output default On/Off/Last; P1 gives status to CDN through the BPCU (E:32598–32607). Slot 1 micro M2409032, slot 18 micro M2409033 (E:27829, E:27821).

SSPC arc-fault logic (window-heat doc, E:35586–35616):
| Mode | DC | AC |
|---|---|---|
| High level | current > SSPC rating for > 20 ms total in a 100 ms window | 5 cycles in a 100 ms window, current < 10× rating |
| Low level (loose wire) | 5 cycles in a 5 min window, current > 150% of rating | 5 cycles in 5 min, current < rating |
- Detector sits on the SSPC card; on arc the card sends one bit; the fail-safe relay or anti-ice contactor then disconnects and lights INOP; cycling the switch resets (E:35595, E:35586).
- Heater checks: shorted = 3 consecutive instantaneous > 55 A (fwd anti-ice); open = < 10 A with SSPC on; window sensor shorted ≤ 200 Ω, open ≥ 350 Ω (E:35642, E:35634, E:35608, E:35616).

RDC secondary power (ata42:520–640): RDC secondary feeds come through RPDU ECBs: RDC 7 from RPDU 81, 3 from 73, 13 from 75, 9 from 33, 10 from 34, 8 from 82, 4 from 74, 17 from 41, 15 from 21, 11 and 1 from 71, 12/2/14 from 72; primary for RDC 16/6/18/23/19 from the F/O instrument bus via RPDU 82; for 17/15/5 from the captain bus via RPDU 81; RDC 21 from both buses via P300 ECBs. Left ARS secondary from the right 28 V bus via RPDUs 32, 34, 74; right ARS from the left bus via 32, 33, 73.

BITE and software hosts:
- Fault states ACTIVE / NOT ACTIVE / LATCHED, hard or intermittent; LRU NVM keeps the last 64 flight legs (E:7496, E:7519).
- Standard RPDU → gateway over TTP/C → CDN → CMCF; GCU, ELCU, SPDU → BPCU over TTP/C (E:7504, E:7509).
- Window-heat applications WHCF (3 instances L4, L6, R5) and WHOM (L1, L6, R4) on 5 GPMs; WHCF manages ESW/ECB data to SPDUs, ELCUs, RPDUs (E:7293, E:7312, E:7122, E:7297).
- RPDS = 4 clusters, one gateway each; RPDUs take 115 VAC from the P300/P400 ELCUs and 28 VDC from the SPDUs; ground-service switch also controls SSPCs (E:7053, S:7667, S:7689, S:7680).

## Hardware: circuit breakers
| Type | ID | Where | Behaviour | Cite |
|---|---|---|---|---|
| ECB (electronic) | CE + 7 digits | SSPCs in RPDU/SPDU, flight-control SSRs | I²t at rating, instant trip, arc fault, fusible link equal to rating, load shed | E:6012, E:33699–33704, E:34134 |
| ELCC (contactor) | CK + 7 digits | ELCU-driven contactors | OC (6 curve types), differential (20 ±10 A or 14 ±6 A), ground fault, unbalance | E:6009, E:31934–31953 |
| TCB (thermal) | CT + 7 digits | P300/P400, P500/P600, P49 | manual only, indication-only on CBIC; those ≥ 75 A and P49 unmonitored | E:6022, E:6125 |
- ID digits 1–2 = ATA chapter (R1 conflict 4).
- Dataset in B: 375 IDs (CE 109, CK 156, CT 110) with name, panel, rating, grade (258 NAMED, 101 ID-ONLY, 15 CONFLICT, most conflicts now settled in R1b). Full load path in L (1327 rows).
- By panel (B counts): CT P300 61, P400 30, P49 14; CK P100 45, P200 41, P400 19, P300 18, P150 13, P800 9, P700 5; CE P300 35, P400 35, SPDU/RPDU-resident 38 (panel unread).
- By ATA digits (B): 24 electrical 173, 21 air 59, 42 CCS 27, 27 flight controls 14, 30 ice 14, 28 fuel 11, 32 gear 10, 29 hydraulics 8, others ≤ 6.
- The ~900 RPDU SSPC channels beyond these rows come from the CBIC list and RPDU module cards (DC19 19 ch, AC12 12 ch per module): generated from `aircraft/24-electrical/sspc.*` as the list completes; entries missing from B are `ID-ONLY` or absent **(verify)**.
- Every breaker is one entity feeding a real load (rule of this plan). Datasets generate them; no breaker without a load.
- SSPC leakage when off: 28 V modules < 0.1–2 mA, < 6–14 V; 115 V modules < 2–15 mA, < 18–30 V (E:33835–33841).
- SSPC status bits: Load Feed UV, Arc Fault, ESW Failed Open/Short, Comms Default, Load Override (shed), ATE Override, Load Failed Open, Load Data Valid (E:34138–34146).

## Hardware: crew interface
| Control / indication | Behaviour | Cite |
|---|---|---|
| BATTERY switch (latching, P5) | ON starts L CCR; OFF de-energises on-ground battery-only after 30 s (A) / ≤ 2 min (F) — conflict | A:800, A:804, F:79662 |
| GEN CTRL L1/L2/R1/R2, APU GEN L/R | latched ON = excitation + GCB close; OFF opens GCB, resets protection; hardwire to GCU, CDN backup | E:1132, E:1218, E:1219 |
| GEN OFF light | GCB sensed open | E:1228 |
| DRIVE disconnect (guarded) | solenoid; lamp on when LOP / bad bearing; in flight only when lamp lit | E:1036, E:1196, E:1229 |
| FWD EXT PWR L/R, AFT EXT PWR (AVAIL, ON) | AVAIL = connected + quality OK; push closes EPC | F:79692, F:9399 |
| TOWING POWER, TEST | batteries only; HIGH/MED/LOW lights | F:79793, F:79814 |
| BATTERY refuel switch, ground service switch | at refuel panel; Ground Service mode | E:21663, F:81196 |
| CABIN/UTILITY, IFE/PASS SEATS | shed switches | F:6163, F:6167 |
| MANUAL RAT DEPLOY | hydraulic panel; relay K3 via P49 | F:80781, E:30509 |
| Battery charge lights | HIGH ≥ 60 min, > 30.1 V; MED ≥ 30 min, 29.7–30.1 V; LOW ≥ 15 min, < 29.7 V | E:1614–1617 |
| ELEC synoptic | SYS then ELEC; bus amber off / green on; flow lines green with arrow when flowing, white when not; start-power arrow; LOAD SHED box (HYD, FUEL, AIR, WINDOW HEAT, MISC HEATERS, EXT LIGHTS, COMM/NAV, CABIN), newest at top | F:63833, F:69080, F:80300, E:35040, A:5769 |
| CBIC pages | CB STATUS (≤ 15 ECBs), BY STATE, SEARCH, CUSTOM (maintenance only), RECENT ≤ 30; states OPEN/TRIPPED/CLOSED/LOCKED/UNKNOWN | E:5872, E:6346, E:5957 |

EICAS messages (from F, E):
| Message | Level | Condition | Cite |
|---|---|---|---|
| ELEC AC BUS L1/L2/R1/R2 | caution + beeper | bus not energized | F:80206 |
| ELEC GEN OFF L1 L2 R1 R2 | advisory | GCB open (also GEN CTRL OFF with engine running) | F:79706, F:79937 |
| ELEC GEN OFF APU L,R | advisory | APB open | F:6165 |
| ELEC GEN DRIVE L1.. | advisory | drive fault, only with engine running | F:81472, E:2069 |
| ELEC GEN DEBRIS | | cleared by GEN CTRL cycle | E:2426 |
| ELEC TRU L/R/C1/C2 | status | C1 < 11 A while AC supplies; C2 < 11 A with BB supplied; ATUC failed open | E:2529, E:2539, E:2555 |
| ELEC BACKUP BUS | status | < 130 Vrms 500 ms with source online; reset > 165 V | E:2234, E:2236 |
| ELEC STANDBY SYS | advisory | ground contactor faults BB iso, C1 TRU rly/iso, RCB, CIT, FIT, MBR | E:2502, F:21231 |
| ELEC BATTERY OFF | advisory | switch OFF ≥ 1 s; resets ON ≥ 1 s | E:2238, E:2239 |
| MAIN BATTERY DISCH | advisory | ≥ 30 A 20 s no transfer pending; or 3–30 A 10 min | E:2580, E:2581 |
| MAIN BATTERY LOW | advisory | HBB < 22 V 1.5 s (reset > 24 V); or SOC below dispatch | E:2391, E:2392, E:2584 |
| MAIN BATTERY / APU BATTERY | advisory | failed (short, imbalance, BMU) | F:12104, F:9497 |
| ELEC APU BATTERY CHARGER / OVERHEAT | status | charger fail 1 s / over-temp 1 s | E:2201, E:2203 |
| ELEC APU START PWR UNIT | status | SPU short or internal | E:2218 |
| ELEC HVDC SYS | | ATRUC trip-request power-up test failure | E:8184 |
| ELEC 115 VAC BUS L/R | status | < 50 Vrms 500 ms; reset > 85 V | E:2179, E:2182 |
| ELEC 28 VDC BUS L/R | status | < 18 V 1.38 s; reset > 22 V | E:2184, E:2185 |
| CAPT / F/O FLT INSTR BUS | | < 22 V 1.5 s; reset > 25 V | E:2279, E:2280 |
| ELEC CABIN/UTIL OFF, IFE/SEATS OFF | advisory | switch off | F:6145 |
| RAT GEN HEAT | status | both heater elements failed (MMEL 3 days) | E:3950 |
| ENG STARTERS | advisory | both starters on an engine failed | F:79976 |

More EICAS messages (E):
| Message | Condition | Cite |
|---|---|---|
| ELEC GEN CTRL UNIT (status, per GCU) | GCU inoperative, failsafe included | E:2277 |
| ELEC GEN DEBRIS | GCU lost one TTP/C channel; GCU fan inop; invalid GCU data; chip detector debris or ECD inop | E:2373, E:2279, E:2383 |
| ELEC GEN DRIVE | VFSG low oil pressure or both PMG inputs lost; stays set after disconnect; resets when oil OK and PMG OK after start | E:2393, E:2395, E:1232 |
| ELEC GEN OFF VFSG | GCB open without a preventing condition; GCB state disagrees with command > 150 ms; PMG shorted; GCU 28 V switch power fail 1.5 s; clears when VFSG power ready after start | E:2425, E:2420, E:2454, E:2455, E:2388 |
| ELEC GEN OFF APU | APB commanded closed and sensed open; APU GEN switch OFF with APU at 95% for 2 s; APB disagreement > 150 ms; ASG PMG shorted; AGCU 28 V fail 1.5 s | E:2416, E:2414, E:2420 |
| ELEC GEN SYS / SYS APU | VFSG oil switch shows no LOP after shutdown; protective trip of a bus tie or source breaker; APU GCU inop or TTP/C failure | E:2454, E:2248, E:2273 |
| ELEC GEN 1/2 AIR/OIL VALVE | oil cooler air valve not at command, or both position sensors failed | E:2381 |
| ELEC APU STARTER / START SYS | SPU main output or housekeeping fault; ASG, GNR, SC, CMSC path fault | E:2291, E:2294 |
| ELEC ENGINE STARTER, ENG STARTERS | one starter system inop (longer start); MES not possible after START selected | E:2942, E:2642 |
| ELEC MC HYD C1/C2, L/R | EMP CMSC internal fault or redundant CAN comms loss | E:2505, E:2507 |
| ELEC BUS PWR CTRL UNIT | BPCU fan test failure | E:8062 |

Maintenance page (E:4724–5003):
- Page 1 sources, page 2 motor controllers and bus data.
- VFSG/ASG load as fraction of maximum continuous (example L1 VFSG 0.16 = 41.6 kVA; ASG 0.40 = 90 kVA); voltage = 3-phase average; frequency = phase A.
- MMEL: VFSG system may be inoperative only with the APU available and run continuously; ASG flight within 180 min of landing; AGCU inop allowed within 180 min (E:2904, E:2891, E:3017).

Switch details (R7):
- BATTERY, IFE/PASS SEATS, CABIN/UTILITY: latching; OFF lights amber (F:79679, F:79690, F:79754, E:1511).
- FWD/AFT EXT PWR: momentary; AVAIL and ON lights white; push with AVAIL closes the EPC, push with ON opens it; pressing resets an EP protective trip (F:79693, F:79694, E:1311, E:1272).
- GEN CTRL: normally left latched ON; OFF light amber = GCB open; cycling resets the GCU. APU GEN: OFF light amber = APB open or switch OFF; cycling resets the AGCU (F:79733, E:1149, E:1179, E:1367, E:1401).
- APU selector: 3-position OFF/ON/START, START momentary spring to ON (E:1371, E:1373).
- GROUND SERVICE: momentary at the attendant panel (door 1L); lamp goes off if any generating source commands on or EP goes ON in general operation (E:1551, E:1562, E:1564, E:1567).
- Battery refuel switch on refuel panel M21000 outboard of the left engine (E:1669).
- Receptacle lights (2 per receptacle): CONNECTED, NOT IN USE; NOT IN USE lit when connected and EPC open or ground handling; both go off when the EPC closes (E:3447, E:18302, A:1164). Aft receptacle lit by the L2 GCU (A:1151).
- Fwd EP goes from ON to AVAIL when the APU or an engine starts (F:81000). With two fwd sources, the first connected is used (F:80744).
- In air, the MBR closes on loss of captain bus regardless of switch (E:1440).

Synoptic symbology (R7):
- Engine generator: green = engine running, GEN CTRL ON, powering buses; white = not running or GEN CTRL OFF; amber = ON but not powering (OFF light + ELEC GEN OFF); white low intensity = invalid (F:79929–79969).
- Drive: green connected and running; white engine off; amber DRIVE text on fault; amber low-intensity box with X when disconnected (F:80009–80048, A:5112).
- Starters: white box with green arrow; both failed: amber STARTERS text + EICAS ENG STARTERS (F:79941, F:79977, F:79979). Start arrows from the LARGE MOTOR POWER SYS; R START for right engine (F:80304, F:80320).
- APU gen: green on / available; white off; amber failed (F:80084–80126). APU starter shows L or R (F:80346).
- Fwd EP: green on / available; white off. Aft EP: green when powering motor controllers (F:80157–80227).
- Buses L1/L2/R1/R2: green powered, amber off, white low-intensity invalid; fault gives ELEC AC BUS (F:80245–80261). 115 VAC buses are not on the synoptic (A:5075).
- Load shed list in a cyan box; AIR, FUEL, HYD synoptics show LOAD SHED beside shed equipment (A:5158, F:81321).
- Battery: VOLTS, AMPS, CHG/DISCH, whole numbers, blank if invalid; DISCH also gives MAIN BATTERY DISCH (F:80431–80448, A:5124).
- Source boxes: green on, white outline off, amber with X fault (A:5049).

CBIC symbology (R7):
- Closed green (ECB and TCB); ECB tripped amber + TRIPPED; ECB open white plain; TCB open white with cross-hatched bar; UNKNOWN amber text (F:80579, P:1604, F:80610).
- Collared INOP: blue bar, cyan collar, white INOP tag; DO NOT CLOSE: red tag, cyan collar (F:80598).
- Pages: SYS then CB key; menu FLIGHT DECK CB, OPEN/TRIP CB, BY STATE, SEARCH, BY ATA, BY LOCATION, RECENT USED, CUSTOM LIST (F:80481, F:81343).
- In flight open/close: CTRL, OPEN or CLOSE, confirm window, ENTER on the MFK; crew cycling not recommended (F:81389, F:81427, F:81354).
- Status messages inhibited from engine start to 30 min after lift-off; advisory = amber indented, no annunciation; memo = white (E:2043, E:1917, F:117451, F:117582).
- APU RUNNING memo at 95% speed + 2 s; APU SHUTDOWN advisory on failed start (F:83266, F:83332).

## Physics
- **Generators (VFSG/ASG).**
  - Six-pole machine: POR Hz = rpm / 20 (7400 rpm = 370 Hz), range 360–800 Hz (E:9636, E:17599).
  - Terminal voltage from exciter and PMG-fed regulator, with sub-transient dynamics and load-dependent droop; regulates 235 VAC.
  - Overload: thermal state from kVA vs the 250/312.5/437.5 curve; limiter builds heat in generator and oil.
  - Shorted rotating diode: DC link climbs toward 800 V, unregulated (E:17886).
  - Torque in start mode from the CMSC; mode set STOP/START/MOTOR/GENERATOR (E:9742).
- **Exciter chain in start (GEC1/2/GNR/SC).** Thresholds in the software section. In start the exciter is fed from the 235 V bus (GEC1 phase C, GEC2 phases A+B) or the SPU AC (right ASG), then from the GCU exciter drive in generate mode (GEC3) (E:13536, E:13598, E:13664, E:12211).
- **RAT.** 10 kVA at 230 V; speed depends on airspeed, torque from the hydraulic and generator loads. Output frequency must reach 380 Hz for RCB close (A:5154).
- **Converters.**
  - ATRU: ±250–320 V, ~0.95-class efficiency **(verify)**; losses into PECS coolant.
  - TRU: 28 V; overcurrent tiers below.
  - ATU: autotransformer; ground step-up for ext power.
  - CMSC: DC link ±270 V, inverter with soft-start inrush control and ground-fault interrupt (A:2602).
  - SPU: 80% minimum efficiency (A:2941).
  - All converters are constant-power loads on their input; collapse below input limits.
- **Batteries (per cell).**
  - Open-circuit voltage curve vs SOC, resistance vs temperature/SOC/age, heat I²R, thermal coupling cell-to-cell and to the stainless-steel case (1/8 in) (S:7508).
  - Charger: 46 A, 32.2 V regulated from 28 V instrument bus; charged at 31 V and < 5 A (A:694, A:888); 1–90 min full charge; BCU 20% in 1.5 h (S:7520, A:887).
  - Enclosure: vent tube; burst disc at 18 psig (A:638). Thermal runaway = failure (below).
  - Battery diode module blocks charge when HBB is paralleled with another source: 75 A breaker (A:760–765).
- **Loads.** Constant power (converters, motors under control), constant resistance (heaters, window heat), motors with inrush, lighting. Each load drops out at its own brown-out threshold (power quality "no UV/OV/UF/OF for 150 ±10 ms" = ready, E:19006).
- **Wires.** Per-segment R and L, gauge, length, zone temperature; wire losses into zone.
- **Contactors.** Operating time, coil pickup/hold, bounce, weld, aux contact timing (fail-to-close 150/190 ms).
- **No waveforms.** Harmonic and DC-content numbers only where a monitor reads them (DC content 32/50 A, E:17643, E:17053).

## Software behaviour
### GCU (per VFSG/ASG)
Protection table (VFSG unless noted); every trip disables regulator, then trips GEC3 and GCB/APB, locked out until GEN CTRL OFF-ON (E:17441, E:13669).
| Protection | Threshold / time | Cite |
|---|---|---|
| Underspeed | PMG < 1083 Hz (361 Hz POR, 7220 rpm); GCB open ≤ 70 ms | E:18078, E:18080 |
| Underfrequency | trips GEC3 + GCB within 4 s at/above idle with underspeed; inhibited in engine start; ASG UF1 PMG ≤ 1624.5 Hz in 1.5 s | E:18088, E:18091, E:17613 |
| Overfrequency | PMG > 2397 Hz in 100 ms; reset 2391 Hz; ASG OF2 > 2065.5 Hz (459 Hz POR) in 100 ms, reset 2052 | E:18073, E:18130, E:17684, E:18223 |
| Overcurrent | > 375 A with inverse time 5.1–330 s (ASG 340 ±14 A) | E:17665, E:17672, E:17716 |
| OC inhibits | UV at ≥ 620 A; UPF/open-CT at ≥ 444 A (ASG 399 A); OC blocks power transfer until fault isolated | E:17714, E:17723, E:17725 |
| Open phase | lowest < 11 A and next > 55 A above; 4.0 s; reset > 20 A or < 45 A; locks out start | E:17745, E:17782, E:17749 |
| Differential | gen vs feeder > 30 A (< 450 A) / 45 A (≥ 450 A); 30 ms delay, ≤ 70 ms; reset < 10/15 A | E:17747, E:17749, E:17793 |
| Open gen CT | gen ≤ 11 A and feeder > 30 A more; 30 ms, ≤ 70 ms; reset > 20 A | E:17821, E:17826, E:17861 |
| Unbalanced parallel feeder | > 45 A between two feeders, 2.75 s; reset < 17 A | E:17947, E:17984 |
| Shorted rotating diode | DC link > 700 V for 15 ms, trip ≤ 70 ms; trips GCB on 2 events in 60 s or 3 per flight | E:17921, E:17894, E:18032 |
| Tie-bus differential | 40 A (text 30 A) for 210 ms, trips ≤ 250 ms; VFSG GCU opens ring BTB, ASG GCU opens APB; reset < 20 A | E:18020, E:18053 |
| DC content | any phase > 50 A for 175 ms trips GCB/BTB/ATUC; reset < 32 A | E:17053, E:17643 |
| OVTPU | resets 200 ms; POR collapses, current decays to ~35 A; SRD inhibit lifts 10 ms, DP at once, UV 100 ms | E:18086, E:18098 |
| Exciter field driver fault | 50 ms | E:17102 |
| Unlike-source parallel | 0.2 s, opens last source online | E:16870 |
| ASG ready to load | 2 s after 95% APU speed; underspeed trips APB in 70 ms | E:18181, E:17649 |
| Failsafe (CPU fail, bad pin, all discretes common) | de-energise contactor drivers, regulator off, TTP/C silent; reset GEN CTRL OFF-ON | E:18257–18276 |
| Drive disconnect | conditions true 50 ms; GCB lockout 50 ms; solenoid ground 100 ms; RPDU SSPC 5 s pulse high side, GCU 4 s ground low side; CCR 5 s pulse via BPCU; inhibited with fuel CUTOFF; max 3 s push, wait 60 s | E:13448, E:13008, E:13357, E:13360, E:13346, E:13367, A:4051 |
| Auto drive disc | LOP or PMG loss + underfrequency trip + LOP > 500 ms | E:13459, E:1232 |
| Isolation sequence | on OC, GCU opens contactors in order to find the fault, then reconfigures | E:17675 |

Exciter relays and start sequencing (E):
- GEC1 (phase C of the 235 bus to exciter) closes in MES at PMG ≤ 585 Hz (3900 rpm), opens > 600 Hz (4000 rpm) (E:13544, E:13537).
- GEC2 (phases A,B) closes at PMG ≤ 825 Hz (5500 rpm), opens ≥ 1050 Hz (7000 rpm) (E:13605, E:13598).
- GEC3 (generate drive) latches closed at PMG ≥ 1055 Hz (352 Hz POR) with GCS latched ON; opens at ≤ 1020 Hz or VR disabled 10 ms with GCS OFF (E:13677, E:13697, E:13694).
- GNR opens (latched) in start at PMG ≤ 825 Hz; closes when start false or PMG ≥ 1050 Hz (E:13805, E:13814).
- SC closes latched at start mode and PMG ≤ 825 Hz (28 V from CMSC, ground from GCU); opens at ≥ 1050 Hz or mode false (E:13878, E:13884, A:3192).
- CMSC starter cutoff 6100 rpm (E:39428). Engine at/above idle at PMG 1110 Hz (370 Hz POR) (E:17600).
- VFSG stops start rotation at 46.7% N2 (RR); EEC lights one igniter above 6% N2 (A:4512, A:4511).

### Engine/APU start flow
1. Start switch (P5) to START: BPCU commands CMSCs to MES; CMSCs set P700/P800 contactors and send 28 V to start contactors (A:4469, A:4479).
2. CMSCs send OK TO START to BPCU/EEC; EEC sends RUN; CMSC sends START INITIATED (A:4475, A:3480, A:4480).
3. VFSG torque: 100% for 3 min then 50% (A:4497). Both VFSGs 55–60 s; single VFSG 75–80 s (E:2955).
4. EEC sends RUN STOP at the run condition; CMSC sends START COMPLETE; GCU/CMSC open SC, GCU closes GNR; start switch unlatches to NORM (A:4494, A:4487, A:4495, A:4522).
5. At 370 Hz the VFSG goes online (A:4454). Autostart only; no manual or pneumatic start (A:4454).
- Normal: APU running, both ASGs online; second engine from first VFSG + one ASG (E:2989).
- Alternatives: external power (≥ 2 fwd 90 kVA + load shed; 1 source cannot start; fwd + aft pair cannot); one VFSG + both ASGs; ASGs both failed → ext power + opposite VFSG (A:4428, A:1280, E:2997, E:3102).
- Aft EP only helps right-engine-first with both fwd connected (F:80748, F:80762).
- Simultaneous start: APU running and both ASG online; CACs all off 2 min (F:82782, F:69324).
- Load shed always occurs during start; 3 EP sources: non-normal start sheds ~2 min and cabin lighting/IFE cut 50% (F:74990, E:34872, E:34880).
- APU start: with no AC, APU battery → SPU (115 VAC) → R2 ATRU (±135 VDC) → R2 CMSC → right ASG only; SPU excites the right ASG, RPDU 33 the left (E:2978, E:2981, E:12211). SPUC/SPUB close via CMSC (A:2957).
  - BPCU rotates the start path daily: L1 CAC CMSC/L ASG, R2 CAC CMSC/R ASG, R2 EMP CMSC/R ASG (E:12219).
  - Duty: 3 × 52 s attempts, 60 s between, then 30 min cooldown (E:12256, E:9461).
  - Other APU start sources: external power, VFSG 235 VAC (A:516).

### BPCU (2)
- Controls 115 VAC / 28 VDC configurations, standby/backup bus, ext power, ground handling/service, load shed (A:1067, A:45).
- Either BPCU takes all functions if the other fails (F:81108).
- Commands: relays/breakers/contactors, their feedback; L BPCU primary for Maintenance mode (A:1023, E:34675).
- 115 VAC bus ties: separate with engines running or both fwd EP; if only one side live, both ties close (A:1835, A:1845).
- 28 V ties: L/R DCT in P300/P400, ELCU controlled; CIT/FIT instrument ties, BPCU controlled (A:1855, A:1858).
- TRU relays: L/R by ELCU from BPCU; C1 TRU rly and BB iso rly directly by BPCU; L/C1 TRU from left buses, R/C2 from right (A:1931, A:1893).
- Power-up reset commands all non-essential loads shed (A:5707).
- No BPCU/GCU confirm within 3 s of an ESIC command → TIME OUT (E:5613).
- ELCU answers digital commands within 350 ms of the BPCU gateway (E:31241); sets ELCCs to commanded state ≤ 10 s after CCS available (E:31337); LSAP mismatch keeps ELCC drivers off (E:31351).

### Modes
| Mode | Entry | Notes | Cite |
|---|---|---|---|
| On-ground battery only | BATTERY ON, nothing else | TCB loads on instrument buses only | F:81197, E:1424 |
| Ground handling | fwd EP connected, not selected | ~530 loads, priority over battery-only; ≤ 440 Hz loads | E:34477, F:81197, E:19013 |
| Ground service | GROUND SERVICE ON in ground handling | +160 loads | E:34467, E:34472 |
| External power on | ≥ 1 fwd EP selected ON, no generators | packs, cargo heat inoperative | F:81219, F:81245 |
| Towing power | TOWING POWER ON, battery OFF | towing equipment only | A:914, S:7047 |
| Normal | VFSG/ASG | | |
| In-air battery only | all lost, no RAT | like RAT-only without center pitot heat | F:81284 |
| In-air RAT only / standby | below | | |
| Maintenance (test enable) | GND TEST ENABLE, ground, ≥ 1 source | preemptive LM off, overload LM on, closes all ATRUCs; one 90 kVA EP: 1 EMP, packs off | E:34663–34673 |
| Shutdown | below | | |
| Battery refuel | refuel panel BATT | needs RCB open; ends with shutdown | E:21672, E:21739 |

Single main-bus loss (ELEC AC BUS NNC, Q:3346–3380 range; loads inoperative):
| Bus | Lost loads |
|---|---|
| L1 | C1 hydraulic pump, left aft fuel pump, captain and F/O HUDs |
| L2 | one or more spoilers, right demand hydraulic pump, right aft fuel pump, L2 stabilizer motor; wing anti-ice powered if APU started |
| R1 | C2 hydraulic pump, right fwd fuel pump, captain and F/O HUDs, forward cargo heat |
| R2 | one or more spoilers, left demand hydraulic pump, left fwd fuel pump, R2 stabilizer motor; wing anti-ice powered if APU started |
- With one engine running, no APU or EP: both of that engine's buses and the other two are powered via the tie ring (C, "one engine running" rows).
- Loss of IFE/PASS SEATS switch: IFE, pax seat systems, video surveillance, flight-deck PED outlets (C).
- Both fuel boost, hydraulic and flight-control loads above come from `L`, each tied to the ELCC that feeds it.

Source arbitration:
- Each bus fed from a single source; transfers can interrupt briefly (F:80648, A:4610).
- With both engines on, each main bus from its VFSG; failed VFSG bus from another main bus via BTB ring (F:81002, F:81015).
- APU only on ground: each ASG feeds two main buses (A:4587).
- Fwd EP only on ground: 235 buses through ATUs (F:80994); 115 bus direct (F:80732).
- Aft EP feeds L2 ±270 V bus through L2 ATRU for start, pack, EMPs (A:2443).
- Ground handling loads only from EP or ASG (A:1000).
- Fwd EP + APU: two ATUCs open (A:3040).
- ASG supplies the instrument bus when VFSGs and EPs offline (E:5513).

### Standby, battery-only, RAT
- MBR closes and latches to feed instrument buses; opens 15 s after command lost (E:21750).
- MBR close causes: power-down/transition, landing, transfer pending, EP without engine start, RAT deployed, APU shutdown, shutdown mode, instrument bus dead in air, battery refuel (E:21754–21763).
- HBB fault outside air/standby opens MBR (E:21762).
- Instrument buses paralleled by CIT before any transfer, until completion + 15 s; CIT closes 120 s on ground-to-air; normally closed (E:22203, E:22184, E:22189).
- Standby buses paralleled ≤ 50 ms when an instrument bus ≤ 24 V and TRU current < 360 A (E:22205, E:22208).
- FIT closes: RAT deployed, F/O bus dead in air, F/O bus dead with BATTERY ON, ASG-only cooldown; 100 ms in battery refuel shutdown (E:22305–22318).
- Transfer to battery-only: 30 ms interrupt of instrument buses; standby ↔ battery-only no interrupt (E:21906, E:19182).
- Standby entry: each BPCU requests RAT and enters standby 15 s after continuous loss to BB and both instrument buses (L1 lost + BB lost; L ATUC open + BB lost; C1 TRU iso + BB iso fail to close) (E:19765–19771). Also when RCB closed and no AC bus powered (E:19789).
- Standby reset: any AC bus powered by another source, or ground with battery switch OFF (R5 E:19207). Ends when RAT drops offline (E:19795).
- RAT deploys: C1/C2 TRU loss, loss of hydraulic power, all engines out, loss of all flight-instrument power; 4 EMPs lost + flight-control fault on approach or engine failure at takeoff/landing; manual (E:19774, A:4792–4795). Request via CDN; hydraulics deploy ≤ 200 ms; inhibited unless Air mode (E:19780, E:19783).
- RCB electrically interlocked with BB iso relay; checkout deploy cannot feed BB while R2 is available (E:19780, E:19776).
- RAT power: backup bus + C1/C2 TRU + center-pitot ATU; BB iso opens, C1 TRU iso opens, C1 TRU relay closes; only instrument buses from BB (A:1923, A:5103). If a VFSG/ASG commanded on while RAT powers TRUs, instrument buses stay on RAT until it drops (E:19783).
- Standby loads: captain DUs, MCP, autopilot, ACPs, VHF L, IRU/AHRU L/R, ISFD, FMC, PFC, CCR L, center pitot, fire detection. Inoperative: HF, SATCOM, TCAS, GPWS, transponder, WXR, external lights, WIPS, window heat, autothrottle, flaps, slats, stab trim, reversers (A:4736–4745, A:5072, F:81278).
- RAT stows on ground only, with the P57 stow switch and center hydraulic pressure (A29:3082). RAT heater: below 2 °C, 2 elements (E:3961).

### Shutdown and power-up
- 20 s shutdown starts in ground mode with no VFSG/ASG/fwd EP, BATTERY OFF, refuel NORMAL, APU and engines off (E:22091–22104).
- MBR stays closed through the 20 s; CCS stays powered 11 s after MBR opens (E:22172, E:22102).
- Shutdown cancelled if a source returns.
- BATTERY ON starts L CCR; OFF: CCR shut down, de-energise after 30 s (A:800, A:804). Fwd EP lost with BATTERY OFF: MBR closes 30 s (A:4858).
- Wait 30 min after EP off before main battery disconnect, else WELS comes on (A:586). Parking brake can't release with the battery disconnected (A:593).

### External power
- BPCUs sense phases at the source side of the EPC (E:23839).
  - OV: highest phase > 130 Vrms, inverse time ≤ 10 s, reset < 128 (E:23839, E:23844).
  - UV: lowest < 104.5 Vrms, 9 s; reset > 107.5 V (E:23860, E:23835).
  - OF > 429 Hz 1 s (reset 427); UF < 371 Hz 1 s (reset 373) (E:23955, E:23956).
  - Phase sequence A-B-C, trip in 100 ms (E:23492).
  - OC: 270 A 300 s; 326 A 5.2 s; reset 250/306 A; fault persisting 100 ms locks out BSB then EPC (P:674).
  - Open phase as GCU, 4 s (P:672). ATU differential 40 A, 60 ms delay, ≤ 100 ms (E:23585).
- Power ready = quality 150 ±10 ms + sequence + pin E/F interlock + no start (E:19006, E:19017).
- 300 ms before an AC contactor change on EP ON/OFF; EP commanded off when < 20 Vrms and < 10 A for 50 ms (E:18938, E:18955).
- Aft EP (L2 GCU): OV/UV/OF/UF/sequence/open phase/ATRU saturation/overtemp; fail-to-close 150 ms with < 10 A; DC > 50 A 175 ms locks out start (E:18296, E:18324, E:18318).
- Ground handling requires EP Ready and no VFSG/ASG commanded ON (E:19028).
- Recommended 2 × 90 kVA on the forward receptacles; one source covers typical gate turn with shedding (E:19023, A:5993). ACTIVE/DEFAULT levels 90/120/140/160/180 kVA (A:1283).

### Load management (BPCU)
- Triggers: source/bus change, overload, HPx exceedance, PECS temperature, cabin temperature, PCS faults (E:34334).
- Shed groups restore after a delay, CB reset, or added power (E:34353).
- EP: > 90 kVA for 5 min sheds sequentially every 5 s; > 112.5 kVA for 5 s sheds large groups (E:34415–34421).
- HPx (per-engine generator): exceedance sheds a preset group; second within 20 min adds a group (E:34410, E:34421).
- On overload with ASG/VFSG, BPCU cuts CAC speed to free power (E:34427).
- Thermal shed above 64 °C PECS: cabin lighting scene down, then IFE off; then CMSC self-protect (fuel O/J pump) (E:34439, E:34456, E:34449).
- With EP level 120–180 kVA: > 260 A for 4 min or 313 A for 5 s stops all CMSCs running an EMP (E:34537).
- Preemptive shed: C1 and C2 EMP together, gear stow, high-lift (E:34393). Disabled in maintenance mode.
- Left BPCU sheds left-side loads, right BPCU right-side (A:5984).
- Two fwd EP only: engine starts, all non-essential shed (E:34870).
- If OAT > 26 °C use a conditioned air cart (E:34510).

### CBIC (CCR hosted)
- Control levels: Limited (moving > 2 kt or airborne: reset only for ELCC), Ground (doors open or EP avail), Maintenance (GND TEST ENABLE) (E:5841–5847, E:6743).
- Needs ENTER key from the MKP to confirm (E:6113).
- Path: CDN ARINC 664 to the 4 gateway RPDUs (TTP/C to others), to BPCUs for SPDU/ELCC/TCB, to FCMs for flight-control SSRs (E:6093, E:6079, E:6114). FC changes only when not in flight (E:6120).
- ECB state machine: Closed → Tripped on OC or arc fault; Tripped needs Request Reset (E:6688). ELCC: OC/DP/GF trip; manual reset (E:6714). TCBs cannot be commanded; main-bus ELCCs have no CBIC (E:5944, E:6011).
- State and LOCK in NVM, validated against airplane sequence number (E:5950, E:6116).
- PDHA: 5 LSAPs (PDOM, PDUI, PDHM, PDSD, PDMD) + PDOT; on 2 GPMs in L CCR and 1 in R CCR (E:33427–33448). PDSD is a static SSPC table; each RPDU keeps SSPC state in NVM (E:33436, E:33450).
- SSPC command priority by criticality (E:33695).

### Battery logic (charger + BPCU)
- Fault (short, imbalance, BMU): L BPCU sees grounded charger discrete 15 s, passes MBR trip to R BPCU over TTP/C (E:21945–21959). Not in battery-only(air)/standby (E:21955).
- Discharge inhibit: ≥ 30 A 20 s (no transfer pending), 5–30 A 10 min, or HBB < 24 V 60 s; clears at < 5 A 1 s and HBB > 29 V 1.5 s (E:21949–21961).
- BDM blocks charging when HBB is paralleled (A:760).
- APU battery charged when a TRU powers the F/O bus (E:21636).

### Ground power-up, operational tests, ground handling (R4)
- Operational tests run at ground power-up before any EPC closes, 250 ms closed pulses (E:7839):
  - dual-controlled contactors: L3/R3 BTB, L/R BSB, L/R TRU, C1 TRU iso, C1 TRU rly, MBR, BB iso, CIT, FIT, L/R ATUC (E:7783).
  - L2 GCU tests L3 BTB and L ATUC before L1 GCU; R2 GCU tests R3 BTB before R1 GCU; R BPCU tests L BSB and C1 TRU iso; L BPCU tests R TRU rly (E:7808, E:7995, E:7836, E:7861, E:7885).
  - L/R BTB closed 250 ms after each air-to-ground transition; R DCT opened 250 ms then closed; R BPCU opens CIT 250 ms; MBR closed 250 ms by R BPCU (E:7908, E:7930, E:7956, E:8018).
  - Transfer requests deferred until tests end; a failed test gives an EICAS message (E:7791, E:7794).
  - BPCU fan test: L then R, fans 1 and 2, 5 s each; failure gives ELEC BUS PWR CTRL UNIT (E:8048, E:8062).
- Ground handling: virtual bus in BPCU software; entry needs fwd EP connected not ON, APU off, engines off; the first-connected fwd EPC closes (A:1238, A:1245, A:1230, R7 lot_fcom:80744).
  - CCS needs 3 min to power up before power is applied to the buses (A:1233).
  - APU HBB loads stay off and MBR stays open (A:1237).
  - GH loads keep power after EP ON or APU start (A:1252).
- EP ON, 2 fwd EP adds NGS, hydraulics, IFE, basic cabin, both PECS loops (A:2321).
- APU battery: 2 consecutive start attempts then 5 min cooling (A:3071) vs 3 attempts (E:21589): conflict 16 below.
- Start on 3 EP: R TRU re-sourced to R1 bus; center TRUs off during a Type I EP-only start (E:22453, E:21456).
- L DCT closes (latched) if the left or right 28 V bus is dead; R DCT needs R TRU current < 360 A (E:22446, E:22450, E:22625).
- RUN → CUTOFF enables power transfers and load management (E:1652).
- Flight-instrument supply: capt bus 1 = C1 TRU, 2 = F/O bus via tie, 3 = main battery via MBR; F/O bus 1 = C2 TRU, 2 = capt tie; MES with L+R fwd EP only: main battery sources the F/O bus (P:596, P:603).
- Aft EP ON opens L2 ATRUC by interlock; one source per ATRU (A:5113).

## Timing numbers (one table, all on the discrete-event clock)
| Event | Time | Cite |
|---|---|---|
| RAT: deploy request to command | ≤ 200 ms | E:19780 |
| RAT: deployed to power | 10 s | S:8185 |
| Standby entry after loss of BB + instrument buses | 15 s | E:19765 |
| MBR open after command lost | 15 s | E:21750 |
| Shutdown period (ground) | 20 s; CCS held 11 s after MBR opens | E:22091, E:22102 |
| Battery OFF de-energise | 30 s (A) / ≤ 2 min (F) | A:804, F:79662 |
| Battery to instrument bus transfer | 30 ms interrupt | E:21906 |
| CIT paralleling | ≤ 50 ms; held until transfer done + 15 s; 120 s on ground to air | E:22205, E:22203, E:22184 |
| FIT close in refuel shutdown | 100 ms | E:22315 |
| GCB trip: differential / open CT / SRD | 30 ms delay, ≤ 70 ms | E:17749, E:17826, E:17894 |
| Underspeed trip | 70 ms | E:18080 |
| OF trip | ≤ 100 ms | E:18073 |
| Tie-bus differential | 210 ms delay, ≤ 250 ms | E:18020 |
| DC content | 175 ms | E:17053 |
| ATRU overcurrent discrete | ≤ 5 ms; ground fault 50 ms; unbalance 370 ms | E:31805–31815 |
| CMSC standby overcurrent | 100 ms; ATRUC open request ≤ 10 ms; CAN loss 2 s | E:39612, E:39530 |
| Contactor fail-to-close | 150 ms (BTB, BB iso, DCT), 190 ms (BSB, TRU relays) | E:23081, E:23068 |
| EP power quality | 150 ±10 ms; AC contactor change delay 300 ms | E:19006, E:18938 |
| EP off detect | 50 ms (< 20 Vrms, < 10 A) | E:18955 |
| ELCU command response | ≤ 350 ms; set ELCCs ≤ 10 s after CCS | E:31241, E:31337 |
| BPCU/GCU command confirm | 3 s, else TIME OUT | E:5613 |
| RPDU default state | 3 s | E:3650 |
| Op test pulse | 250 ms | E:7839 |
| TRU overcurrent tiers | 280 A 305 s; 360 A 48 s; 420 A 12 s; 560 A 5.75 s; 900 A 2.6 s; 1200 A 715 ms | E:22953–22958 |
| Fwd EP OC | 270 A 300 s; 326 A 5.2 s; then 100 ms to lockout | E:22871, E:22872, P:674 |
| VFSG overload | 312.5 kVA 5 min; 437.5 kVA 5 s | E:9688, E:9689 |
| ATRU overload | 165 kVA 10 min; 187.5 kVA 5 min; 200 kVA 30 s; 245 kVA 5 s | A:6144–6147 |
| CMSC overload | 150 kW 3 min | E:36425 |
| Engine start | 55–60 s (2 VFSG), 75–80 s (1 VFSG); torque 3 min then 30 min at 50% | E:2955, A:4497 |
| APU start | 52 s max, 60 s between, 30 min after three | E:12256, E:9461 |
| Drive disconnect | 5 s high-side pulse; 4 s ground; 60 s rest; 3 s max push | E:13357, E:13360, A:4051 |
| VFSG oil level read | every 2.5 min | E:4787 |
| Status messages | bus 28 V 1.38 s; 115 V 500 ms; instrument bus 1.5 s; HBB 1.5 s | E:2184, E:2179, E:2279, E:2391 |
| PECS holdup after sources off | 10 min (APU or 2 fwd EP), 22 min (1 fwd EP) | A:2355, A:2357 |
| Cold-soak fire bottle warming | 1.5 h to −10 °F below −40 °F | F:4878 |

## Load numbers worth keeping
- Ground handling ~530 loads; ground service +160; > 900 RPDU loads < 10 A; > 150 SPDU 28 V loads < 50 A (E:34477, E:34472, E:32957).
- EP: 90 kVA per fwd connector; 5-min shed > 90 kVA; 5-s shed > 112.5 kVA; EMP stop > 260 A 4 min / 313 A 5 s (A:1283, E:34415, E:34420, E:34537).
- 28 V loads on TCBs are outside load management (A:1953, A:1970).
- Charger: 46 A, 32.2 V (A:694). SPU 13.5 kW (A:2974). WH, galley, cabin loads: from `L`, per-load numbers **(verify)** per entity.
- Window heat: SPDU AC-1 45 A anti-ice, AC-2 2 × 22.5 A anti-fog (A:1871, A:1873).
- Fwd EP receptacle current 260 A max continuous, aft 188 A (E:4895, E:4900).

## Links to other systems
| Link | Direction | Path | Cite |
|---|---|---|---|
| Engine start | CMSC ↔ EEC | CDN: OK TO START, RUN, RUN STOP, START COMPLETE; start switch magnetic latch | A:4475, A:3480, A:4494 |
| Engine gearbox | in | VFSG shaft speed, oil air valve (EEC) | E:2847 |
| Fuel | out | boost pumps 235 VAC (variable speed 360–800 Hz); O/J pumps from CMSC (230 VAC from 270 VDC); FSM 24 V | A28:2083, A28:2388, A28:2694 |
| Hydraulics | out | EMPs on CMSCs; demand relay R2HD; RAT deploy request, RAT pump | E:36699, E:19780 |
| ECS | out | CAC (CMSC, 270 VDC), ram fans, PECS pumps on 115 VAC | A:2344, A:1442 |
| APU | in/out | ASG speed and ready-to-load; APU start via SPU/CMSC | E:18181 |
| Thermal | out | converter, wire, battery losses into zones; PECS coolant | R3 |
| CCS / CDN | both | BPCU gateway, RPDU gateways, CBIC, ESIC; ARINC 664, TTP/C, CAN | E:33283, E:6044 |
| Displays | out | ELEC synoptic, EICAS, CBIC through ARINC graphics server | E:6054 |
| Fire | out | HBB feeds bottle squibs L1 D44, L2 D46, R1 D45, R2 D47 | E:28237 |
| Flight controls | out | VFSG flight-control PMG (primary); battery backups; RAT | C |
| Brakes | out | EBPSU | F:81060 |
| Gear | out | alternate extension motor on HBB (D31) | E:28225 |
| Anti-ice | out | WIPS heater mats on 235 VAC from 3 sources (ata28_33 rows in elec_structure); window heat via SPDU AC-1/AC-2 | A:1871, A:1873 |

## Solver (from shared-architecture, electrical)
- One network for the whole aircraft: every source, contactor, breaker, wire segment, connector contact, ground stud and load is an element.
- Sparse modified nodal analysis.
  - AC: unbalanced 3-phase phasors at each source's live frequency.
  - DC: 28 V and ±270 V networks.
  - Converters, CMSC, SPU averaged with loss and limit, coupling AC and DC.
- Islands: ring BTB loop → connected components after each topology change; solve each island alone; refactorise only on a topology change.
- Steps: 1 ms while anything changes (start, transfers, trips); asleep when steady.
- Dynamics: generator + exciter + regulator (GCU software), RAT shaft, battery cells, capacitor charge, motor/transformer inrush, contact bounce.
- Protection reads solved currents/voltages: I²t, instant trip, arc fault, differential, OV/UV/OF/UF, DC content, SRD.
- Sources of motion: engine/APU shaft speed from `71-engines` (VFSG on IP gearbox), airspeed from truth for the RAT.
- Fixed event order, fixed sum order (determinism, rule 3 of shared-architecture).

## Failures (0–1 on parts, never outcomes)
| Part | Failure | Observable |
|---|---|---|
| VFSG/ASG | bearing wear, rotor imbalance, low oil, winding short, shorted rotating diode, open phase, PMG loss, disconnect jam | GEN OFF/DRIVE, bus loss |
| GCU/BPCU/ELCU | internal fault, failsafe, 248-day counter, TTP/C loss, pin error | GEN OFF, bus fault, status |
| Contactors/relays | stuck open/closed, slow, welded, coil open, aux mismatch | fail-to-close, BTB/BSB, TRU status |
| ATRU/ATU/TRU/CMSC/SPU | degraded output, thermal, GFI, cooling loss, overload, saturation | HVDC SYS, TRU, APU START PWR UNIT |
| PECS | pump loss, coolant temperature rise | thermal shed |
| Batteries | cell short, imbalance, BMU/charger failure, cold lockout, thermal runaway, burst disc | MAIN BATTERY, vent odor |
| SSPC/ECB | spurious trip, fail to trip, ESW failed short/open, arc fault, leakage | CBIC, load loss |
| RPDU/SPDU | micro channel, gateway, backplane, power module | RPDU CHANNEL, cluster default |
| TCB | trip, nuisance trip, stuck | indication |
| Wiring | chafe, arc, open, short to ground, connector high resistance | trips, heat |
| Sensors | CT offset, voltage sense, temperature | wrong protection, false messages |
| External power | quality fault, pin interlock, chatter | AVAIL, EP trips |
| RAT | heater fail, uplock, gen failure | RAT GEN HEAT |
| Oil air valve | locked open, over-cool | VFSG oil |

## Temperature, wear, airframe tolerances
- Cold: battery resistance rise and charger lockout; oil viscosity into VFSG oil; breaker thermal trips slower; RAT heater below 2 °C (E:3961).
- Heat: converters and CMSC derate above coolant limit 96 °C (ATRU) or PECS thermal shed 64 °C; GCU forced-air cooling; solenoid 60 s rest.
- Wear: battery fade, generator bearings, contactor cycles, drive-disc solenoid heating.
- Airframe tolerances: battery capacity (50/65 Ah range), regulator setpoint (230 ±4 V), CT offsets, converter efficiency, GCU pin config, SSPC thresholds (leakage), wire resistance.
- Zones: P100/P200/P700/P800/P500/P600 bays, P300/P400 fwd EE bay, crown RPDU zones.

## Code layout
```
crates/solvers/electrical/        MNA solver, islands, phasors, converter averaged models
crates/systems/24-electrical/src/
  sources/     vfsg, asg, rat, external power
  storage/     main battery, APU battery, BMU, charger, BDM
  protection/  GCU, BPCU protections, ELCU, TRU OC tiers, SSPC I2t, arc
  pdu/         RPDU, SPDU, SSPC, TTP/C cluster comms
  control/     BPCU modes, load management, standby, start, CBIC/PDHA apps
  crew/        P5 switches, ELEC synoptic, EICAS messages, ESIC
aircraft/24-electrical/
  buses.*  contactors.*  panels.*  controllers.*  rpdus.*  sspc.*  tcb.*
  converters.*  cmsc.*  batteries.*  wires.*  loads.*  messages.*
```
- Data files generated from `refs/facts` (B, L, R tables), each row with grade and citation.
- CCS-hosted code (load management, PDHA, CBIC) implements the `lru` hosted-app interface; `42-ccs` places it in a GPM partition.
- Tests live in `tests/`.

## Tests
1. Cold and dark, BATTERY ON: which buses live, 30 s/2 min off delay, 20 s shutdown, MBR.
2. APU start on battery at +15 °C vs −30 °C: SPU, ATRU ±135 V, R2 CMSC; 3 attempts, 52 s, cooldown.
3. Engine start: both ASG, one VFSG, 2 × 90 kVA EP, with load shed; start time 55–60 vs 75–80 s.
4. Frequency-linked trips: UF/OF thresholds, underspeed 70 ms, ASG ready-to-load.
5. Every GCU protection: injected fault → time to trip matches the table; lockout and reset.
6. BTB ring reconfiguration after a VFSG loss; single source island solve.
7. RAT: all-engine loss → 15 s → standby, deploy ≤ 200 ms, 10 s to power, 380 Hz RCB close, instrument buses only.
8. Battery-only transfers: 30 ms interrupt, CIT 50 ms paralleling, MBR 15 s open delay.
9. External power quality: OV 130 V, UV 104.5 V, OF/UF, sequence, 150 ms ready.
10. Load shed: EP 90/112.5 kVA, 5 s steps, HPx twice in 20 min, thermal 64 °C.
11. ATRU/TRU/CMSC overload curves against the ratings.
12. Battery: charger limits, discharge inhibit, thermal runaway spreading, burst disc.
13. SSPC: I²t trips, arc detection, thermal breaker misses the intermittent arc.
14. RPDU gateway loss: cluster defaults after 3 s; essential loads on the other RPDU.
15. DRIVE disconnect: 5 s / 4 s pulses, 60 s rest.
16. Drive disc, GCU failsafe, 248-day counter.
17. Generated per-breaker test: pull each CE/CK/CT; its load loses power and the expected function degrades.
18. Determinism: same result on 1 and N threads, bit exact.
19. EICAS messages: every one with threshold and reset times.
20. GCU/BPCU power-up and fail-safe with computer hold-up times.
21. Source priority: for every bus and each loss combination, the source chosen follows the rank table, ground and air.
22. Ground power-up: operational test pulses (250 ms) before any EPC closes; transfers deferred; CCS 3 min before buses powered in ground handling.
23. Ground modes: handling, service, EP on, towing, maintenance; load counts (~530, +160).
24. Instrument bus supply chain: C1/C2 TRU, tie, MBR; MES on two fwd EP uses the battery for the F/O bus.
25. Fuel shutoff module: three valve cycles on 24 V after loss of both instrument buses.

## Documents and data
- Used: ata24, eps24, stm, lot_fcom, qrh, A28, A29, A38, 24eps.pdf figures.
- **Cites to re-check (verify):** the elec2 quote check could not find the quote at these lines (line drift or wrapped text); the numbers match the table row but treat as unconfirmed: A:1283 (EP 90 kVA levels), A28:2388, E:13697 (GEC3 opens ≤ 1020 Hz), E:1371 (APU selector), E:17643 (DC content reset 32 A), E:17716 (ASG OC 340 ±14 A), E:18130 and E:18223 (OF reset values), E:2529, E:33180 (AC12 phases), E:34134 (fusible link), E:34353 (shed restore), E:5513 (ASG instrument-bus fallback), F:79679, F:79694, F:79814 (switch descriptions).
- Still needed: the SSPC/CB configuration list, trip curves (ELCU types 1–6), wire data (gauge, length), battery cell data, GCU voltage regulator dynamics.

## Conflicts (open)
| # | Topic | Statements | Decision / status |
|---|---|---|---|
| 1 | Gateway RPDUs | figures: 71, 72, 81, 82; backup table lists 74 as gateway, 72 standard (E:33402–33404) | use figures: 71, 72, 81, 82 (R1) |
| 2 | SPDU count | 2 (7 statements) vs "4" (E:476, A:513) | 2 |
| 3 | Main 235 buses | 4 (VFSG) vs 6 (with LTB, RTB) | 4 + 2 tie = 6 |
| 4 | Breaker prefix | ECB/SSR CE, ELCC CK, TCB CT | CE/CK/CT |
| 5 | ±135 VDC on R2 bus | real, APU battery start only | modelled |
| 6 | Fuel shutoff module 24 V | deliberate, not a typo | modelled |
| 7 | 230 vs 235 VAC | same bus; real 230 V on O/J pump and RAT | as written |
| 8 | Main battery capacity | 50 Ah (E, A) vs 65 Ah (S) | **open**; use 50 Ah, tolerance to 65 |
| 9 | Battery switch off delay | 30 s (A) vs up to 2 min (F) | **open** |
| 10 | Tie-bus differential | 30 A (text) vs 40 A (table) | **open**; table value |
| 11 | ASG OC | 340 ±14 A table vs no text value | table |
| 12 | VFSG OF2/OF numbers | 2397/2391 vs ASG 2065/2052 | per source (different machines) |
| 13 | Contactor IDs | text-column misalignment made L1 GCB/GNR, L2 BTB/L ATUC, RCB/R DCT, RFCL/RA GNR share lines | settled by figures (R1b): L1 GNR CK2415001, L1 GCB CK2422525, L2 BTB CK2421502, L ATUC CK2425501, RCB CK2433503, R DCT CK2432808, RFCL CK2415006 |
| 13a | RA GNR ID | CK2412006 (photo, 24-12 series) vs CK2415006 (single-line, table) | **open**; CK2412006 probable (CK2415006 is RFCL) |
| 13b | L2 ATRUC soft-start ID | table CK2435502 duplicates ATRUC; diagram CK2435506 | diagram |
| 13c | R1 liquid cooling pump ID | CK2181505 (tables) vs CK2181504 (diagram, duplicates L1) | tables |
| 13d | AC feed RPDU 33 card | MAR02 PL3.1 vs APR25 PL3.3 / P300M3 | APR25 |
| 13e | R1 ELCU card legend | P100M1/M2 printed on P200 page for PR2.1/2.2 | P200M1/M2 (typo) |
| 13f | Main charger TCBs | CT2431801 input, CT2431802 output, 75 A, in P300 (the P400 label on the same text line is the FIT box) | P300 |
| 13g | CE2127811 | STM placeholder ID on every example row | real name SPLY FAN-F/O EQPT CLG MC |
| 13h | RTB source ranks | not fully read | **open** (assume mirror of LTB) |
| 14 | 30s RPDUs | text says two crown, figure all four crown | use figure |
| 17 | CMSC rating and mass | 110 kW, 53 kg (eps24) vs 105 kW, 115 lb (ata24, 787-8/-9), 115 kW (787-10) | **open**; ata24 for 787-9 = 105 kW |
| 18 | Aft EP rating | 90 kVA receptacle, system-limited 65 kVA (S:6823) vs 188 A rated (E:4900) | **open**; 65 kVA limit |
| 16 | APU battery start attempts | 2 then 5 min cooling (A:3071) vs 3 × 45 s (E:21589) vs SPU 3 starts (A:2973) | **open**; 3 × 45 s per E |
| 15 | Fwd EP AVAIL | "up to 90 kVA" vs ACTIVE/DEFAULT levels 90–180 kVA | levels are the EP management setting |
| 16 | ASG voltage and speed | text: 235/400 V at the POR, four-pole (E:12539, E:12534); figure callout: 230/400 V, 11,600–13,200 rpm, 360–440 Hz (E:12564–12566) | regulate 235 V at the POR; model the ASG as four-pole (2 pole pairs) at 12,000 rpm nominal |
| 17 | EP overload shedding | "If 5 minute EP source overload conditions exist (>90kVA) ... sequential load shedding at 5-second intervals"; "If 5-second EP source overload conditions exist (>112.5kVA) ... large groups" (E:35409–35418) | read "5 minute / 5-second" as the cart rating class: shedding starts as soon as the load exceeds the class limit (interpretation) |
| 18 | Generator overload | "If an overload exists ... while ASGs and/or VFSGs are supplying the buses, the BPCU LM function reduces CAC speed" (E:35420–35423); VFSG 250 kVA cont., 312.5 kVA 5 min, 437.5 kVA 5 s | CAC speed reduction first (min speed APPROX 50 %); sequential shed only if still over 100 % for 5 min or over 125 % for 5 s (from the VFSG ratings); HPx group shedding added (row 19) |
| 19 | HPx limit | logic given (GCU current → BPCU HPx per engine, preset group, fixed re-enable, repeat within 20 min adds a group, E:34405–34421); no numeric limit | APPROX limit 220 kW at ground idle rising to 650 kW at 15,900 rpm; groups cabin → misc heaters → window heat; re-enable 60 s APPROX |
| 20 | VFSG gen-mode entry vs idle | gen mode at 370 Hz / 7,400 rpm ground idle (F:37); engine idle speed not given | APPROX ground idle 7,550 rpm so the VFSG enters gen mode; VFSG may not feed a bus while starting or below entry |
| 21 | APB close timing | APU ready-to-load = 95 % + 2 s (R2:79) | APB closes only after ready-to-load; removes an ASG self-feed during start; APU start scenarios now ready at ~62 s |
| 22 | MAIN BATTERY DISCH on ground battery-only | branch 3–30 A for 10 min (E:2581); model draws ~19 A | message shows after 10 min on ground battery-only; scenario 02.e20 corrected to match its own citation |
| 23 | AC bus caution threshold | < 130.5 Vrms for 500 ms (F:345) | was 130.0; set to 130.5 |
| 24 | PECS coolant | thermal shed above 64 °C, order lighting → IFE (E:34439, E:34456); supply 85 °F nominal, ATRU min flow 12 lb/min (A:2458) | coolant node per loop: ATRU losses in, carried out at documented flow × coolant cp (STANDARD 3.6 kJ/kg·K) to the 85 °F supply; APPROX 60 kJ/K, 2 W/K still loss |
| 25 | Panel load placement | loads.csv gives voltage and panel for ~980 non-RPDU loads, no channel | placed on the panel bus by voltage, side by panel (P100/P300 left, P200/P400 right) else by parity; draws APPROX by bus class; ~286 rows without voltage or panel not placed |
| 27 | Breaker circuits | 284 breakers (eps24 SPDU/TCB/ELCC tables) | 282 placed on their table bus; ECBs with card+channel are SPDU SSPCs, other ECBs and TCBs thermal breakers, ELCFs ELCCs, all on measured current; draw APPROX 10 % of rating (DC), 2 % (AC), 1 % (instrument), standby on HBB |
| 26 | Load circuit wiring | no wire list | each circuit is a fault part (W:, CONN:); open drops the load, strand damage adds I²R (APPROX 0.05 Ω base), chafe is a real shunt and trips the circuit at APPROX 15 A |

| 28 | Constant-power loads at the power-transfer limit | fixed-point stalls near the nose (stress seed 2086) | damped Newton fallback on DC and AC islands (AC as real 2×2 blocks); second start on the low-voltage branch when no upper solution exists |
| 29 | VFSG drive disconnect | condition 50 ms; switch ≤ 3 s, wait 60 s (solenoid heat); auto on UF trip + LOP > 500 ms; pellet 390 °F (E:13448, A:4051, E:1232, E:12512) | solenoid heat budget 3 s, recovers over 60 s, damaged past it; crew press held APPROX 1 s; thermal pellet on VFSG oil temp |
| 30 | VFSG modes and duty | Stop/Start/Motor/Generator; 3 min full torque then 30 min at 50 %; motoring 150 ft-lb (E:9742, E:9782, E:9745) | mode per VFSG; cooldown caps start torque at 150 ft-lb; ENG_MOTOR action spins the engine dry |
| 31 | Start on external power | ≥ 2 forward 90 kVA sources (lot_fcom) | start cancelled and flagged when external power is the only source and < 2 forward EPCs are closed |
| 32 | Generator overload | 125 % 5 min, 175 % 5 s | shed after 5 min above 100 %, after 5 s above 125 %, at once above 175 % |
| 33 | APU capability vs altitude | 450 kVA SL, 100 kVA at 43,100 ft | linear between; ASG share of BPCU source kVA |
| 34 | Fwd EP commanded off | < 20 Vrms and < 10 A for 50 ms (E:18955) | ext switch drops; reselect needed |
| 35 | Ground handling | EP/ASG only, below 440 Hz (dist:191) | GH needs fwd cart ≤ 440 Hz |
| 36 | Battery switch OFF | 30 s battery-only hold; up to 2 min total (A:804, lot_fcom) | 120 s MBR watchdog after last MBR command |
| 37 | Standby paralleling | ≤ 50 ms (E:22208) | 50 ms latency |
| 38 | Charger | full charge 1–90 min (A:887) | CC/CV longer than 90 min latches charger fault |
| 39 | Managed load draws | no OEM per-load data | public cruise budget (PNNL-21382 fig 2.2, max icing): ECS 320 kW, hydraulics 40, ECS fans 32, galleys 120 (±270 / 230 V); panel loads still APPROX |

## Gaps
- Which CMSC drives which pump or NGS motor (verify).
- GATU contactor ID; RTB source ranks; ELCU channel map for every ELCC (from B/L).
- ATRU/TRU efficiency, GCU regulator transfer function.
- Wire gauge/length per feeder; connector contact resistance.
- SSPC and TCB trip curve parameters by rating (ELCU curve types 1–6 numeric).
- Battery cell chemistry and charger temperature limits.
- 248-day GCU counter source (A380 port, not in these documents).
- Computer hold-up times for each LRU.
- Public budget not yet applied: wing ice 60 kW, fwd cargo AC 60 kW, equipment cooling 40 kW, 115 V various 140 kW, 28 V 34 kW (panel loads are always on; ground EP would overload).
- Unbound facts (24): VFSG PF, weight, shear section; ASG freq window; receptacle pins; ATRU PU amps and APU-start ±135 V; RAT deploy max; load counts; diff trip / SUSP totals; battery enclosure, APU holdup, BCU, transfer interrupt, CCS max.
- Facts for other systems in tests/facts-pending (fire, hydraulics, lights, APU, starting).
