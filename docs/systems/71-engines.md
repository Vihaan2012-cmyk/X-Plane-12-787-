# 71–80 — Engines: Rolls-Royce Trent 1000-J3

Status: draft. Every number is from a verified fact table (quotes checked by script). Citation keys:
- `T` = rrtcds.txt (EASA TCDS E.036 Issue 24)
- `S` = stm.txt
- `F` = lot_fcom.txt (LOT FCOM Rev 12)
- `Q` = qrh.txt
- `A24`, `A26`, `A28`, `A29`, `A30` = the ATA books
- `E` = eps24.txt (Hamilton Sundstrand EPGSS course)
- `ICAO` = emissions databank

`APPROX` marks anything engineered because no source states it.

## Identity
- **Engine:** 787-9 / Trent 1000-J3 (F:28306).
- **Type:** three-spool, high-bypass turbofan (S:15980).
- **Thrust:**
  - takeoff 347.5 kN net, 350.9 kN bare (T:1065, T:1076);
  - MCT 319.5 kN (T:1087);
  - LOT's rating is 78,100 lb Boeing Equivalent Thrust (F:82316).
- **ICAO J3:** BPR 8.9, OPR 47.8, rated 350.9 kN. Fuel flow at T/O, C/O, App and Idle: 2.649 / 2.168 / 0.699 / 0.256 kg/s.
- **Physical:** fan 2.85 m with 20 blades (T:718, S:16162); dry mass 6114 kg (T:867).

## Hardware
- **LP spool (N1):** fan plus a 6-stage LPT. The fan gives about 80% of takeoff thrust (S:16161, S:16165, S:16168).
- **IP spool (N2):** 8-stage IPC plus 1-stage IPT. **It drives the accessory gearbox** (S:16183, S:16171, S:15065).
- **HP spool (N3):** 6-stage HPC plus 1-stage HPT (S:16161, S:16168).
- **Annular combustor** (S:16167).
- **Gearbox (IP-driven):** 2 VFSGs and 1 EDP (A24:4139, A29:1683). The fuel pump, oil pump and PMA mounting is **(verify)**.
- **Variable geometry and bleeds:**
  - 1 VIGV stage plus 2 VSV stages, linked, with 2 actuators (S:16717, S:16711, S:16701).
  - 3 IP8 bleed valves and 2 HP3 bleed valves, worked by solenoids using HP6 air (S:16723, S:16720, S:7160).
- **Secondary air system:**
  - SAS from IP8/HP3/HP6, with the SASV (S:14764, F:82629).
  - CAC turbine cooling from HP3 (S:16699).
  - TCC with 2 fan-air valves (S:16705).
  - ESS core anti-ice, IP8 air to the IPC inlet (S:15608).
- **Nacelle anti-ice:** PRSOV + PRV + controller air cooler + valve controller, controlled by CCS → MEDC (A30:338, A30:680).
- **Electronics:**
  - Dual-channel EEC (FADEC), on the engine's left side (S:15939, S:16329).
  - EMU for vibration and health monitoring (T:1250, S:2490).
  - MEDC (A26:206).
  - Data entry plug (S:16326).
- **Sensors:**
  - N1C, N1T, N2C, N3 speeds; TGT (2 systems × 6 thermocouples); turbine overheat; T30, T25, P20/T20, P0, PS26, P30.
  - Oil pressure and temperature, oil debris (ODS), fuel flow, fuel temperature, LP filter ΔP.
  - 3 accelerometers feeding the EMU.
  - (S:16319, S:16320, S:16310, S:1782, S:16821.)

## Gas path physics
- **Three spools solved together, every step:**
  - fan, IPC and HPC maps;
  - combustor energy balance;
  - HPT, IPT and LPT flow capacity;
  - bypass and core nozzles.
- **Spool dynamics:** inertia, plus power offtake on the **IP spool** (two VFSGs up to 250 kVA each, plus the EDP).
- **Calibration targets:**
  - ICAO 4-point fuel flow;
  - TCDS thrust;
  - LOT FCOM PD/PI cruise and engine-out tables (TPR, fuel flow);
  - TCDS speed and TGT limits;
  - start timelines from video.
