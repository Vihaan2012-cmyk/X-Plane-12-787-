# Data flows (first ICD input)

Every documented data movement between boxes. This seeds [shared-data.md](shared-data.md). The Check column has the same meaning as in [computers.md](computers.md).

## Avionics and computing

| Data item | From box | To box(es) | Path (CDN/ACS/ARS/RDC, ARINC 429, CAN, discrete, analog) | Units/range/rate if stated | Source | Check |
|---|---|---|---|---|---|---|
| Airplane system I/O (generic) | Airplane systems | ACS, FOX, RDC | Systems connect via ACS, FOX or RDC into CDN | CDN: 100 Mbps fiber, 10 Mbps copper outside CCRs | stm:1840-1868 (p.5-4) | OK |
| Hosted application data (GPM to GPM) | GPM | GPM | CDN via ACS (ARINC 664, switched Ethernet) | 100 Mbps (in CCR copper and fiber) | stm:1753-1755; stm:2026-2027 | OK |
| Non-664 system data | Airplane systems (CAN, ARINC 429 hi/lo, analog, analog discrete) | RDC, then CDN | RDC converts to ARINC 664 | ARINC 429 high and low speed; rates ? | stm:1962-1980 | NEAR |
| CDN network transport | ARS, FOX, ACS | ARS, FOX, RDC, ACS | ARS fiber 4 ports; ARS copper 20 ports direct inputs | ARS fiber 100 Mbps; copper 10 Mbps | stm:2143-2154; stm:2035-2039 | OK |
| Graphics commands (ARINC 661 over ARINC 664) | GPM (DCAS software) | GG | ACS (ARINC 664 switch) | ARINC 661 content; rate ? | ata31:209; ata31:2917-2920 | OK |
| Display pixel data | GG | HDD (5), HUD projector (2) | Fiber optic pixel buses (6 per GG; 1 to each HDD, 1 to onside HUD) | Rate ? | stm:2678-2702 | NEAR |
| HDD and HUD status | HDD, HUD projector | GG | Not stated (status returned to GGs; fiber path not given) | ? | stm:2687-2689 | NEAR |
| Hardware MFK/CCD data | MFK, CCD | RDCs (2 per panel), then GGs via CDN | ARINC 429 bus | ARINC 429 rate ? | ata31:723-724; ata31:767-768 | OK |
| EFIS/DSP control data | EFIS/DSP panels | RDC (then CDN) | ARINC 429 data buses | ? | ata31:705-706 | OK |
| Tuning control data | TCP | RDC, CDN, VHF/HF transceivers | RDC and CDN; direct to left VHF on CDN failure (link ?) | ? | stm:8966-8985; stm:9128-9150 | OK |
| Digital audio (transmit) | ACP | AGU | CDN (digital) | ? | stm:8414-8421 | NEAR |
| Analog audio to/from radios | AGU | VHF/HF transceivers; antennas | Analog (AGU converts digital to analog) | ? | stm:8966-8985; stm:9128-9150 | OK |
| Backup audio (CDN failed) | Left ACP | Left AGU | Direct link (bypasses CDN) | ? | stm.txt:8547 | QUOTED |
| Ground crew call | TCP (CALL) | Ground call horn (nose wheel well) | ? | 3 seconds horn | stm:8720-8735 | OK |
| SELCAL alert | AGU (decodes airplane code) | DCAS (EICAS message); ACP (aural via comm/warning speakers) | ? (DCAS path not stated); aural via ACP | ? | stm:9434-9450 | NEAR |
| SATCOM audio and data | ACP, AGU | SRT, DLNA module, HGA | CDN to AGU; then RF path SRT -> DLNA -> HGA | ? | stm:9281-9300 | OK |
| HF RF path | HF transceiver | Coupler, antenna | RF (coupler in vertical stabilizer) | ? | stm:9128-9150 | OK |
| Air data pressure | Pitot and static ADMs | FCE (air data reference) | Digital (ADMs convert pressure to digital signal) | ? | stm.txt:10471 | QUOTED |
| AOA, TAT | AOA sensors, TAT probe | FCE (air data reference); SWF | ? (path not stated) | ? | stm:10330, 10341-10343; stm:5243-5244 | NEAR |
| Voted air data (altitude, computed airspeed, Mach, TAS, AOA, air temperature) | FCE (ADRF) | Systems (incl. DCAS) via CCR cabinets | CDN (to CCR cabinets) | ? | stm:10324-10330, 10332-10343 | NEAR |
| Standby heading and groundspeed | FCE (center 1) | ISFD | ? (not stated) | ? | stm:11208-11211 | NEAR |
| Center pitot and static (IAS, baro alt) | ADM (center) | ISFD | ? (not stated) | ? | stm.txt:11365 | QUOTED |
| ILS deviation | Left INR | ISFD | ? (not stated) | ? | stm:11233-11235 | NEAR |
| Backup airspeed and altitude | ISFD | DCAS | CDN (used only if all FCEs fail) | ? | stm:11213-11217 | NEAR |
| Inertial reference inputs | FCE (air data and flight control) | IRU, AHRU | ? | ? | stm:10455-10470 | OK |
| GPS data | INR | IRU, AHRU | ? (not stated) | ? | stm:10471-10473 | NEAR |
| Hybrid inertial/GPS data | IRU, AHRU | CDN, then DCAS (PFD, ND) | CDN | ? | stm:10475-10482 | NEAR |
| GPS position and time | INR (GPS) | FMF, ERS | ? | ? | stm:10857-10885 | OK |
| VOR, ILS, MB, GPS data | INR | DCAS, FMF | ? | ? | stm:10492-10500 | OK |
| Radio tuning (ADF, VOR, DME, ILS) | FMF | ADF, DME, VOR, ILS receivers | ? (via FMF, CCR; ADF/VOR/ILS per stm:11092-11098; DME not cited) | ? | stm:11092-11098 | NEAR |
| DME slant range | DME interrogators | FMF, DCAS (consumers UNSUPPORTED) | ? | ? | stm:11494-11495 | NEAR |
| DME suppression pulses | ATC function (ISSPU) | DME interrogators, TCAS | ? | ? | stm:11759-11761 | NEAR |
| Radio altitude | RA transceivers | DCAS, AFF, TMF, TAWS, TCAS | ? | 0 to 2,500 ft | stm:11358-11371 | NEAR |
| Altitude for ATC reporting | FCE | ATC function (ISSPU) | ? | ? | stm.txt:11916 | QUOTED |
| TCAS advisories (TA/RA) | TCAS (ISSPU) | DCAS, flight deck aurals; other TCAS aircraft via ATC link | CDN (ATC data link); ? | ? | stm:11929-11955 | OK |
| Weather radar returns | RTM | ISSPU | Fiber optic (RTM sends weather data to ISSPU) | ? | stm:12116-12118 | NEAR |
| Antenna stabilization | ERS | WXR | ? | ? | stm:12095-12118 | OK |
| TAWS inputs | CDN (air data, inertial, flight management, flap data, etc.) | TAWS (ISSPU) | CDN | RA 30 to 2,450 ft (modes) | stm:12277-12278, 12287-12292 | NEAR |
| TAWS and TCAS alerts | ISSPU | DCAS, communication/warning speakers | CDN | ? | stm:12271-12290; stm:11955 | OK |
| Lateral and vertical guidance | FMF | AFF (FCE) | ? | ? | stm:12744-12765 | OK |
| Thrust and speed commands | FMF, MCP | TMF | ? | ? | stm:12744-12765; stm:13118-13138 | OK |
| Autopilot commands | AFF (FCE) | PFCF -> PCUs, BDAs | ? | ? | stm:12954-12975 | OK |
| Thrust lever movement | TMF | Thrust levers (two servomotors), engines | Two independent servomotors | ? | stm:13132-13134; stm:13139 | NEAR |
| Flight data recording | CDN | Flight recorders (both) | Fiber optic from CDN; FDAF formats per ARINC 767 | 2 h audio; 25 h flight data | ata31:3437, 3478-3480, 3441-3445 | OK |
| Flight recorder to core network | Flight recorder | Core network (IDN) | Ethernet (IDN) | ? | ata31:3453; ata46:282-296 | OK |
| IDN to CDN and other isolated systems | Core network (IDN switch) | CDN, flight recorder, flight deck printer | Ethernet (IDN) | ? | ata46:282-300 | OK |
| EFB to core network | EFB EU, EFB DU | Core network (ODN/IDN) | Ethernet and wireless (path ?) | ? | ata46:295-300 | OK |
| Ground link | Ground network | TWLU (wireless), TCS | Wireless | ? | ata46:193-194 | OK |
| Maintenance laptop | ML | CWLU (wireless) or wired ML ports | Wireless and Ethernet | ? | ata46:256-258 | OK |
| DCAS maintenance pages | DCAS | Core network (MCD) | ARINC 661 web server / Ethernet | ? | ata31:194-196; ata31:2936-2943 | OK |
| Caution and warning | CAS (DCAS) | Master warning/caution lights, aural speakers | Discrete and aural outputs | ? | stm:5150-5165 (p.6-28) | OK |
| Nose gear pressure | Two NLG pressure transducers | CAS | ? | ? | stm:5172-5176 (p.6-28) | OK |
| CCR reset | Reset switches (2) | PCMs | Hard-wired discrete | ? | stm:2181-2191 | OK |
| CCS maintenance pages | CCS maintenance pages (GPM, CDN terminal, PCM status) | MFDs | ? (display path not stated) | ? | stm:2264-2269 | OK |
| Altitude alert | MCP altitude, air data | DCAS (PFD) | ? | 200 ft (caution) (stm:5301) | stm:5288-5292, 5301 | NEAR |
| Stall warning | AOA function, air data reference, flight controls, earth reference data, landing gear, ice and rain, engines | SWF -> stick shaker actuators | ? | Stick shaker at trip point | stm:5230-5254 | NEAR |