- **Maps** are public generic shapes scaled to those anchors (`APPROX`).
- **Variable geometry and bleeds** follow EEC schedules (`APPROX` shapes). Air-flow control keeps compressor stability during start, transients and reverse (S:14796).
- **Anti-ice bleed limits** (T:1389): at most 2.9% of W26 up to TET 1015 K, 2.9→2.1% from 1015 to 1765 K, 2.1→0.5% from 1765 to 1810 K, and 0.5% above that.
- **TGT:**
  - measured at the LPT 1st-stage NGV (T:2336);
  - displayed TGT is **trimmed** by the EEC using the DEP trim profile (T:2398);
  - the J3 has three trim profiles (T:2459–2469), and which one an airframe has is a tolerance (`APPROX` choice).
- **Surge and stall:** stall margin emerges from the maps. A surge triggers ENG SURGE (F:83465).

## Engine fuel system
- **Flow path:**
  1. airframe feed → spar valve;
  2. LP centrifugal pump → FOHE → LP filter;
  3. 2 HP gear pumps (primary + secondary for acceleration and high power) → HMU, where the FMV is controlled through the PDCV;
  4. PRSOV → fuel flow transmitter → HP filter → nozzles.
  5. Bypass fuel returns to the pump.
  - (S:16479, S:16487, S:16167, S:16458, S:16480, S:15296, S:14847, S:16476.)
- **FOHE** heats the fuel to stop ice (S:16587). FOHE icing is real (AD 2022-14-04).
- **Spar valve:**
  - Normally opened and closed through relays in P300/P400, from the fuel control switch.
  - The fire switch sends 28 VDC straight to the valve to close it (S:2124, S:16283).
  - Opens when the fire switch is IN and either START or RUN is selected. Closes when the fire switch is OUT, or when CUTOFF and NORM are both selected (F:82187, F:82803).
- **Engine fuel valve:** opens when the fire switch is IN and RUN is selected (F:81880).
- **TCDS fuel limits:**
  - temperature: −54 °C minimum before start, −45 °C minimum for acceleration, 65 °C maximum;
  - inlet pressure: at least 5 psi above vapour pressure (2 psi for transients), or 3.4 psi with the boost pumps off up to 35,000 ft for 600 min;
  - maximum 70 psi steady, 140 psi transient (T:1529, T:158, T:1703–1711, T:1725–1728).

## Oil system
- **Unregulated:** pressure follows engine speed (S:16589).
- **Path:** tank → pump → 2 surface coolers (EOSC) plus the FOHE (the main cooler) → bearings → 10 scavenge elements → chip detectors and ODS → tank (S:16606, S:430, S:16589).
- **Cooler bypass:** the EEC opens the cooler bypass valve (S:16586).
- **Filter:** a clogged scavenge filter gives ENG OIL FILTER and the oil bypasses the filter (F:82428).
- **Viscosity depends on temperature.** That sets oil pressure, pump torque and starter load, so in the cold, oil pressure is slow to rise and spikes high first (F:15898–15900). It emerges.
- **TCDS oil limits:**
  - at least −40 °C to start;
  - at least 40 °C to accelerate, rising linearly to 90 °C as fuel temperature falls from −23 °C to −45 °C;
  - 196 °C maximum, 201 °C for up to 15 min;
  - minimum pressure 30 psi at ground idle, rising linearly to 90 psi at 100% (T:1568, T:1581, T:1593, T:1789–1813).
- **Quantity:** there's no oil quantity limit, and LO auto-displays the secondary engine page (F:83022).

## EEC (FADEC) and EMU
- **Channels and power:**
  - Two channels, one in control at a time, with automatic changeover (S:16329, S:16330).
  - Powered by the PMA, with aircraft 115 VAC through an RPDU from START (S:16312, S:7678, A24:4470).
  - The PMA also provides a backup N2 signal (S:2968).
- **Inputs:**
  - TLA resolvers (2 per lever);
  - fuel switch state from the FCM;
  - air data from the FCE over the CDN;
  - engine sensors.
  - (S:16265, S:16264, S:5332.)
- **Outputs:**
  - HMU metering;
  - VIGV/VSV, bleeds, SASV and TCC;
  - oil cooler bypass;
  - igniters;
  - the reverser isolation valve and interlock;
  - start torque commands to the CMSC;
  - data to DCAS and CMCF over the CDN (S:16494, S:16702, S:15945).
- **Thrust parameter:**
  - TPR, computed by the EEC from T20/P20, TGT, P30 and more (S:16808, S:3330).
  - The EEC continuously computes maximum TPR, and full forward gives maximum rated thrust (F:82496).
- **Modes (S:16264, S:16345, F:82477–82576):**
  - **Normal:** controls on TPR.
  - **Soft alternate:** automatic when air data or other signals are lost. Controls on N1, and the switch stays in NORM.
  - **Hard alternate:** manual ALTN, or automatic after **10 min in soft alternate**. Thrust can step to the commanded N1.
  - In alternate there's no thrust protection: full forward overboosts (ENG LIMIT PROT). N1 and N2 redline protection still apply.
- **Protections:**
  - **RPM limiting** on N1 and N2, with N3 protected indirectly. If limiting fails, fuel is shut off (F:82587–82616, ENG RPM LIMITED).
  - **SASV failed open:** above 400 ft AGL thrust is cut to a safe limit (ENG THRUST HIGH, ENG SEC AIR VLV). Below 80 kt on the ground, a SASV failed closed gives increased idle (F:82636–82665).
  - **Ground idle asymmetry:** an engine that won't decelerate with the lever at idle is shut down (F:82615).
- **Idles (F:82730–82766):**
  - **Minimum idle:** on the ground and most of the flight.
  - **Approach idle:** flaps ≥25 or gear down, held until touchdown.
  - **Icing idle:** EAI on.
  - **Ice-crystal idle:** set by an ISA band and altitude **(verify OCR)**.
- **EMU:**
  - Vibration from the 3 accelerometers plus rotor speeds, by source N1, N2, N3 or broadband.
  - At 4 units the display goes reverse video and the secondary page auto-displays.
  - There's no certified vibration limit (S:16821, F:82119, F:83150).

## Start (autostart only)
- **No manual start, no pneumatic start.** The VFSGs act as starters on the IP spool (A24:4454, E:4208, F:82762).
- **Power sources:**
  - Normal: APU (both ASGs).
  - Alternatives: external power (at least 2 forward 90 kVA with load shed; 3 recommended without the APU), the opposite engine, or the RAT in flight.
  - One external source can't start an engine, and one forward plus one aft can't either (A24:4419–4428, A24:1102, A24:1280).
  - A simultaneous start needs the APU and both ASGs; otherwise the second command is ignored (F:82782).
  - Don't start both engines together with OAT above 40 °C (F:7225).
  - Load shed always happens during a start (F:74990).
  - **All of this emerges from the electrical model's power budget.**
- **Sequence (A24:4469–4487, F:82808–82826):**
  1. START → BPCU puts the CMSC into MES mode → the CMSC sets contactors in P700/P800 → the start contactors close (each needs a GCU ground plus CMSC 28 VDC) → the GCU opens the neutral relay.
  2. The CMSC sends OK TO START, then START INITIATED, over the CDN.
  3. The EEC sends a torque command, the VFSGs dry-motor the engine, and the spar valve opens.
  4. FUEL CONTROL to RUN → the EEC opens the fuel valve and fires the igniters at the right N3 speed **(conflict below)**.
  5. The EEC monitors EGT, N2 and N3.
  6. Starter cutout at about **46.7% N2 (RR)** → RUN STOP → START COMPLETE → the selector returns to NORM.
  7. RUNNING shows for 30 s after reaching idle (F:81882, F:81974).