## Electrical, engines, APU, fuel

| Data item | From box | To box(es) | Path | Units / range / rate if stated | Source | Check |
|---|---|---|---|---|---|---|
| Generator commands and status (GCU-BPCU) | GCU | BPCU | Isolated data bus | ? | ata24:960 | OK |
| Gen control and protection commands | BPCU | GCU (6) | Data bus command or discrete input | ? | ata24:1423-1424 | OK |
| ATU contactor commands | GCU | ATUC | Data bus commands from BPCU, or discrete inputs | ? | ata24:1420-1425 | NEAR |
| Flight deck control and indication (BPCU) | BPCU | CDN (flight deck and airplane systems) | CDN | ? | ata24:961, 972, 1003 | OK |
| BPCU control of GCUs, SPDUs, ELCUs | BPCU | GCU, SPDU, ELCU | TTP/C (dedicated) | ? | ata24:1004; ata24:955 | OK |
| External power contactor control (L/R EPC) | BPCU L/R | Left/right EPC | ? (contactor control, path not stated) | ? | ata24:1072, 1095; stm:7392-7395 | OK |
| Aft external power contactor (aft EPC) | GCU L2 | Aft EPC | ? (L2 GCU opens and closes aft EPC; signal path not stated) | ? | ata24:1101, 1161 | OK |
| Gateway RPDU network interface | Gateway RPDUs (4) | CDN and TTP/C | CDN and TTP/C | ? | ata24:1977; stm:7683-7686 | OK |
| Standard RPDU to gateway | Standard RPDU (13) | Gateway RPDU | TTP/C bus | ? | stm:7683-7686; ata24:1978 | OK |
| SPDU gateway to CDN | SPDU | CDN via BPCU | CDN (BPCU as gateway) | ? | ata24:1637-1638; ata24:1854-1855 | NEAR |
| SPDU and RPDU 28 Vdc feed | SPDU (P300/P400) | RPDUs | Hardwired power | 28 Vdc | stm:7688-7690 | OK |
| 235 Vac to +/-270 Vdc conversion | ATRU | CMSC | Hardwired power | +/-270 Vdc | stm:7589-7591, 6814-6816 | OK |
| Large motor drive power | CMSC | Hydraulic EMPs, CACs, RAM fans, engine/APU start | Hardwired power | Variable frequency 235 Vac | stm:6982-6990, 7556-7561 | OK |
| 235 Vac to 115 Vac conversion | ATU (P500, P600) | 115 Vac L/R buses | Hardwired power | 115 Vac | ata24:1407-1415; stm:7399-7404 | OK |
| 235 Vac to 28 Vdc conversion | TRU | 28 Vdc L/R buses; Capt and F/O instrument buses | Hardwired power | 31 Vdc (stm:6801) vs 28 Vdc (stm:7748) | stm:6799-6807, 7747-7748 | OK |
| 28 Vdc to +/-130 Vdc | EBPSU | Electric brake system (via EBAC) | Hardwired power | +/-130 Vdc | ata24.txt:1572 | QUOTED |
| Main battery to instrument buses | Main battery | Capt and F/O instrument buses via MBR | Hardwired (MBR closed by BPCU) | 28 Vdc | ata24:782-795; stm:7500-7503 | OK |
| Battery charging | BCU | Main or APU battery | Hardwired (BDM limits to BCU path) | 46 A, 32.2 Vdc; 20% in 1.5 h | ata24:690-695; stm:7520-7523 | OK |
| RAT output to backup bus | RAT | Backup bus (via RAT control breaker) | Hardwired | 230 Vac 3-phase, 10 kVA max | stm:8057-8067 | NEAR |
| RAT output monitoring and control | RGCU | RAT (via RAT control breaker) | ? (RGCU monitors output and closes RAT control breaker; other path not stated) | ? | stm:8061-8063; ata24:4821-4822 | NEAR |
| RAT isolation of R2 bus | BB ISO relay | R2 235 Vac bus | Discrete (relay) | ? | stm:8068-8070 | OK |
| Battery state to ESIC | Main and APU batteries (via BCU) | ESIC / electrical synoptic | ? | Voltage, current, charge status | stm:7508-7512 | OK |
| EEC primary power | PMA | EEC | Engine-driven PMA (hardwired) | ? | lot_fcom.txt:80870 | QUOTED |
| EEC secondary power | 115 Vac bus | EEC | Hardwired | 115 Vac | lot_fcom.txt:80864 | QUOTED |
| Thrust lever angle (TLA) | TCM (thrust lever resolvers) | EEC (both channels) | Analog (resolver) | TRA position signal | stm:16047-16051; stm:16068-16072 | NEAR |
| Fuel control switch positions | FCM | EEC and CCS | CDN | ? | stm:16053-16055 | NEAR |
| Air data to EEC | CDN | EEC | CDN (loss of air data -> soft alternate) | ? | stm:16054-16058 | OK |
| Engine parameters to display | EEC | CCR (DCAS/EICAS) | CDN | ? | stm:15744-15748; stm:16583-16584 | OK |
| Engine fault data | EEC | CMCF | ? (path not stated) | ? | stm:15740-15743 | NEAR |
| Igniter control | EEC | Igniter exciters | ? | ? | stm:16702-16708 | OK |
| Igniter power | Capt instrument bus (one igniter unit); F/O instrument bus (other igniter unit) | Igniter units | Hardwired, via P300/P400 relays | 28 Vdc | stm.txt:16905 | QUOTED |
| Engine status sensors | Engine sensors | EEC | ? | ? | stm:16132-16134 | NEAR |
| Fuel height sensor signal | Fuel height sensors | FQDC | Analog (capacitance) to FQDC, converted to digital | ? | stm:14377-14381; ata28:3951-3952 | NEAR |
| Fuel density / compensator | Compensator and density sensors | FQDC | Analog (capacitance / vibrating element) | ? | stm:14382-14389; ata28:3946-3953 | NEAR |
| FQDC to RDC | FQDC (each channel) | RDC | CAN bus | ? | ata28.txt:4244 | QUOTED |
| FQDC to FQMS | FQDC (3) | FQMS (CCR) | CDN per stm:14370-14373; ata28:3952-3953 says via RDCs (see Gaps) | ? | stm:14370-14373; ata28:3952-3953 | NEAR |
| Pump and valve control | FQMS | Fuel pumps and crossfeed valve | ? | ? | stm:14259-14263 | OK |
| Pump and crossfeed switch positions | Pump and crossfeed switches | FQMS | CDN | ? | stm:14251-14254 | NEAR |
| Jettison switch positions | Jettison switches | FQMS | CDN | ? | stm:14249-14250 | NEAR |
| Refuel panel data | RCP | FQMS | ? (CDN not stated) | ? | ata28:1019-1025, 1035 | NEAR |
| APU fuel shutoff valve control | APUC | APU SOV | Commanded by APUC; signal path not stated | Open/close | stm:13767-13768 | NEAR |
| APU fuel module control | APUC | FMU motor controller and solenoids | ? | ? | stm:14997-14998 | OK |
| APU start signal | APUC | BPCU / CMSC | ? | ? | stm:15231-15233; stm:15208-15211 | OK |
| APU status to displays | APUC | HDD and maintenance laptop | ? | ? | stm:14765-14766 | OK |
| APU maintenance data | ISM | Non-volatile memory | ? | Hours, cycles, pressure, temp, speed, EGT | stm:14759-14766 | OK |
| NGS operation | NGS control software (right CCR) | NGS pack | ? | ? | stm:14667-14668; stm:14578-14580 | OK |

## Hydraulics, landing gear, flight controls

| Data item | From box | To box(es) | Path (CDN/ACS/FOX/RDC, ARINC 429, CAN, discrete, analog, hardwired) | Units / range / rate if stated | Source | Check |
|---|---|---|---|---|---|---|
| Control column position | Control column RVDTs (2 clusters) | ACE (FCE) | Analog | ? | A27:4871-4890 | OK |
| Control wheel position | Wheel RVDT clusters (2 x 4, 3 per cluster to FCE) | ACE (FCE) | Analog | ? | A27:1883-1891 | OK |
| Rudder pedal position | Pedal RVDT clusters (2 x 3) | ACE (FCE) | Analog | ? | A27:3914-3918 | OK |
| Control wheel / column / pedal force | Force transducers | ACE (FCE); flight data recorder | Analog | ? | A27:1845-1892, 3916; A27:4871-4890 | OK |
| Pilot commands (normal/secondary) | ACE | FCM | FCE intermodule buses / flight controls data bus (digital; protocol ?) | ? | A27:645-651, 884-896 | OK |
| Surface commands | FCM | ACE | FCE intermodule buses (digital; protocol ?) | ? | A27:645-651; STM:19390-19392 | OK |
| Surface commands to actuator servo | ACE / FCE | REU (aileron, elevator, rudder), SREU, EMCU | Digital command, FCE intermodule bus; then servo analog to PCU | ? | A27:1883-1889; STM:19390-19392 | OK |
| PCU LVDT position feedback | PCU LVDT | REU, SREU, EMCU (analog); ACE (digital) | Analog (LVDT); digital to ACE | ? | A27:1870-1873, 4886-4888 | NEAR |
| Spoiler and speedbrake commands | SREU and EMCU | Spoiler PCU (analog servo); EMA (235 VAC motor drive) | Analog servo; 235 VAC power | 235 VAC, 3-phase, variable frequency | STM:19482-19487; A27:3523-3540 | OK |
| Speedbrake lever transducer | Speedbrake lever | ACE, then FCE | Analog | ? | STM:19471-19495 | OK |
| Direct mode rate sensor data | DMRS (4) | ACE | Analog (?) | ? | A27:605, 1088 | OK |
| Air data, navigation receiver data, sensor inputs | Air data, nav receiver, inertial sources | PFCF (FCE) | CDN (ARINC 429 data buses) | ? | STM:19191-19197; A27:614-615 | OK |
| Autopilot commands | AFF (in FCE cabinets) | PFCF (FCE) | ? | ? | STM:19189-19191 | NEAR |
| PFCF indications and status | FCM | CDN (to DCAS / displays) | CDN, ARINC 429 | ? | A27:614-615, 893-894 | OK |
| PFC disconnect switch | P5 switch | FCE | Discrete | ? | A27:641-643 | OK |
| Flight control surface lock switch | Lock switch | FCE | Discrete | ? | A27:643 | OK |
| Pitch trim switch (control wheel) | Trim switches (4) | FCE (ARM and control signals) | Not stated | ? | A27:5743-5748, 5770 | NEAR |
| Stabilizer trim commands | FCE L and FCE C2 (FCM commands on intermodule buses) | HSTA EMCU L2 and R2 | Digital on HSTA actuation data buses (protocol ?) | Direction and rate command | A27:6128-6133 | NEAR |
| Stabilizer position | Stabilizer position modules (3) | EMCU, then FCE | Analog | ? | A27:5773-5776 | OK |
| Stabilizer cutout | Cutout switches (L2, R2) | EMCU; EMCU data to FCE | Discrete | ? | A27:5974-5978 | OK |
| Alternate pitch trim | Alternate trim switches | HSTA EMCU (direct wired) | Discrete (hardwired) | ? | A27:6382-6392 | OK |
| Flap lever position | Flap lever (4 RVDTs) | FCE L, R, C1, C2 | Analog (RVDT) | ? | A27:7353-7354, 8261-8271 | NEAR |
| Flap and slat solenoid commands | ACEs (FCE) | HCM solenoid valves | ACE solenoid control (path type ?) | ? | ata27.txt:7684 | QUOTED |
| Slat and flap motor power and drive | Slat EMCU | Slat and flap PDU electric motors | 235 VAC power (per EMCU description) | ? | STM:20341; A27:7233-7234 | OK |
| Flap skew position sensors | Flap skew position sensors (RVDT) | FCE; EMCUs for spoilers 5 and 10 (alternate droop) | Analog (RVDT) | ? | A27:3068-3069, 3516, 3773, 3841 | OK |
| Electric motor status (temp, resolver) | EMCU | FCE (fault monitoring) | ? | ? | A27:3535-3540 | OK |
| Electric motor pump speed command | Load management / CMSC | EMP motor controller | CDN (data bus); fallback 5500 rpm if CDN fails | 2700 rpm (ground load management); 5500 rpm (takeoff, start); 6800 rpm (thrust reverser) | A29:704-716, 1824-1826, 1870, 2682, 1841 | OK |
| EMP switch (OFF/AUTO/ON) | P5 switch | EMP motor controller | Hardwired discrete | ? | A29:1898-1902 | OK |
| EMP sensor data | Electric motor sensor | HYDIF | ? | ? | A29:1866-1872 (approx) | OK |
| EDP solenoid and shutoff | P5 EDP switch | EDP solenoid, ESSOV | Discrete (hardwired) | ? | A29:666-668; STM:17317-17330 | OK |
| Reservoir quantity | Reservoir quantity LVDT | HYDIF, then DCAS | Analog to HYDIF; data to DCAS | 0.00-1.00 (empty to full); LO below 0.40 | STM:17407-17420; A29:951-955 (approx) | OK |
| System pressure | Pressure transducers | HYDIF | Analog (?) | ? | STM:17412-17416; A29:1990-1999 | OK |
| Hydraulic indications | HYDIF | DCAS, EICAS, synoptic, maintenance pages | ARINC 664 (AFDX) per A29; not in path list | ? | A29:448-452; STM:17403-17412 | OK |
| RAT deploy command | HYDIF or BPCU | RAT deployment actuator solenoids | Hosted command: HYDIF or BPCU (A29:3072); other path ? | RAT full power: hydraulic 6 s (A29:3366) or 6.35 s (A29:3569); electrical 10 s (A29:3367, 3569). STM:17743-17745 gives 10 s for hydraulic and electrical | A29:3072, 3366-3367, 3569; STM:17743-17745 | OK |
| RAT deploy switch | P5 guarded switch | RAT deployment solenoids | Discrete (hardwired) | ? | A29:649-666 | OK |
| RAT stow command | P57 stow switch | RAT stow solenoid | Discrete; center system power needed | ? | A29:3080-3081, 3136; STM:17608-17613 | OK |
| RAT status (RPM, press, pos) | RAT sensors | HYDIF, maintenance pages | ? | RPM, psig, position | A29:2927-2930, 3027-3030 | OK |
| RAT generator voltage control | RAT GCU | RAT generator; backup bus | Electrical | 235 VAC, 10 kVA (generator) | A29:3057, 3067-3069 | OK |
| Landing gear lever position | Lever module (P2) | RDC (ground discretes), then CDN | Discrete to RDC; CDN | ? | A32:1468-1481; STM:18526-18530 | OK |
| Landing gear control proximity sensors | Control prox sensors | PSDC (analog to digital), then RDC, then CDN | Analog; CAN bus (PSDC to RDC); CDN | ? | A32:1486, 1507-1513 | NEAR |
| Indication proximity sensor data | Indication prox sensors | PSDC, then RDC, then CDN; then DCAS / HDD | Analog; CAN; CDN | ? | A32:5161-5165, 5001; STM:18396-18466 (approx) | OK |
| Air/ground data | Tilt and strut-compressed sensors | PSDC (MLG wells) | Analog to PSDC (A32:1486 for prox sensors); onward path not stated in cited lines | ? | A32:1176-1178, 1210, 1234-1235 | NEAR |
| Door and truck position | PSDCs | RDC, then CDN | CAN, then CDN | ? | A32:4843-4856, 5536 | OK |
| Gear and steering commands | LGA/NWS function (CCS) | Bypass/auto-off valve modules (solenoids) | CDN to RDC (ground side); RPDU (power side) | 28 VDC (RPDU) | A32:1071-1076, 1716-1717 | OK |
| Gear sequence power | RPDU | Solenoids, PSDCs, lever lock | 28 VDC | 28 VDC | A32:1075-1076, 1468-1472 | OK |
| Gear, door and truck status for display | Indication prox sensors (via PSDC) | Flight deck indication (display list ?) | Display path (bus ?) | ? | A32:1019-1022, 1160-1161, 341-342 | NEAR |
| Brake pedal position (captain and F/O) | Brake pedal sensors (2 left, 2 right) | BSCU left and right | Analog (brake pedal sensor analog inputs) | ? | A32:6490-6491, 6505 | NEAR |
| Parking brake lever | Parking brake lever switch | BSCU left and right | Not stated | ? | A32:6493-6495 | NEAR |
| Autobrake selector and RTO | Autobrake switch | Left BSCU outboard channel | Not stated | Levels 1-4, MAX AUTO, RTO | A32:6502-6504; FCOM p.311 | NEAR |
| Thrust lever angle (autobrake) | EECs | BSCU left and right | CDN (common data network) | ? | A32:6496-6497, 6511 | NEAR |
| Wheel speed | Magnet ring (per MLG wheel) | ARDC, then BSCU | Magnetic sensing; CAN bus to BSCU | ? | A32:6494; STM:18891-18896 | OK |
| Brake temperature | Brake temperature sensor (8, one per MLG brake) | MLG ARDC, then BSCU, then CDN | Connects to ARDC; ARDC data to BSCU on CAN; BSCU to CDN | ? | ata32.txt:6887 | QUOTED |
| Tire pressure and temperature | TPS (1 per wheel) | ARDC (wireless RF), then BSCU, then CDN | Wireless RF (ARDC interrogation); ARDC to BSCU on CAN; BSCU to CDN | Pressure corrected for ambient temperature (rate ?) | A32:6327-6336, 6403-6413, 6545-6546 | NEAR |
| Brake position commands | BSCU channel | EBAC (4) | CAN bus | ? | A32:6486-6487, 6562-6563 | NEAR |
| Brake power enable | BSCU channel | EBPSU | Not stated (power enable) | ? | A32:6519-6523 | NEAR |
| EBPSU power to EBAC | EBPSU | EBAC | Electrical | +/-130 VDC | ata32.txt:6865 | QUOTED |
| EBAC motor drive | EBAC | EBA (8 per EBAC) | Electrical (wire bundle to each EBA) | ? | ata32.txt:6917 | QUOTED |
| EBA data | EBA | EBAC (CAN), then CDN | CAN; CDN | ? | A32:6492, 6536-6537, 6565 | NEAR |
| BSCU data | BSCU | Other BSCU channels (CDN); EBACs (CAN) | CDN; CAN bus to EBACs | ? | ata32.txt:6832 | QUOTED |
| ARDC power | BSCU | MLG ARDCs (8), NLG ARDCs (2) | Electrical | 40 VDC (MLG, from related BSCU); 28 VDC (NLG, from right BSCU) | ata32.txt:6870 | QUOTED |
| Nose wheel steering commands | LGA/NWS (CCS) | NWS REU via CDN | CDN | ? | A32:5796, 5821-5823 | NEAR |
| Tiller and rudder pedal inputs | Tiller module; FCE rudder pedal position | LGA/NWS function (CCS) via CDN | CDN (rudder input through FCE to CDN); tiller path ? | Tiller up to 70 deg (low speed); max 8 deg at 100 kn | A32:5792-5794, 5818-5820, 5890-5893 | NEAR |
| NWS wheel position feedback | NWS position transducer modules (2) | NWS REU | Analog (RVDT signals) | ? | ata32.txt:6109 | QUOTED |
| NWS REU command to EHSV | NWS REU | EHSV and SOPV (NWS valve module) | Electrical servo (analog ?) | ? | ata32.txt:6293 | QUOTED |
| Nose wheel position feedback | NWS position sensor | NWS REU | Analog (feedback) | ? | A32:6105 | OK |
| Tow disconnect switch | Tow lever switches | NWS REU | Not stated | ? | ata32.txt:6295 | QUOTED |
| Hydraulic system HYDIF data bus | HYDIF (CCS GPMs) | Other systems | ARINC 664 | ? | A29:448 | OK |