- **VFSG starter rating:**
  - 300 ft-lb in start mode, 150 ft-lb for maintenance motoring (E:9756, E:9745).
  - 100% torque for 3 min, then 50% for up to 30 min (A24:4497).
  - The VFSG enters generator mode at ground idle: 370 Hz, 7400 rpm (E:9761).
- **Ground start aborts and retries (F:82827–82867):**
  - **Abort with no retry:** both starters lost, or no N1 rotation. Gives ENG AUTOSTART.
  - **Correctable:** compressor stall, hot start, no EGT rise, starter duty timer exceeded, or one starter lost. The EEC cuts fuel and ignition, motors for 20–30 s, then retries on a reduced fuel schedule with both igniters.
  - Maximum 2 attempts on the ground. If both fail, the EEC motors the engine to clear fuel.
  - Fuel and ignition come back on later attempts when EGT is below 200 °C.
- **In flight:**
  - Unlimited attempts until success or CUTOFF.
  - It cuts fuel at the takeoff EGT redline, or on no light-off or a hung start.
  - It windmills 30 s between attempts.
  - X-START shows when outside the windmill envelope, and the starter-assist envelope is displayed (F:82868–82897, F:81895).
  - Envelope numbers **(verify: FCOM figure)**.
- **TCDS start limits:** TGT 700 °C on the ground and 900 °C for relight, oil at least −40 °C, fuel at least −54 °C (T:1512).
- **Not on this engine:** bowed-rotor motoring. The FCOM doesn't mention it.

## Ignition
- **Two independent systems,** each an exciter plus an igniter, cooled by fan air (S:6911, S:16910, S:16911).
- **Power:** P300/P400 relays. The captain instrument bus feeds one exciter and the FO bus feeds the other. Standby power is the backup (S:1905, S:6815, F:82916).
- **Selection:** the EEC alternates igniters between ground starts and uses both in flight (F:82911).
- **Auto-relight:** on flameout, or at or below idle in RUN, both igniters fire until idle. Also used in heavy rain or hail. There's no continuous ignition (F:82899–82926).

## Thrust reverser
- **Type:** hydraulically actuated, fan stream only. Two halves, each with 2 actuators, a sync shaft, a track lock and proximity sensors. The HCU is in the pylon. Sixteen blocker doors and cascades (S:17033, S:16131, S:14960, S:17062).
- **Logic:**
  1. Reverse lever raise (forward levers at idle, on the ground) → TCM microswitch → RPDU via CDN → track-lock solenoid.
  2. TLA → EEC → HCU isolation valve.
  3. LVDTs report position to the EEC. The aisle-stand interlock holds the lever at reverse idle until the cowl is partly deployed.
  4. REV shows amber in transit and green when deployed.
  - (S:16264, S:17172, S:17162, F:83065, F:83072.)
- The EEC also limits thrust in reverse, and the reverse levers deploy the automatic speedbrakes (F:83057, F:82154).
- **Messages:** ENG REV AIR/GND, ENG REV COMMAND, ENG REV LIMITED, ENG REVERSER.
- **Engine-out reverse:** with ENG REV AIR/GND, the reversers will deploy in flight if commanded (Q:5192).

## Other engine-attached systems (owned by their own plans)
- **EDP:** 5000 psi, 27 gpm, up to 6800 rpm. The fire switch closes the ESSOV and depressurises it (A29:1683, A29:1694, A29:1638).
- **Fire:**
  - Detection: 2 channels → MEDC → RDC → PFPS.
  - Extinguishing: 2 × 800 in³ Halon 1301 bottles at 825–850 psig, either bottle to either engine, in the forward cargo right cheek (A26:206, A26:479, A26:460, A26:579).
- **EAI:**
  - The P5 switch has OFF, AUTO and ON. AUTO follows the ice detectors.
  - Don't use it above 10 °C.
  - With EAI on, idle rises to icing idle (A30:687, F:15785).
- **PDOS:** cowl opening, using 115 VAC from RPDU 75 (S:16121).