---

## Environmental, ice and rain, fire, cabin, lights, doors, cargo

| Data item | From box | To box(es) | Path | Units/range/rate if stated | Source | Check |
|---|---|---|---|---|---|---|
| Pack control commands (pack switch on P5) | P5 panel | PCU | Via CCS (pack switch position is a CCS enable input) | P5 both packs AUTO sequence | stm:20945-20952, 20958-20960 | NEAR |
| PCU-to-PCU data | PCU (A) | PCU (B) | CAN | ? | stm:20829-20830; ata21:756-758 | OK |
| PCU data sources (CCS) | CCS | PCU | Direct data flow via CDN and RDC (redundant with CACTCS partition) | ? | stm:20837-20840; ata21:521-523, 739-740 | OK |
| CAC commands | PCU | CMSC | CAN (PCU communicates with CMSCs over CAN) | ? | stm:20958-20959; ata21:756-757, 957 | OK |
| CAC data (inlet/outlet pressure and temps) | CAC sensors | PCU | Direct sensor to PCU (path not otherwise stated) | ? | stm:20843-20845, 20944-20947; ata21:1138-1144, 1190-1195 | OK |
| CMSC power | ATRU | CMSC | 270V DC bus | +/-270V DC | stm.txt:21226 | QUOTED |
| CAC power/control power | CMSC | CAC | Control power from CMSC; CAC motor is variable speed AC | ? | stm.txt:21236 | QUOTED |
| ODS overheat alarm | ODS controller | CCS | Discrete or CAN (?) | Both channels must agree (AND) | stm:21164-21169 | OK |
| Pack shutdown command | CCS | PCU | ? | ? | stm:21166-21170 | OK |
| NGS overheat alarm | ODS controller | CCS; CCS then shuts NGS hosted app in CCRs | ? (not stated) | ? | stm:21165-21169 | NEAR |
| Misc equipment smoke data | Smoke detector | CCS | ? | Smoke/dust, low airflow, overheat 212F (100C) (thresholds not re-read) | stm:22860-22900 (partial) | NEAR |
| CPCS data | CPCS | CCS (for aft equipment cooling) | ? | ? | stm.txt:21715 | QUOTED |
| Cabin/utility switch (P5) | P5 panel | CCS (misc equipment cooling) | Hardwired discrete (?) | ? | stm:22860-22900 | OK |
| LGV temperature/airflow data | Temp sensors (flight deck, cabin zones, OFCR, OFAR, crown, bulk cargo) and airflow sensor | CCS | Hosted application in CCS | ? | stm:23240-23243, 23251-23256 | NEAR |
| LGV fan control | CCS | LGV fans | ? | One fan normally; both on AVS | stm:23249-23262 | OK |
| Aft EE cooling switch position | P5 panel | CCS | ? | ? | stm:24107-24108 | NEAR |
| Aft EE smoke detector | Smoke detector | CCS | ? | ? | stm:24116-24117 | NEAR |
| Forward cargo target temp | P5 cargo temp panel | CCS | Hosted function in CCS | 70F (21C) target | stm:24212-24217 | NEAR |
| Forward cargo duct temp sensor | Duct sensor | CCS | ? | ? | stm:24222-24225 | NEAR |
| Icing data | PIDS | CCS (WIPS, EAI, CIPS) | Via CCS hosted functions | ? | stm:26690-26691, 26697-26704; ata30:142, 269-274 | OK |
| Air data input to PIDS | FCE ADRF | PIDS | ? (ADRF data via FCE) | Static pressure P0, Pt, TAT, AOA | stm:26693-26708; ata30:203-207 | OK |
| WIPS control | WING switch (P5) | WIPC via CCS | Via CCS | AUTO/ON | stm:26959-26968 | NEAR |
| WIPS heater mat power | WIPC (24 cards) | Heater mats | 235V AC from P100, P150, P200 | 235V AC | stm:26951-26952, 26961-26963 | OK |
| WIPS slat temperature | Center heater mat sensor | WIPC | ? | Controls heater power | stm:26944-26951 | NEAR |
| EAI control | MEDC | EAI valve | MEDC controls torque motor power and varying-current signal | Regulates core valve pressure | stm:26833-26838; ata30:344, 436 | OK |
| EAI fan valve / core valve status | Pressure sensors | MEDC | Sensor data to MEDC | ? | stm:26818-26819; ata30:336-337 | OK |
| DLODS leak alarm | DLODS | MEDC / EICAS | ? | EICAS caution message | stm:25556-25582 | OK |
| Fire detector loops | Fire detector elements | MEDC, then RDC, then PFPS function (CCS hosted) | MEDC to RDC to PFPS (ata26:204-207); MEDC CAN to RDC; RDC to CDN (ata26:341-344) | Fire vs overheat alarm set by temperature | stm:25433-25434, 25445-25447; ata26:204-207, 341-344 | OK |
| Fire switch squib discharge | Fire switch handle | Fire extinguisher squib | 28V DC; squib discharged by CCS hosted apps | Squib on first pull-up, then turn | stm:25691-25697 | NEAR |
| Air data sensor heat control | P5 switches | ADRF / CCS | ? | Indications EICAS/HDD | stm:27162-27169 | OK |
| Window heat sensors | Window temp sensors (primary and spare per window) | CCS | ? | Controls power, overheat protection | stm:27281-27284 | NEAR |
| Window heat control | P5 switches | CCS | ? | De-select or reset | stm:27283-27286 | NEAR |
| Wiper control | Wiper control switch (P5) | WECUs | ? | INT: 1 cycle/7 s; LOW: 80 cycles/min; HIGH: 120 cycles/min | stm:27371-27378 | OK |
| Wiper data | WECUs | CCS via RDC; HDD maintenance pages | Via RDC | ? | stm:27390-27394 | NEAR |
| Oxygen mask flow pulse | Mask sensor / controller | Oxygen controller | ? | Pulses by blood saturation, 10,000-43,100 ft | stm:27775-27779 | OK |
| PSU door open command | CCS | PSU oxygen doors | Hosted application in CCS | Cabin alt > 15,000 ft | stm:27773-27778 | NEAR |
| EDW power | CSS / switch controller | EDW | Power via leads embedded between glass layers | Transparent to opaque; PED: 2 settings; passenger: 5 settings | stm:29832-29838 | NEAR |
| Door proximity sensors | Proximity sensors (up to 16 per PSDC) | PSDC | ? | ? | stm:29659-29661 | NEAR |
| PSDC / EPAS door data | PSDC, EPAS modules | CCS (proximity sensor system hosted app) | ? | Door status | stm:29638-29646 | OK |
| Cargo door open toggle | Toggle switch | RDC and RPDU | ? | Opens when on ground with ground power | stm:29495-29500 (not re-verified) | NEAR |
| Cargo handling command | MCP / joystick | CMDU, then PDUs | Joystick to CMDU; CMDU to PDUs via RDC and RPDUs | Single operator load/unload | stm:29987-29990, 29993-29999 | NEAR |
| Flight deck light level | Flight deck light controls (switches) | Light power control units (PCU) | Monitored and controlled by CCS hosted application | Adjustable intensity LEDs | stm:28619-28627 | NEAR |
| Cabin light levels | CCS hosted app (not stated) | Passenger cabin lights | ? | Software controlled LEDs | stm:28478-28482 | NEAR |
| Engine anti-ice fan valve pressure | Fan valve inlet and outlet pressure sensors | MEDC | Sensor to MEDC | ? | ata30:596-600; stm:26818-26819 | OK |
| CMSC enable/inhibit | CCS | CMSC | CDN | Enable and inhibit inputs (airplane config, in-air/ground, OAT, P5 pack switch) | stm.txt:21222 | QUOTED |
| Cargo SMCU squib control | SMCU | Cargo fire squibs and flow valves | ? | Discharge and flow valve control | stm:26273-26275 | NEAR |
| Cargo fire automatic discharge | CCS hosted apps | Cargo fire extinguishers | Hosted application | Cargo fire alarm with airplane in air and at least one engine running | stm:26273-26279 | NEAR |
| PCU channel power | RPDU (one per channel) | PCU channel | Power from different RPDU per channel | ? | stm:20821-20822; ata21:754, 762-763 | OK |