## Indications (DCAS reads the EEC over the CDN)
- **Primary, always on:**
  - TPR: actual, commanded, reference/target, and an amber maximum line;
  - N1;
  - EGT (TGT): white, amber at maximum continuous, red at takeoff or start limit.
  - (F:82373, F:82433, F:81858.)
- **EGT colouring:**
  - The amber band is inhibited for 5 min at takeoff or go-around, 10 min with one engine out (F:82455).
  - The red start-limit line shows until the engine is stable at idle (F:82805).
- **Secondary:** N2, N3, fuel flow (kg/h × 1000), oil pressure, oil temperature, oil quantity (litres), vibration (F:5160, F:82003).
  - **Auto-display:** at power-up, START, CUTOFF in flight, a fire switch pulled, an exceedance, N2 below idle, LO oil, high vibration, or the FUEL FLOW advisory (F:82374–82392).
- **Red-box latch:** after an exceedance the box stays red. Cancel/recall switches it between red and white (F:82431).
- **Status text:** RUNNING, AUTOSTART, AUTORELIGHT, idle target, X-START, in-flight start envelope (F:81882–81994).
- **Alternate mode:** the TPR display blanks and N1 targets show instead (F:82557).
- **Loss of engine data:** the indication blanks. There's no EICAS message for it (F:82368).
- **Limit numbers:** colours come from the FCOM, numbers from the TCDS. Amber band values **(verify)**.

## EICAS messages
ENG AUTOSTART, ENG CONTROL, ENG CORE ANTI-ICE, ENG EEC MODE, ENG FAIL, ENG FUEL FILTER, ENG FUEL VALVE, ENG LIMIT EXCEED, ENG LIMIT PROT, ENG OIL FILTER, ENG OIL PRESS, ENG OIL TEMP, ENG REV AIR/GND, ENG REV COMMAND, ENG REV LIMITED, ENG REVERSER, ENG RPM LIMITED, ENG SEC AIR VLV, ENG SHUTDOWN, ENG START CUTOUT, ENG STARTERS, ENG SURGE, ENG THRUST, ENG THRUST HIGH, ENG TURB DAMAGE, THRUST ASYM PROT, FIRE ENG, OVERHEAT ENG, BTL DISCH ENG, DET FIRE ENG.
- **ENG FAIL alert during takeoff:** voice plus master WARNING on the PFDs and HUDs (F:83168).

## TCDS limits (J3)
| Item | Limit |
|---|---|
| N1 / N2 / N3 TO | 101.5 / 103.6 / 101.5 % (N2 transient 103.9 % for 30 s, TO/GA only) |
| N1 / N2 / N3 MCT | 101.5 / 100.8 / 99.5 % |
| 100 % reference speeds | LP 2683, IP 8937, HP 13391 rpm |
| TGT | start 700 °C, relight 900, TO 900 (5 min), MCT 850, overtemp 920 (20 s) |
| TO rating | 5 min; 10 min after an engine failure |
| Ambient | up to ISA +40 °C |
| ETOPS | 330 min at MCT + 15 min hold |
| EEC software | DO-178B Level A; EMU Level E (vibration display Level C) |

## Cold, ice and heat
All of these come out of the models: oil viscosity, LCD temperature, ice accretion, the start power budget.
- **Oil pressure:** slow to rise at first, and the initial rise can be higher than normal (F:15898–15900).
- **Displays:** slow and dim when cold (F:15907).
- **Warm-up:** at least 5 min at taxi thrust, with oil above the lower amber band before takeoff (F:7313, F:7312).
- **Fan ice shedding:**
  - Ground run-ups to 20 TPR (42 % N1) for 30 s at the intervals the FCOM sets.
  - A static run-up to 50–55 % N1 before takeoff when OAT ≤ 3 °C with EAI on.
  - In flight with high vibration, an idle then max-climb N1 cycle (F:15794–16126).
- **Brake release:** don't release above 55 % N1. Crosswind capability drops above 20 TPR (F:10022, F:10026).
- **Cold parking:** if parked unpowered below −40 °C, the fire bottle compartment must be warmed to −23 °C (F:4878). That's a fire-plan link.
- **FOHE fuel icing** and ice-crystal effects come from the environment plan.

## Failures (0–1, per component)
- **Rotating parts:** blades (fan, IPC, HPC, HPT, IPT, LPT: erosion, FOD, crack, release), bearings, shafts (→ ENG TURB DAMAGE).
- **Combustion and air:** combustor or nozzle coking, VIGV/VSV actuators, each bleed valve, SASV (open or closed), TCC.
- **Fuel:** LP and HP pumps (each HP pump), HMU and FMV, PDCV, PRSOV, FOHE (leak or ice), filters (contamination → bypass).
- **Oil:** pump, coolers, cooler bypass valve, chip detector or ODS (debris generation), leaks.
- **Ignition:** igniters and exciters (each).
- **EEC:** each channel, DEP, PMA, each sensor (bias, fail).
- **EMU and accelerometers.**
- **Reverser:** actuators, track lock, HCU, LVDTs, interlock.
- **Starting system:** start contactors, CMSC.
- **Environment:** volcanic ash, bird strike, ice crystals, heavy rain or hail.

## Temperature, wear and airframe differences
- **Deterioration with cycles and hours:** blade erosion and seal wear → efficiency falls → TGT margin shrinks, with a realistic margin curve (`APPROX`).
- **Per-engine tolerance:** trim profile, map offsets, sensor biases, oil consumption (`APPROX`; not in the TCDS).
- **Engine changes:** a swapped-in engine brings its own history and rating plug (F:82344).

## Tests
1. A normal APU start: the timeline (motoring, light-off, 46.7 % cutout, idle, RUNNING) matches video within tolerance.
2. A start on 2 forward external power units with load shed works. A start on 1 external unit is refused.
3. A hot start triggers the correctable sequence: 20–30 s of motoring, then a reduced-fuel second attempt. A second failure brings ENG AUTOSTART.
4. A −30 °C soak: slow oil pressure with a high initial spike, a longer start, and warm-up holding the oil below the amber band.
5. Losing CDN air data gives soft alternate, then hard alternate after 10 min. TPR blanks and N1 takes over.
6. Alternate mode at full forward gives ENG LIMIT PROT and an overboost.
7. SASV failed open above 400 ft cuts thrust to the safe limit and gives ENG THRUST HIGH.
8. A dual flameout at high altitude: slow relight that looks like a hung start, and X-START below the windmill envelope.
9. Ice-crystal conditions with FOHE ice give fuel restriction behaviour **(verify against AD 2022-14-04)**.
10. Calibration: fuel flow at the ICAO points and the LOT FCOM cruise tables within tolerance. Thrust matches the TCDS.
11. Reverser: in-flight selection is blocked. With ENG REV AIR/GND, it deploys if commanded.
12. Vibration at 4 units gives reverse video and the secondary page auto-displays.

## Conflicts (unresolved)
- **Ignition timing:**
  - S:17011 says about 3.5 s after fuel on for a first start;
  - A24:4511 says igniter on at N2 > 6 % (RR);
  - F says "at the appropriate N3 RPM".
- **Cutout:** about 46 % (S:17013) against 46.7 % N2 (A24:4512). We use 46.7 %, the RR-specific figure.
- **BPR:** about 10:1 (S:15982) against 8.9 (ICAO, J3-specific). We use 8.9.
- **EDP location:** two descriptions in ATA 29 (A29:1604, A29:1615).

## Gaps (engineered, marked `APPROX`)
- Compressor and turbine maps.
- EEC schedules: acceleration and deceleration, VIGV/VSV, bleeds, the TPR loop.
- Oil system thresholds and oil consumption.
- Exact amber-band values.
- Starter duty timer values.
- In-flight start envelope numbers.
- Power-offtake limits (they're in the Installation Manual).
- Reverser timings.
- The FCM/EEC CDN path details.
- Ice-crystal idle band (OCR).
