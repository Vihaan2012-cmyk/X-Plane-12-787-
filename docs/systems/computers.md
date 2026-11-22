# Every computer on the 787-9 (Trent 1000)

Generated from the documents by reading agents, then checked automatically. **Check** column:
- **OK:** the row's terms appear at its cited line.
- **QUOTED:** a second pass found a verbatim quote at the citation, confirmed by script.
- **NEAR:** a partial match; read it before relying on it.
- **BAD:** the citation doesn't support the row; it needs fixing.

Citations point at the text extracts in `refs/text/` (`stm` = System Technical Manual, `ataNN` / `ANN` = 147 Technic ATA book, `loc` = Locator, `lot_fcom` = LOT FCOM). The model for each box is [shared-computer.md](shared-computer.md).

## Resolved conflicts

| Item | Sources disagree | Resolution |
|---|---|---|
| Main/APU battery | STM 65 Ah, 29.6 V; ATA 24 50 Ah, 28 V | **NTSB AIR-14/01 Table 1** (LVP65-8-402): 75 Ah nominal, 29.6 V nominal, 20–32.2 V operating, 8 cells × 3.7 V (2.5–4.025 V), 61.8 lb. The STM's 65 Ah is probably the rated figure in the LVP**65** part name. ATA 24's 50 Ah is treated as an error |
| Hydraulic fluid volume | STM 20 / 45 gal; ATA 29 reservoirs 5 / 11 gal | Not a conflict. The STM figures are total system fluid and the ATA figures are reservoir capacity. Both are used |
| FCE R location | Worded differently | All sources say the aft E/E bay |

The open conflicts are listed under each system below.

## Avionics and computing

| Box (full name) | Acronym | Qty | Location | Power source | Network / buses | Function (<=12 words) | Source | Check |
|---|---|---|---|---|---|---|---|---|
| Common computing resource cabinet | CCR | 2 (L, R) | Fwd EE bay; L CCR in E1 rack area | ? (PCMs feed cabinet modules) | CDN (ACS/FOX internal copper) | Houses GPMs, PCMs, ACS, FOX, GG; runs hosted apps | stm:742-749; stm:2016-2039 (p.5-6) | OK |
| Power conditioning module | PCM | 2 per CCR (4) | Inside CCR | 28v dc from one of 4 dc buses + hot battery bus; out 12.5v dc | Hard-wired reset (CCR reset switch) | Converts 28v dc to 12.5v dc for cabinet modules | stm:2041-2045; stm:2181-2191 | OK |
| General processing module | GPM | 8 per CCR (16) | Inside CCR | Via PCM (12.5v dc) | ACS (ARINC 664 CDN) | Runs hosted airplane-system apps; DCAS runs in three GPMs per CCR | stm:2024-2028, 2036-2040, 2101-2111, 2505-2508; pdf p.54 | NEAR |
| Graphics generator | GG | 2 per CCR (4) | Inside CCR; 4 in PDS (ata31: L1/L2 left, R1/R2 right) | Via CCR PCMs | ACS (ARINC 664 commands in); 6 fiber pixel outputs out | Formats display images for HDDs and HUD projectors | stm:2678-2702; ata31:403-430 | OK |
| Fiber optic translator | FOX | 2 per CCR (4) | Inside CCR | Via PCM | Fiber (CDN) to ACS; copper to GPMs | Converts ARINC 664 data between fiber and copper | stm:2029-2033, 2041-2046 | OK |
| ARINC 664 network cabinet switch | ACS | 2 per CCR (4; ch A/B) | Inside CCR | Via PCM | ARINC 664 CDN: 20 copper 100 Mbps internal; 4 copper 10 Mbps external | Switches CDN traffic among GPMs, GGs, FOX, RDCs, systems | stm:2023-2039 (p.5-6) | OK |
| ARINC 664 remote network switch | ARS | 6 (3 ch A, 3 ch B; 2 fwd, 4 aft) | Fuselage: Fwd L/R, Aft L1/L2, R1/R2 | ? | ARINC 664 CDN: 4 fiber 100 Mbps; 20 copper 10 Mbps direct inputs | Controls data flow between FOX and RDCs; monitors flows | stm:2129-2154 (p.5-8); pdf p.50 | OK |
| Remote data concentrator | RDC | 21 | Throughout airplane (stm:1958-1960) | ? | ARINC 664 to ACS/ARS (dual channel, ch A + ch B); CAN, ARINC 429, analog, discrete on system side | Interfaces non-ARINC 664 systems (CAN, ARINC 429, analog, discretes) to the CDN | stm:1956-1980; stm:2133-2134 | NEAR |
| CCR reset switch | CCR RESET | 2 (L, R, guarded) | P5 panel | Hard-wired to PCMs | Discrete (hard-wired) | Reboots CCR cabinets; crew use if HDDs fail in flight | stm:2181-2195 (p.5-9) | OK |
| Head-down display | HDD | 5 (capt/F-O inboard and outboard, lower center) | Instrument panels; lower HDD center | RPDU 71 (LIB), 73 (LOB), 72 (RIB), 74 (ROB); lower HDD via P300 | Fiber pixel buses from GGs; status back to GGs | 12.1 x 9.1 in LCD; PFD, ND, EICAS, MFD, checklist | stm:2474-2481, 2681; ata31:307-330, 360-366 | OK |
| Head-up display projector | HUD projector (HPU) | 2 (capt, F-O) | Overhead projection unit | L: RPDU 75; R: RPDU 76 | 6th GG pixel output (onside GG) | Projects flight symbology onto combiner | stm:5032-5036, 2701-2702; ata31:360-400 (power: ata31:370, 381) | OK |
| HUD combiner | HUD combiner | 2 | Windshield (glass plate) | Powered by HUD projector (+/-15v dc) | Optical | Combines HUD symbols with outside view | stm:5041-5050; ata31:371-372, 382-383 | OK |
| Remote light sensor | RLS | 2 (L, R) | Flight deck | +24v / -5v dc from onside inboard HDD (ata31:876) | ? | Ambient light sensing for automatic brightness | ata31:334-335, 361, 856-876 | OK |
| EFIS/DSP control panel | EFIS/DSP | 2 (captain, F-O) | Flight deck (onside) | L: captain instrument bus via RPDU 73; R: RPDU 74 | ARINC 429 to RDCs | Controls onside PFD/ND (EFIS) and MFD (DSP); backup mode | ata31:653-706; stm:2899-2930 (p.6-5) | OK |
| Multifunction keypad | MFK | 2 | Flight deck | L: P300; R: RPDU 74 | ARINC 429 bus | Alphanumeric entry; CDU, MFD, EICAS selections | ata31:253-254, 712-725; ata31 power section | OK |
| Cursor control device | CCD | 2 | Flight deck (control area) | L: P300; R: RPDU 74 | ARINC 429 bus to 2 RDCs | Touchpad and selector switches; cursor for lower MFD, onside inboard HDD, EFB | ata31:755-769 | NEAR |
| Instrument source select panel | ISSP | ? | Flight deck | ? | ? | Selects onside PFD/MFD source | stm:2525-2526; stm:2930 | OK |
| Clock switch | CLOCK SW | 2 (L, R) | Flight deck | ? | ? | Controls onside clock function (clock display on auxiliary display) | ata31:236, 325-326; stm:2818, 2832 | OK |
| Brightness control panel | BCP | ? | Flight deck | ? | ? | Manual brightness of onside HDDs, onside HUD, lower MFD | stm:2824-2831 | NEAR |
| Integrated standby flight display | ISFD | 1 | Flight deck (location ?) | 28v dc from center 1 FCE | CDN (backup airspeed/alt to DCAS); left INR (ILS); FCE (heading, GS) | Backup pitch, roll, heading, IAS, altitude, ILS deviation | stm:11208-11235; fcom p.94 (RVSM: do not fly on ISFD alone) | OK |
| Flight control electronics cabinet | FCE | 4 (L, C1, C2, R) | FCE L, C1, C2 in fwd EE bay; FCE R in aft EE bay | ? | ? | Hosts AFF and air data reference function; ATC altitude | stm:735-740, 792; stm:10318-10335; stm:11754-11760 | OK |
| Pitot air data module | ADM (pitot) | 3 (C, L, R) | Fuselage (locator component locations) | ? | Digital signal to FCEs (not stated as CDN) | Pitot pressure to air data reference | stm.txt:10471 | QUOTED |
| Static air data module | ADM (static) | 3 (C, L, R) | Fuselage (locator component locations) | ? | Digital signal to FCEs (not stated as CDN) | Static pressure to air data reference | stm.txt:10471 | QUOTED |
| Total air temperature probe | TAT | 1 | Fuselage (loc: 209, 80L, 164) | Heat element 115v ac via RPDU | Temperature data to FCEs (path not stated) | Total air temperature to FCE | stm:10327, 10341-10343; stm:27151-27159; locator.txt:4092 | OK |
| Angle of attack sensor | AOA | 2 (L, R) | Fuselage (loc: 211, 83L/R, 192) | Heat elements 115v ac via RPDU | AOA data to FCEs (path not stated) | Angle of attack to FCE; SWF input | stm:10330, 10341-10343; stm:5240-5244; stm:27151-27155; locator.txt:4087-4089 | NEAR |
| Inertial reference unit | IRU | 2 (L, R) | Aft EE bay on trays (stm:10463-10464; loc: 1250, 53L/R, 128) | ? | CDN (hybrid output); FCE and INR inputs | Attitude, position, velocity; 3 ring laser gyros, 3 accelerometers | stm:10455-10477; stm:10463-10464; locator.txt:5015 | NEAR |
| Attitude heading reference unit | AHRU | 2 (L, R) | Aft EE bay on trays (stm:10463-10464; loc: 1241, 53L/R, 128) | ? | CDN | Attitude and heading; hybrid inertial/GPS data | stm:10455-10479; stm:10463-10464; locator.txt:5009-5012 | NEAR |
| Airplane personality module | APM | 4 | With IRU/AHRU trays in aft EE bay | ? | ? | Corrects tray-to-airplane axis misalignment | stm.txt:10615 | QUOTED |
| Integrated navigation receiver | INR | 2 (L in E2 rack; R in E8 rack) | E2 rack (fwd EE bay); E8 rack (fwd cargo door) | ? | GPS to IRU/AHRU and FMF; ILS to ISFD (L) | VOR, ILS, marker beacon, GPS, GLS receiver | stm:762-766, 860-865; stm:10492-10500 | OK |
| Automatic direction finder transceiver | ADF | 2 (L, R) | Rack not stated in cited lines (UNSUPPORTED for "E8 rack") | ? | ? | ADF bearing pointers on PFD mini-map; audio to flight deck speakers | stm:860-861; stm:11640-11644 | OK |
| DME interrogator | DME | 2 (L, R) | E8 rack | ? | To FMF and DCAS | Slant-range distance to ground stations | stm:830-855; stm:11491-11512 | OK |
| Radio altimeter transceiver | RA | 2 (own antennas) | E8 rack | ? | To DCAS, AFF, TMF, TAWS, TCAS | Radio altitude, 0 to 2,500 ft | stm:857-858; stm:11352-11372 | OK |
| Integrated surveillance system processor unit | ISSPU (ISS) | 2 (L, R) | E2 rack (L and R) | ? | CDN; ATC/TCAS/WXR/TAWS inside | Hosts ATC transponder, TCAS, weather radar, TAWS | stm:752, 770; stm:11658-11690 | OK |
| ATC/TCAS antenna | ATC/TCAS ant | 2 (top, bottom) | Top and bottom of airplane | ? | RF to ISSPU | Directional antennas for ATC and TCAS | stm:11929-11950; stm:658-692 (p.3-2) | OK |
| Weather radar (RTM, antenna, drive) | WXR | 2 RTMs; 1 antenna; 1 drive | Antenna location ? | ? | Fiber from RTM to ISSPU; ERS attitude data | Weather returns, predictive windshear, map display | stm:12095-12118 | OK |
| VHF transceiver | VHF | 3 (L, C, R) | L in E2 rack; C and R in E4 rack | ? | CDN via AGU/TCP; analog audio to antenna | VHF voice and data communication | stm:725-760, 797-818; stm:8962-8990; loc:1617-1647 | OK |
| VHF antenna | VHF ant | 3 | ? | ? | RF | VHF transmit and receive | stm:8962-8975 | OK |
| HF transceiver | HF | 2 (L, R) | E11 rack (fwd side of aft cargo door) | ? | CDN via AGU/TCP (RDC) | HF voice communication for long range | stm:869-883; stm:9124-9150; loc:1590-1593 | OK |
| HF antenna coupler | HF coupler | 2 | Vertical stabilizer, below antenna | ? | RF | Matches HF antenna impedance | stm:9128-9150; loc:1604-1606 | OK |
| SATCOM transceiver (SRT) | SATCOM | 2 (L in E3, R in E4) | E3 and E4 racks (aft EE bay) | ? | CDN via AGU; DLNA module to HGA | Satellite voice and data communication | stm:801, 817; stm:9281-9300 | OK |
| High gain antenna | HGA | 1 | ? | ? | RF via DLNA module (stm:9293-9294) | SATCOM antenna | stm:9293-9309 | NEAR |
| Audio control panel | ACP | 3 (captain, F-O, observer) | Glareshield (capt, F-O); observer station | ? | CDN (digital audio via AGU); left ACP direct to left AGU on CDN failure | Selects audio sources (comm, nav, cabin, PA, flight interphone, SATCOM); PTT; SELCAL reset | stm:8414-8437; stm:9443-9446 | OK |
| Audio gateway unit | AGU | 4 (E2 L, E2 R, E3 L aft, E4 R aft) | E2, E3, E4 racks | ? | CDN; analog audio to radios | Converts digital audio to/from analog for radios and cabin | stm:755-756, 802, 820; stm:8962-8990; loc (AGU L/R) | OK |
| Tuning control panel | TCP | 3 | P8 aft aisle stand | ? | RDC/CDN to radios; direct to left VHF on CDN failure | Tunes VHF/HF/SATCOM; selects ATC, TCAS, GPWS, WXR pages | stm:8805-8835; stm:8966-8985 | OK |
| Emergency locator transmitter | ELT | 1 (plus control panel, AIM, program switch module) | Antenna per locator.txt:1722 (top of airplane not stated in cited lines; UNSUPPORTED for "top") | ? | ? | Transmits 121.5 and 243.0 MHz; digital data every 50 s on 406 MHz | stm:9788-9801; stm:9810; locator.txt:1696-1722 | NEAR |
| Flight recorder | FR | 2 (fwd, aft; each with ULB) | Fwd and aft (loc: fwd, aft entries) | RIPS (recorder independent power supply); fwd FR from 28v dc bus (ata31:3491) | CDN (fiber optic); core network interface | Stores 2 h audio and 25 h flight data | ata31:3399-3520; loc:4259-4290 | OK |
| Core network cabinet | CNC | 1 | Fwd EE bay (ata46:382) | ? | ODN and IDN; 3 routers, 2 switches; 5 FSM slots | Hosts CSM, FSM, EGM, NIM for ground/maintenance data | ata46:245, 382, 546, 621-633, 1019-1020; stm:746; loc:873 | NEAR |
| Terminal wireless LAN unit | TWLU | ? (1 implied) | Passenger cabin, with antenna | ? | ODN; wireless to ground network; TCS | Wireless link between airplane and ground network | ata46:129-130, 193-194; stm:686-688 | OK |
| Crew wireless LAN unit | CWLU | ? (optional) | External and internal antennas | ? | ODN (wireless maintenance laptops) | Wireless connection for maintenance laptops | ata46:26-62, 275-276 | OK |
| EFB electronic unit | EFB EU | 2 (captain, F-O) | E2 rack | ? | Core network (EFB interface) | Electronic flight bag processing | stm:763, 769; loc:853, 859; ata46:136 | OK |
| EFB display unit | EFB DU | ? | Flight deck | ? | ? | Display for EFB | ata46:137 | OK |
| Flight management function (FMF) | FMF | 3 (master, hot spare, backup) | CCR cabinets | CCR (via PCM) | CDN; ARINC 664 (?) | Lateral/vertical guidance, navigation, performance | stm:12744-12765 | OK |
| Thrust management function | TMF | ? (in CCR) | CCR cabinets (stm:13130-13131) | CCR | Two independent servomotors to thrust levers (stm:13132-13134) | Controls thrust levers from takeoff to touchdown | stm:13118-13121, 13130-13134 | NEAR |
| Autoflight function (AFF) | AFF | Software in 3 FCEs | FCE cabinets | FCE | ? | Autopilot, flight director, autothrottle, go-around | stm:12954-12975 | OK |
| AFDS hardware | MCP / TOGA / BDA | MCP 1; TOGA 2; BDA 3; A/T disconnect switches on T/Ls (count ?) | Flight deck (MCP); control columns/wheels | ? | Backdrive commands to BDAs (stm:12968-12971); PFCF not stated in cited lines | Autopilot commands to surfaces and controls | stm:12959-12971; stm:13122-13124 | OK |
| Control display unit | CDU | ? (on MFD via MFK) | Flight deck (MFD) | ? | DCAS / MFK | Flight plan, performance and navigation pages | stm:12823-12840; stm:2466 | OK |
| Flight deck printer | FDP | ? | Flight deck | 115v ac from RPDU 75 (ata31:3887) | IDN (core network) | Prints flight deck and EFB data | ata31:3880-3887; ata46:282, 296 | NEAR |
| Cabin services system controller | CSSC | ? | E2 rack | ? | CDN | Cabin interphone and PA routing (CSS, out of scope) | stm:759; stm:10031-10050 | OK |

### 3. GPM hosted applications

Transcribed from the rendered table on **PDF page 54 (book page 5-7)** of `e:\X-Plane 787\refs\incoming\787- System Technical Manual by Boeing RR1K.pdf`. The text version (stm.txt:2050-2095) lists only the application names. The dots are cell fills. Red cells are in the L columns and blue cells in the R columns. No legend appears on the page, so what red and blue mean (primary vs secondary host) is "?". "--" means no marked cell.

| Hosted application | Red cells (L GPM) | Blue cells (R GPM) | NOCITE | NOCITE |
|---|---|---|---|---|
| Cabin Air Temperature Control System | L4 | -- | NOCITE | NOCITE |
| Equipment Cooling System | L1 | R2 | NOCITE | NOCITE |
| Integrated Cooling System | L1 | R6 | NOCITE | NOCITE |
| Low Pressure System | L4 | R6 | NOCITE | NOCITE |
| Power Electronics Cooling System | L1 | R4 | NOCITE | NOCITE |
| Communication Management Function | L8 | R8 | NOCITE | NOCITE |
| Switches - Flt Deck and Control Panels | L6 | R6 | NOCITE | NOCITE |
| Circuit Breaker Indication & Control | L1, L4 | R1 | NOCITE | NOCITE |
| Electrical Power Distribution & Control | L1, L4, L8 | R1 | NOCITE | NOCITE |
| Engine Fire Protection System | L1, L6 | R1 | NOCITE | NOCITE |
| Cargo Fire Protection System | L8 | R1 | NOCITE | NOCITE |
| Fuel Quantity System | L1, L6 | R1 | NOCITE | NOCITE |
| Hydraulic System Control | L1, L4 | R4 | NOCITE | NOCITE |
| WWFDS, EAI & CIPS | L1 | R1 | NOCITE | NOCITE |
| Window Heat System | L1 | R4, R5 | NOCITE | NOCITE |
| Display Crew Alerting System | L2, L3, L4 | R2, R3, R6 | NOCITE | NOCITE |
| Landing Gear - Indication & Control | L1, L7 | R1, R7 | NOCITE | NOCITE |
| Lighting Systems | L2, L4 | R3, R6 | NOCITE | NOCITE |
| Thrust Management Function | L5 | R5 | NOCITE | NOCITE |
| Flight Management Function & NDB | L5, L6 | R5 | NOCITE | NOCITE |
| Water & Waste Systems | L5, L6 | R1, R6 | NOCITE | NOCITE |
| ACMF | -- | R1 | NOCITE | NOCITE |
| Central Maintenance Computing System | L7 | R7 | NOCITE | NOCITE |
| Nitrogen Generation System | -- | R4 | NOCITE | NOCITE |
| Doors - Indication & Control | L1, L7 | R1, R4 | NOCITE | NOCITE |

Counts: 25 applications. Red/blue meaning "?".

### Conflicts

- DCAS/DCAF GPM count and copies: stm:2506-2507 says DCAS is "located in three general processing modules (GPM) in each". ata31:1512 says "Each CCR cabinet uses 3 GPMs with the DCAF software and 2 GGs to". ata31:203 says "There are 4 copies of the DCAF software." Three GPMs per CCR gives six, not four. The earlier note that DCAF is in "GPM 2 and 3" was not found in the sources and is removed. The PDF table (red L2-L4, blue R2/R3/R6) still agrees with three per side.
- Locator glossary: locator.txt:557 reads "RDC Ramp Damage Checker". stm:1956-1957 reads "There are 21 remote data / concentrators (RDC) in the common". The STM definition is used here.
- ARS count (consistent, no conflict): stm:2129-2131 reads "Two ARSs are located in the forward / part of the airplane and four are in the aft part". stm:2131-2133 reads "There are six ARINC 664 network / remote switches (ARS)". The STM diagram (p.5-3, pdf p.50) shows Fwd L/R and Aft L1/L2, R1/R2.

### Gaps

- Flight deck (stm ch 4, lines 960-1611) was not read beyond the escape/overview pages. Panel and switch inventory is incomplete.
- stm ch 6 (lines 2900-5020) and ch 9 (SELCAL, CMF menus, cabin interphone) were skimmed, not fully read. Several DCAS and comm sub-functions are missing.
- stm ch 10 TAWS modes, ISS details and ch 11 autoflight modes were sampled only.
- FCOM (lot_fcom.txt) yielded few component facts; chapters 4, 5, 10, 11 and 15 are mostly OCR-garbled procedures and limitations. Only p.94 (ISFD RVSM), p.97 (ACP/SATCOM limitation), p.207, p.215, p.279, p.283 (chapter starts) were cited. Ch 15 warning systems not extracted.
- Power sources are "?" for most RF, nav, air data and network boxes; the documents only give power for CCR, flight deck panels, HDDs, HUD projectors, the FDP, and the air data heat elements.
- Data-bus type is "?" for most sensor-to-computer links (ADM, AOA, TAT, IRU/AHRU, INR, DME, RA).
- No units or rates were given in the documents for most flows (only CDN speeds, ELT frequencies, RA range, altitude alert thresholds, FR capacities).
- Locator coordinates (loc: x, y, z) are kept as-is and not converted; the coordinates themselves were not checked against the locator.
- Rack and location claims for ADF (E8 rack) and ELT (top of airplane) are not confirmed in the cited lines and are flagged UNSUPPORTED.
- CSS, CMCF ground tests, and FR record details beyond capacity were not extracted (out of scope or not read).
- Hosted application color legend missing from PDF p.54; the table is transcribed from the image, not the text layer.

## Electrical, engines, APU, fuel

| Box (full name) | Acronym | Qty | Location | Power source | Network / buses | Function | Source | Check |
|---|---|---|---|---|---|---|---|---|
| Variable frequency starter generator | VFSG | 4 (L1, L2, R1, R2; 2 per engine) | Engine gearbox (2 VFSGs on each engine gearbox) | Engine rotor speed (driven directly from main engine gearbox; no constant speed drive) | Feeds its own 235 Vac main bus L1/L2/R1/R2 | Starts engine; generates 235 Vac in flight; up to 250 kVA | ata24:3121-3126; ata24:4488-4495; stm:6720-6725 | OK |
| APU starter generator | ASG | 2 (L, R) | APU gearbox (ata24:2688) | APU speed (driven by APU gearbox, ata24:2741); APU start power from APU 28 Vdc battery or external power (ata24:495) | 235 Vac main buses (an ASG can feed a bus if VFSG power goes off) | Starts APU (torque to turn APU, ata24:2586); generates 235 Vac; up to 225 kVA | stm:6722-6726; ata24:495, 2586, 2688, 2741, 4488-4495 | OK |
| Generator control unit | GCU | 6 (one per VFSG/ASG) | ? | ? | Isolated data bus to BPCU; TTP/C to BPCU; L2 GCU also controls aft EPC | Controls and protects its generator; L2 GCU monitors aft ext power | ata24:509; stm:6973-6977; ata24:902-903, 960, 1101 | OK |
| RAT generator | RAT | 1 | Ram air turbine (spring-loaded actuator into airstream) | Ram air | Backup bus via RAT control breaker; R2 235 Vac bus isolated by BB ISO relay | Emergency 230 Vac 3-phase, up to 10 kVA, power in 10 s | stm:8054-8070 [8-16]; stm:6728-6730; fcom:80775 (p1608, 6.20.4) | OK |
| RAT generator control unit | RGCU | 1 | ? | ? | ? | Monitors and controls RAT output | stm:8061-8063; ata24:511, 327 | OK |
| Main battery | Main Batt | 1 (lithium-ion) | Forward E/E bay | BCU (charger); BDM isolates | HBB via BDM; Capt and F/O instrument buses via MBR | Power for RAT deployment, emergency braking, refuel, power-up | stm:6714-6717, 7498-7525 [8-11]; ata24:544; fcom:80775-80800 (p1608) | OK |
| APU battery | APU Batt | 1 (lithium-ion) | Aft E/E bay (stm:6719) | APU BCU (dedicated BCU, stm:7519-7520) | APU HBB (direct, stm:7505-7506) | APU start (SPU); towing nav lights | stm:6719, 7505-7506, 7513-7520 | OK |
| Main battery charger unit | BCU | 1 (main) | Forward E/E bay (stm:6714-6716) | Unregulated 28 Vdc from Capt instrument bus (ata24:684-695) | Charges main battery | Regulated 46 A, 32.2 Vdc charge output; 20% in 1.5 h | ata24:684-695; stm:6714-6716, 7519-7521 | NEAR |
| APU battery charger unit | BCU | 1 (APU) | Aft E/E bay | Unregulated 28 Vdc from F/O instrument bus | Charges APU battery | Regulated 46 A, 32.2 Vdc charge output | stm:6719; ata24:690-695 | OK |
| Battery diode module | BDM | 1 | Left forward E/E bay, on beam in front of P300 panel (ata24:765-767) | Main battery | Between main battery and HBB (ata24:756-758) | Ensures main battery charged only through BCU; 75 A CB | ata24:742-767; stm:7522-7525 | NEAR |
| Main battery relay | MBR | 1 | ? | ? | Closed by BPCUs; connects main battery to Capt/F/O instrument buses | Connects main battery to instrument buses when no other source | ata24:782-795; stm:7500-7503, 7738 | OK |
| Bus power control unit | BPCU | 2 (P300 and P400) | P300 and P400 panels (fwd E/E bay) | 28 Vdc from TRUs and batteries | CDN gateway; TTP/C; isolated data bus to GCUs; EPC control | Main EPS control; ext power control; load shed management | ata24:963-1010, 1051-1070; stm:6705-6708 | OK |
| Power distribution panel | P100 | 1 | Aft E/E bay (ata24:4530-4543) | 235 Vac L1/L2 generators (ata24:4473-4474) | 235 Vac bus L1/L2; tie bus (ata24:4473-4474) | Generator power panel: houses 2 GCUs, GCBs, GNRs, BTBs, ELCCs, ELCUs and start contactors; does not house ATRUs | ata24:4468-4481, 4530-4543; ata24:4338; stm:6715-6718 | OK |
| Power distribution panel | P150 | 1 | Aft E/E bay (ata24:2867) | 235 Vac from APU starter generators (ata24:2853) | Left tie bus (LTB) and right tie bus (RTB) to P100/P200 (ata24:2854-2858) | Auxiliary power panel: APU GCUs (AGCU), GNRs, auxiliary power breakers, APU start contactors, WIPS load contactors (ata24:2860-2865) | ata24:2848-2867; ata24:396; stm:6715-6718 | OK |
| Power distribution panel | P200 | 1 | Aft E/E bay (ata24:4530-4543) | 235 Vac R1/R2 generators (ata24:4498) | 235 Vac bus R1/R2 (ata24:4472-4473) | Generator power panel (right side); houses 2 GCUs, GCB, BTB, ELCC and ELCUs | ata24:4468-4481, 4498-4503, 4530-4543; ata24:4339; stm:6715-6718 | OK |
| Power distribution panel | P300 | 1 | Forward E/E bay (stm:6705-6707) | 115 Vac from ATU and 28 Vdc from TRUs in P500/P600 (ata24:1604-1606) | L 115 Vac bus, L 28 Vdc bus, Capt instrument bus (ata24:1601-1606) | Power distribution panel; hosts BPCU L, ELCU L3/L4, SPDU(s), external power TRU (L) (ata24:1128) | ata24:1583-1610; ata24:1128; ata24:963; stm:6705-6707 | OK |
| Power distribution panel | P400 | 1 | Forward E/E bay (stm:6705-6707) | 115 Vac from ATU and 28 Vdc from TRUs in P500/P600 (ata24:1604-1606) | R 115 Vac bus, R 28 Vdc bus, F/O instrument bus (ata24:1601-1606); RAT generator distribution bus (ata24:1586) | Power distribution panel; hosts BPCU R, ELCU R3/R4, SPDU(s), external power TRU (R) (ata24:1129) | ata24:1583-1610; ata24:1129; ata24:963; stm:6705-6707 | OK |
| Power conversion panel | P500 | 1 | Forward E/E bay (stm:6705-6707) | 235 Vac from P100/P200 (ata24:1390); 115 Vac from forward external power (ata24:1391) | ATU (P500) feeds L 115 Vac bus in P300 (stm:7399-7404) | Power conversion; hosts ATU (left), 2 TRUs, 2 EBPSUs (ata24:1508-1533) | ata24:1389-1391, 1429, 1508-1533; stm:6705-6707, 7399-7404 | OK |
| Power conversion panel | P600 | 1 | Forward E/E bay (stm:6705-6707) | 235 Vac from P100/P200 (ata24:1390); 115 Vac from right forward external power (ata24:928-929) | R 115 Vac bus (ata24:1415); ATU in ground external power mode feeds TRUs and 235 backup bus (stm:7378-7384) | Power conversion; hosts ATU (right), 2 TRUs, 2 EBPSUs (ata24:1508-1533) | ata24:1389-1391, 1429-1430, 1508-1533, 928-929; stm:6705-6707, 7378-7384 | OK |
| High voltage DC panel | P700 | 1 | Aft E/E bay | 235 Vac L1/L2 | +/-270 Vdc L1/L2 | Hosts 2 ATRUs (E5 rack) and 4 CMSCs | stm:6718, 7214-7217, 7916-7917; loc:972-976 | OK |
| High voltage DC panel | P800 | 1 | Aft E/E bay | 235 Vac R1/R2 | +/-270 Vdc R1/R2 | Hosts 2 ATRUs (E6 rack) and 4 CMSCs | stm:6718, 7214-7217, 7916-7917; loc:972-976 | OK |
| Flight deck electrical control panel | P5 | 1 | Flight deck overhead | ? | ? | Gen, drive, battery, EXT PWR switches and indications | stm:7033-7068 | OK |
| Towing power panel | P5 towing | 1 | Flight deck overhead (stm:7033-7068) | ? | ? | Battery test (test switch and indicator lights), towing power, battery state | stm:7505-7506, 7515-7518; stm:6993-7007 | OK |
| Auto transformer rectifier unit | ATRU | 4 (L1, L2 in P700; R1, R2 in P800) | P700 and P800 | 235 Vac main bus | +/-270 Vdc to CMSCs | Converts 235 Vac to +/-270 Vdc for motor controllers | stm:7214-7217, 7589-7591; ata24:509; loc:944 | OK |
| Common motor start controller | CMSC | 8 | 4 in P700, 4 in P800 | +/-270 Vdc from ATRUs | Variable-frequency 235 Vac output to large motors | Drives hydraulic EMPs, CACs, RAM fans, engine and APU start | stm:7589-7592, 7916-7917, 6982-6990; ata24:509; loc:972 | OK |
| Auto transformer unit | ATU | 2 (one in P500, one in P600) | P500 and P600 | 235 Vac (bidirectional) | 235 Vac <-> 115 Vac | Converts 235 Vac to 115 Vac (and reverse with ext power) | stm:6793-6794, 7378-7404, 7750-7753; ata24:1420 | OK |
| Galley auto transformer unit | GATU | 1 | Left aft cargo sidewall, forward of bulk cargo door (ata24:1569) | 235 Vac 3-phase from L2 bus through GATU contactor (ata24:1575-1576) | 115 Vac 3-phase to aft galley; GATU contactor controlled by GCU; BPCU monitors GATU (ata24:1563-1565) | Supplies 115 Vac for aft galley | ata24:1546-1577; stm:7758-7759 | NEAR |
| Transformer rectifier unit | TRU | 4 (L, C1, C2, R) | P500 (L, C1) and P600 (C2, R) per ata24:1458-1459; STM drawing places the TRU row under P300/P400 (stm:7217-7229); see Conflicts | 235 Vac (ata24:1455) | 28 Vdc L/R buses; Capt and F/O instrument buses; each TRU feeds an EBPSU (ata24:1459-1461) | Converts 235 Vac to 28 Vdc (31 Vdc per stm:6800-6801; 28 Vdc per stm:7747-7748; see Conflicts); backup bus feeds C1/C2 TRUs (ata24:1586-1588) | ata24:1455-1462, 1586-1588; stm:6800-6807, 7747-7748, 7217-7229 | OK |
| Electric brake power supply unit | EBPSU | 4 (2 per P500/P600) | P500 (2) and P600 (2) (ata24:1487-1489) | 28 Vdc from TRU output and from hot battery bus (ata24:1483-1485); HBB is backup (ata24:1497-1498) | 28 Vdc in, +/-130 Vdc out to electric brake actuation controllers (ata24:1495-1496) | Regulates power to electric brake system | ata24:1476-1498; stm:7755-7756 | OK |
| Electrical load control unit | ELCU | 8 (P100: L1, L2; P200: R1, R2; P300: L3, L4; P400: R3, R4) | P100, P200, P300, P400 | 235 Vac / 115 Vac from panels | TTP/C from BPCU | Load shed and load control for large loads | stm:7566-7578, 7632-7652; loc:904-910; ata24:955, 1004 | OK |
| Secondary power distribution unit | SPDU | 4 (2 in P300, 2 in P400) | P300, P400 | 28 Vdc | BPCU gateway to CDN; TTP/C | Distributes 28 Vdc loads under 50 A and feeds RPDUs | stm:7566-7574, 7688-7690; ata24:513, 1637-1638, 1852-1854 | OK |
| Remote power distribution unit | RPDU | 17 (4 gateway, 13 standard) | Throughout fuselage (fwd and aft EE bays and cabin) | 28 Vdc from SPDUs (P300/P400); 115 Vac from ELCUs | Gateway: CDN + TTP/C; standard: TTP/C to gateway | Supplies loads under 10 A (115 Vac and 28 Vdc) | stm:7599-7688; fcom:80900-80980 (p1611, 6.20.7); ata24:513, 1977-1978 | OK |
| Solid state power controller | SSPC | ? (per RPDU; per-RPDU count not stated) | Inside RPDU (RPDU supplies loads through SSPCs, stm:7667-7681) | 115 Vac or 28 Vdc | CDN to BPCU (microprocessor modules, ata24:1878); controlled through CBIC | Switches and protects individual RPDU loads | stm:7665-7681; ata24:1876-1880 | NEAR |
| Power electronics cooling system | PECS | 1 (loop) | Liquid cooling loop | ? | ? | Cools HVDC motor controllers; energized when engine/APU running or fwd ext power ON | ata24:476; stm:7897; fcom:81096-81112 (p1613, 6.20.9) | OK |
| Permanent magnet generator | PMG | 3 | Engine driven (fcom:80858) | Engine rotation | Flight control electronics (independent of airplane electrical system, fcom:80858-80861) | Primary power for flight control electronics; backed by 28 Vdc bus and batteries | fcom:80856-80866 (p1610) | NOCITE |
| Permanent magnet alternator | PMA | 2 (one per engine) | Engine driven, one per engine (fcom:80870-80872) | Engine rotation | Primary power to EEC; secondary from 115 Vac bus (fcom:80874-80877) | Primary power for EEC; backup from airplane 115 Vac when PMA inoperative | lot_fcom.txt:80870 | QUOTED |
| Engine electronic controller | EEC | ? (2 channels per engine, stm:16121-16123) | ? (engine-mounted; physical location not stated) | PMA primary; 115 Vac secondary (fcom:80870-80877; stm:16125-16127) | CDN (air data, FCM fuel switches); CCR cabinets (engine parameters to display); CMCF (fault data) | FADEC: thrust management, engine systems, starting, thrust reverser operation, fuel flow calculation | stm:15728-15748, 16050-16058, 16106-16127 | NEAR |
| Thrust control module | TCM | ? | ? (thrust levers and TLA resolvers; physical location not stated) | ? | TLA resolver signals to both EEC channels (stm:16047-16051) | Thrust levers, TLA resolvers, autothrottle servos, FCM, reverser lockout, autothrottle disengage switches, TOGA | stm.txt:16266 | QUOTED |
| Fuel control module | FCM | ? (in TCM, stm:16059-16060) | ? (component of TCM) | ? | Fuel switch positions to EECs and CCS via CDN (stm:16053-16055) | Sends fuel control switch position data; fuel switches also control P300/P400 fuel control relays that open and close engine fuel spar valves (stm:16063-16066) | stm:16053-16066 | NEAR |
| Igniter exciter and igniter | IGN | 2 systems per engine (one on Capt bus, one on F/O bus) | ? | 28 Vdc: Capt instrument bus powers one igniter unit, F/O instrument bus the other, via P300/P400 relays (stm:16692-16700) | EEC controls igniter units (stm:16702-16708) | Spark for engine start and relight | stm:16684-16708 | NEAR |
| Engine starter motor | ? | 2 (two starter motors on N2 shaft, fcom:82773-82790) | Engine accessory gearbox (fcom:82773-82790) | Electrical system (APU or forward ext power, fcom:82773-82790) | Driven via CMSC start (stm:6982-6990); START selectors control starter motors (fcom:82773-82790) | Starts engine; starter cutout speed not stated | fcom:82773-82790 (7.20.10); stm:6982-6990 | NEAR |
| APU controller | APUC | 1 | E12 rack, bulk cargo compartment (stm:14773-14774) | ? (APUC power source not stated) | Signals BPCU for APU start (stm:15229-15233); HDD and maintenance laptop (stm:14765-14766) | Controls all APU phases; protective shutdown; stores fault info | stm:14759-14774, 15229-15233 | OK |
| APU fuel shutoff valve | APU SOV | 1 | APU fuel feed | Main airplane battery (fcom:83236) | Opened and closed by APUC (stm:13767-13768) | Shuts APU fuel supply; closes on APU fire (stm:13771-13775) | stm:13763-13775; fcom:83236 | NEAR |
| APU fuel module unit | FMU | 1 | APU | Hot battery bus power | Motor controller and solenoid valves controlled by APUC | Meters primary and secondary APU fuel; fuel pump speed control | stm:14977-14998, 15398-15420 | OK |
| APU start power unit | SPU | 1 | APU | APU battery (28 Vdc) | Hot battery bus (APU HBB) | APU start power | stm:7513-7515, 6774-6775; fcom:83240-83260 | OK |
| APU inlet sensor module | ISM | 1 | APU inlet | ? | Non-volatile memory (maintenance data) | Records APU hours, cycles, pressure, temperature, speed, EGT data | stm:14759-14805 | OK |
| APU | APS5000A | 1 | Tailcone (stm:14767-14768) | Starter generators (stm:6722-6723) | 235 Vac buses via ASGs | Auxiliary power; can start at all altitudes up to 43,100 ft; electrical power available up to 43,100 ft | stm:14761-14768; stm:6722-6723 | NEAR |
| Fuel quantity data concentrator | FQDC | 3 (ata28:3962) | Not stated in cited lines; fuel sensors are in wing and center tanks (stm:14379-14394) | ? | CAN bus to RDCs (ata28:4029-4030); CDN to FQMS (stm:14366-14372) | Reads fuel sensor signals and converts to correct format; interface to CDN | ata28:3946-3962, 4029-4030; stm:14366-14372 | NEAR |
| Fuel quantity management system (hosted app) | FQMS | 3 copies (2 left, 1 right CCR) | CCR cabinets (ata28:1025-1028) | ? | CDN; sensor data from FQDCs; pumps and crossfeed valve control | Calculates fuel weight, controls pumps/crossfeed, monitors faults | ata28:1019-1028; stm:14248-14263, 14370-14373 | NEAR |
| Fuel quantity sensors | FQIS sensors | Main tank: 14 height, 1 height/temp, 1 compensator, 1 density. Center: 9 height, 1 height/temp, 2 compensator | Inside each tank | FQDC excitation | Capacitance signals to FQDC | Measure fuel height, temperature, density | stm:14379-14394; ata28:3951 | OK |
| Fuel boost pump | Boost pump | 4 (two per main tank, stm:13871-13872; with 2 O/J pumps = "all six fuel pumps", stm:13880) | Main fuel tanks (stm:13871-13872) | ? (pumps in a tank are powered from different sources for redundancy; sources not named, stm:13873-13878) | FQMS controls pumps via CDN (stm:14248-14263) | Feeds fuel to engine feed manifold (stm:13868-13870) | stm:13868-13880; stm:14248-14263 | OK |
| Override/jettison pump | O/J pump | ? | Fuel tanks | ? | Motor controller | Supplies fuel to engines when boost pumps fail; jettison pumping | stm:13882-13893, 14113; ata28:98-100 | OK |
| Scavenge jet pump | Scavenge jet | 2 | ? | Boost pump motive flow | ? | Removes remaining fuel from tanks | stm:13982-13997 | OK |
| Fuel jettison nozzle valve | Jet nozzle | ? | ? | ? | FQMS monitors jettison switches via CDN | Dumps fuel overboard; computes max landing weight | stm:14094-14120, 14248-14250; ata28:132-139 | OK |
| Refuel control panel | RCP | 1 | Refuel station, leading edge of left wing (stm:13513-13516) | Refuel door proximity sensor signal powers RCP (ata28:1019-1035) | FQMS (refuel valve control); CDN path not stated (ata28:1019-1025) | Refuel/defuel control and status | stm:13513-13531; ata28:1019-1035 | NEAR |
| Fuel shutoff module | ? | ? | ? | Emergency power (per ata28) | CAN bus interface | Closes fuel shutoff valves | ata28:2685-2714 (context not verified) | OK |
| Nitrogen generation system pack | NGS | 1 | Fuselage (NGS Pack) | Passenger cabin air; motor compressors (via CMSC) | Distribution system to tanks; NGS control software | Produces nitrogen enriched air for fuel tank inerting | stm:14570-14586, 14651-14672, 7912; ata28:227-233, 965-984; loc:5940 | OK |
| Electrical system indication and control | ESIC | 1 (software) | ? (host not stated) | ? | Interface to electrical system contactors, CBs and bus isolation | Maintenance and control pages for contactors, CBs, bus isolation | stm:6608-6616; stm:7245-7261 | NEAR |
| Backup bus isolation relay | BB ISO | 1 | Backup bus (ata24:1782-1793) | ? | Isolates R2 235 Vac bus (stm:8068-8070) | Opens during RAT deployment | stm:8064-8070; ata24:1782-1793 | NEAR |

### Ratings and bus names

- VFSG: up to 250 kVA each, 4 units, 235 Vac variable frequency. Sources: stm:6725; fcom:80617-80700 (p1605).
- ASG: up to 225 kVA each, 2 units. Sources: stm:6725-6726, 14691; ata24:509.
- External power: 2 forward receptacles (90 kVA each), 1 aft receptacle (rated 90 kVA, limited to 65 kVA). Sources: stm:6705-6712; fcom:80720-80775 (p1607). Forward minimum for engine start: two 90 kVA sources. Sources: fcom:80720 (p1607).
- RAT: 230 Vac 3-phase, 10 kVA max, deploys in 10 s. Sources: stm:8054-8062.
- Main battery: lithium-ion. Capacity 65 Ah (stm:7500-7502) vs 50 Ah (ata24:544) (CONFLICT, see Conflicts). Nominal 29.6 Vdc (stm:6720-6721) vs 28 Vdc (ata24:544) (CONFLICT, see Conflicts).
- APU battery: lithium-ion; same rating as main per stm:7500-7502 and ata24:544 (capacity conflict as above).
- BCU: regulated 46 A, 32.2 Vdc output. Sources: ata24:684-695. Charges battery 20% in 1.5 h: stm:7520-7521.
- BDM: 75 A circuit breaker. Source: ata24:765.
- HVDC power: about 600 kW (PECS load). Source: ata24:476.
- 235 Vac main buses: L1, L2, R1, R2 (main); backup bus (235 Vac, fed from R2 normally). Sources: stm:6749-6750, 6809-6817; fcom:80900-80980 (p1612, 6.20.8).
- 115 Vac buses: L 115V AC bus, R 115V AC bus. Sources: stm:6764-6765, 6767; fcom:80930-80960.
- 28 Vdc buses: L 28V DC bus, R 28V DC bus, Captain instrument bus (CIT), First officer instrument bus (FIT), Start power unit (SPU) bus. Sources: stm:6771-6774; ata24:1000 range.
- Hot battery buses: HBB (main), APU HBB. Sources: stm:6779, 7498-7499.
- +/-270 Vdc buses: L1, L2, R1, R2 (270 V DC). Sources: stm:6773-6774, 7214-7217.
- +/-130 Vdc: E-BPSU outputs. Sources: stm:7755-7756; ata24:1480-1481.
- RPDUs: 17 total (4 gateway, 13 standard); loads under 10 A. Sources: stm:7676-7677, 7674; fcom:80900-80980 (p1611).
- SPDUs: loads under 50 A. Source: stm:7573-7574.
- Engine: Trent 1000, dual channel FADEC. Source: stm:15728-15733.
- EEC: two channels per engine. Source: stm:16121-16123.
- APU: Hamilton Sundstrand APS5000A, altitude to 43,100 ft. Source: stm:14767-14768, 14763-14765.
- Fuel quantity: FQIS sensor groups A and B (dual isolated). Source: stm:14373-14374.
- FQMS: 3 copies in CCR (2 left, 1 right). Source: ata28:1026-1028.
- Igniter: 2 systems per engine. Source: stm:16684-16685.

### Conflicts

Each item quotes both sources. Edition is given for each.

1. Main battery capacity and voltage (65 Ah vs 50 Ah; 29.6 V vs 28 V)
   - Capacity, STM (Rev 1.0), stm:7500-7502: "The main and APU batteries are lithium ion batteries rated at 65 ampere hours."
   - Capacity, training book (Revision 0, 01 Jan 2022), ata24:544: "battery is a 28v dc lithium-ion battery with a capacity of 50 ampere-hours."
   - Voltage, STM (Rev 1.0), stm:6720-6721: "Each battery supplies a nominal 29.6v dc power."
   - Voltage, training book (Revision 0, 01 Jan 2022), ata24:544: "28v dc lithium-ion battery" (same line as the 50 ampere-hour quote).
   - Not resolved. The STM and the training book are different editions and different document types.

2. TRU output voltage (31 V vs 28 V)
   - STM (Rev 1.0), stm:6800-6801: "Four transformer rectifier units (TRU) convert 235v ac power to 31v dc".
   - STM (Rev 1.0), stm:7747-7748: "The TRUs convert variable frequency 235v ac to 28v dc."
   - Training book (Revision 0, 01 Jan 2022), ata24:1455: "The transformer rectifier units (TRU) change 235v ac to 28v dc."
   - Both 28 V statements are in the STM and the training book; the 31 V statement is in the STM ground-power overview (page 8-4). Not resolved.

3. ATU panel (P500/P600 vs P300/P400)
   - P500/P600: STM (Rev 1.0), stm:7378-7379: "The ATU in the P600 power distribution panel." and stm:7399-7400: "The ATU in the P500 power distribution panel." Also stm:7749-7750: "The power conversion system is in the P500 and P600 panels." Training book (Revision 0, 01 Jan 2022), ata24:1429-1430: "One in the P500 panel / One in the P600 panel."
   - P300/P400: STM (Rev 1.0) drawing, stm:7217-7229: the schematic row labelled "ATU TRU TRU TRU TRU ATU" sits under the P300 and P400 labels. No text in the STM names P300/P400 for the ATU.
   - Not resolved. The STM text and training book say P500/P600; only the STM drawing puts the ATU and TRU row under P300/P400.

4. SPDU count (4 vs 2)
   - Training book (Revision 0, 01 Jan 2022), ata24:513: "Transformer Rectifier Units (TRU), 4 SPDUs and 17 RPDUs."
   - STM (Rev 1.0), stm:7549: "SPDUs (2)" (power layout diagram, no per-panel count).
   - The same training book also says at ata24:1854: "The 2 secondary power distribution units do these functions:".
   - Not resolved. The SPDU row keeps 4 (2 in P300, 2 in P400) from ata24:513.

### Gaps (not stated in the documents)

- Main battery capacity (65 Ah vs 50 Ah) and nominal voltage (29.6 V vs 28 V): see Conflicts; not resolved.
- Locations not stated: GCU, RGCU, MBR, TCM and EEC (physical), FCM, APU fuel module beyond APU, fuel pumps and jettison nozzles beyond tank level, FQDC positions, ESIC host, RAT airframe location.
- Resolved in this pass: BDM location (ata24:765-767). Fuel pumps: 4 boost pumps (two per main tank) and 2 override/jettison pumps in the center tank (stm:13868-13880); "all six fuel pumps" = 4 boost + 2 O/J.
- SPDU count: see Conflicts (4 vs 2).
- ATU and TRU placement: see Conflicts (P500/P600 in text vs P300/P400 in the STM drawing).
- Data path for most discrete control signals (EPC closure, APUC to SOV/FMU, FQMS pump and valve control, EEC to igniter, RGCU to RAT). Mostly "?".
- FQDC to FQMS path disagrees between sources: stm:14370-14373 says CDN to the FQMS in the CCR; ata28:3952-3953 says the FQDC values go to the FQMS through the RDCs.
- FQDC physical positions: the earlier draft's "flap support rib 3 / keel beam / right wheel well" has no support in the source text. The only right-wheel-well reference (ata28:305-306) is the center tank access panel, not an FQDC.
- ESIC host: the earlier draft's "hosted in CCS" has no source in the cited lines and was removed.
- APU controller electrical power source: not stated. The earlier draft's "APU battery powers APUC" has no source.
- Engine starter cutout speed: not stated.
- Fuel chapter FCOM (ch 12) system description body was not located in lot_fcom.txt (only the TOC at lot_fcom.txt:1562-1569 and fuel-related sections in ch 7 at lot_fcom.txt:83498). Not read.
- Engine EEC physical location and the engine sensor list (stm:16132-16134 only). Not read in full.
- RAT location on airframe. Not stated beyond "ram air turbine" in airstream.
- NGS controller location and compressor count (stm:14667-14668 partial). Not stated.
- Electrical power to ESIC and CCS: path not stated.
- ata28 section on fuel shutoff module (ata28:2685-2714) context was not verified; treat as unverified.
- ATA 24 pages for TRU/ATRU location beyond panel level.
- SSPC count per RPDU: ata24:1876-1880 gives SSPC counts per DC card (e.g., "DC-19 circuit card has 10-amp SSPCs (2), 5-amp SSPCs (5), and 2.5-amp SSPCs (12)"), not per RPDU.
- Interfaces for PECS pumps and reservoir location.
- Refuel panel data path: CDN not stated (ata28:1019-1025).

## Hydraulics, landing gear, flight controls

| Box (full name) | Acronym | Qty | Location | Power source | Network / buses | Function | Source | Check |
|---|---|---|---|---|---|---|---|---|
| Flight control electronics cabinet | FCE | 4 (L, C1, C2, R) | L, C1, C2 in fwd EE bay; R in aft EE center (A27:732, 708-712) | Own PCM (see PCM row) | FCE intermodule buses; CDN | Houses ACE, FCM, PCM; computes PFCF and HLF commands | A27:585-606, 732, 708-712; STM:19240-19244; LOC:2724-2746 | OK |
| Actuator control electronics | ACE | 4 (1 per FCE): ACE 1 in FCE L and C2; ACE 2 in FCE R and C1 | Inside each FCE | 28 VDC via PCM | FCE intermodule buses; CDN ? | Interfaces pilot sensors and surface actuators; passes FCM commands to REU/SREU/EMCU | A27:754-756, 843-850 | OK |
| Flight control module | FCM | 3 (L, C1, R; C2 has none) | Inside FCE L, C1, R | 28 VDC via PCM | Flight controls data bus network; CDN | Calculates primary and HLF surface commands; identical units | A27:884-915; STM:19238-19240 | OK |
| Power conditioning module | PCM | 4 (1 per FCE) | Inside each FCE | L: left VFSG PMG (primary), left 28V bus (secondary), FCE battery (backup). R: right VFSG PMG, right 28V bus, FCE battery. C1: left VFSG PMG (primary), captain FIB (secondary), hot battery bus (backup). C2: F/O FIB (primary/secondary), hot battery bus (backup) | ? | Supplies 28 VDC to FCE, ACE, FCM | A27:627-668, 783-829 | OK |
| FCE battery | ? | 2 (FCE L, FCE R) | Adjacent to FCE L and FCE R | Lithium-ion, charged in place (source ?) | ? | Backup for FCE L and R, min 10 min; dispatch needs >0.3 A and >27.3 V | A27:722-728, 990-1010; LOC:2730-2745 | OK |
| Direct mode rate sensor | DMRS | 4 | Behind aft liner/bulkhead area | ? | To ACE (direct mode) | Rate data for PFCF direct mode | A27:605, 673, 1088 | OK |
| Aileron and flaperon remote electronics unit | REU | 8 (1 per aileron/flaperon PCU and per flaperon manifold) | At each PCU, wing trailing edge | ? | FCE command in (digital); LVDT feedback out | Servo control of hydraulic PCU | A27:1883-1890, 2299-2310; LOC:2845-2857 | OK |
| Elevator remote electronics unit | REU | 4 (1 per elevator PCU) | On each elevator PCU | ? | From ACE; LVDT feedback to ACE | Servo control of elevator PCU | A27:4983-4985, 5333-5345 | OK |
| Rudder remote electronics unit | REU | 3 (1 per rudder PCU) | Trailing edge of vertical stabilizer | ? | From ACE/FCE | Servo control of rudder PCU | A27:4015-4016; LOC:2988-3003 | OK |
| Spoiler remote electronic unit | SREU | 10 | Wing structure, near each spoiler actuator; 5 per wing (A27:3110, 3207, 3370) | ? | Bi-directional digital data bus to FCE cabinets (A27:3363-3365); LVDT feedback | Servo control of hydraulic spoiler PCUs; monitors actuation failures; concentrates High Lift sensor data | A27:3207-3210, 3353-3370 | OK |
| Electric motor control unit (spoiler) | EMCU | 4 (1 per spoiler EMA: spoilers 4, 5, 10, 11) | ? | 235 VAC, 3-phase, variable frequency | From FCE (digital command) | Drives EMA motor and brake; monitors motor temp and resolver | A27:3031-3032, 3523-3540 | OK |
| Electric motor control unit (HSTA) | EMCU | 2 (L2 and R2) | Locator coordinates only: (Loc: 1933, 27R, 295) and (Loc: 1933, 23L, 296) | 235 VAC: R2 EMCU from 235v ac R2 bus; L2 EMCU from 235v ac L2 bus (A27:6106-6108) | From FCE L and C2 on HSTA actuation data buses (A27:6128-6130); alternate trim switches wired direct (A27:6389); stabilizer position data back to FCE (A27:5773-5774) | Drives HSTA motors; reports stabilizer position to FCE | A27:5773-5776, 6106-6108, 6128-6133, 6389; LOC:3112-3122 | OK |
| Electro-mechanical actuator (spoiler) | EMA | 4 (spoilers 4, 5, 10, 11; 2 per wing) | Spoilers 4 and 5 on left wing, 10 and 11 on right wing (A27:3412-3413) | 650 V DC brushless motor (A27:3426); its EMCU is powered by 235 VAC 3-phase variable frequency (A27:3535-3536) | EMCU on data buses; commands from FCE cabinets via ACE (A27:3421-3424) | Moves electric spoiler | A27:3109, 3412-3413, 3421-3426, 3535-3536 | OK |
| Power control unit (flight surface actuator) | PCU | 25 hydraulic: aileron 4, flaperon 4, spoiler 10 (inboard 4, outboard 6), elevator 4, rudder 3 | On each surface (see Surfaces section) | Hydraulic L/C/R (per surface) | Servo from REU/SREU/EMCU; LVDT feedback to REU, SREU or EMCU | Positions control surface | A27:1848-1850, 3107-3108, 3136, 4015-4016, 5384, 1870-1873 | NEAR |
| Horizontal stabilizer trim actuator | HSTA | 1 (2 DC motors, 2 secondary brakes, 1 mechanical primary brake, ballscrew) | Rear fuselage structure, aft fuselage; locator coordinates only (Loc: 1937, 0, 271) | Via HSTA EMCUs (L2, R2) | From EMCUs; position sensors (3) | Moves horizontal stabilizer (ballscrew) | A27:6024-6041, 5770-5776; LOC:3115-3119 | OK |
| Stabilizer position module | ? | 3 | Stabilizer | ? | Analog to two different EMCUs | Stabilizer position sensing | A27:5737-5776 | OK |
| Leading edge slat | ? | 12 (6 per wing) | Leading edge, both wings | Hydraulic (primary mode) or electric (secondary mode) | FCE | High lift; positions: cruise, takeoff (middle), landing (extended) | A27:7233; STM:20341-20350 | OK |
| Krueger flap | ? | 2 (1 per wing) | Wing leading edge, between engine strut and inboard slat | ? (GRA-driven) | FCE | High lift; retracted or deployed | A27:7239, 7235; STM:20234 | OK |
| Trailing edge flap | ? | 4 (1 inboard + 1 outboard per wing) | Trailing edge | Hydraulic or electric motor via PDU | FCE | High lift; flap positions per 787-8 and -9 settings | A27:7156, 7274; STM:20221-20223 | OK |
| Slat/flap power drive unit (LE) | PDU | ? | Forward left wing-to-body fairing, at left wing leading edge (A27:7236-7237) | Hydraulic (center, via HCM) or electric motor (270 VDC from slat EMCU) | Via HCM and EMCU | Turns slat torque tubes (A27:7754) | A27:7236-7237, 7161-7165, 7360-7362, 8284-8286 | OK |
| Trailing edge flap power drive unit | PDU | ? | Right main wheel well, aft inboard corner at top of wheel well (A27:7279-7280) | Hydraulic (center, via HCM) or electric motor (270 VDC from TE flap EMCU) | Via HCM and EMCU | Turns flap torque tubes; torque tubes connect PDU to GRAs (geared rotary actuators) | A27:7279-7280, 8288-8290, 8638-8640 | NEAR |
| Geared rotary actuator | GRA | 24 LE slat, 2 Krueger, TE flap count ? | In wing | Torque tube drive | ? | Extends or retracts flaps/slats through drive arms | A27:7235-7236; STM:20234 | OK |
| Flap/slat hydraulic control module | HCM | 1 | Keel beam web, right main wheel well (A27:7239, 7282) | Center hydraulic pressure in; 28 VDC solenoid power from PCMs in FCE R and C1 (A27:8277-8278) | Solenoid valves controlled by ACEs (A27:7360-7361) | Controls hydraulic power to flap/slat PDU motors | ata27.txt:7562 | QUOTED |
| Slat electric motor control unit | EMCU | 1 | Forward cargo compartment (AC mix bay), left side | ? | From FCE | Supplies power to electric motors on slat/flap PDUs (secondary mode) | A27:7233-7234; STM:20341 | OK |
| Flight control lock switch | ? | 2 | Flight deck | Discrete | To FCE | Locks surfaces on wing and tail independently | A27:643 | OK |
| PFC disconnect switch | ? | 1 | P5 overhead panel | Discrete | To FCE | Lets crew select direct mode | A27:641-643; STM:19302-19330 (approx) | OK |
| Control column | ? | 2 (connected by bus shaft with breakout) | Flight deck | Mechanical input | Two RVDT clusters (3 RVDTs each) and force transducer, analog to FCE (ACE); force to flight data recorder | Pitch input and elevator feel | ata27.txt:1946 | QUOTED |
| Control wheel | ? | 2 | Flight deck | Mechanical input | RVDT clusters (2 x 4) to ACE | Roll input, wheel force transducer, damper | A27:1845-1892 | OK |
| Rudder pedal assembly | ? | 2 sets (4 pedals) | Flight deck | Mechanical input | Two RVDT clusters (3 RVDTs each) and force transducers, analog to FCEs | Yaw input | ata27.txt:1946 | QUOTED |
| Speedbrake lever | ? | 1 | Flight deck control stand | Transducer signal | To ACE, then FCE | Commands spoilers for speedbrake | STM:19471-19495; A27:3020 | OK |
| Control wheel pitch trim switch | ? | 4 (2 per control wheel) | Outboard side of each control wheel (A27:5746-5748) | ? | Pitch trim control signals to FCE (A27:5743) | Electric pitch trim commands | ata27.txt:6010 | QUOTED |
| Alternate pitch trim switch | ? | 2 | Aisle stand | Discrete, wired direct to HSTA EMCUs | HSTA EMCU | Alternate pitch trim | A27:6382-6392 | OK |
| Rudder trim selector | ? | 1 | Aisle stand (A27:4342) | ? | FCEs send rudder trim commands to rudder trim actuator (A27:4355) | Rudder trim commands (manual switch and P-Beta automatic) | A27:3934-3939, 4338-4344, 4355 | NEAR |
| Alternate flap switch and selector | ? | ? | Flight deck (A27:7143-7145) | 28 VDC (A27:8292-8294) | ACEs in FCE L and C2 (A27:8273-8274) | Alternate flap and slat control | A27:7143-7145, 8273-8274, 8292-8294 | OK |
| Flap lever | ? | 1 (4 RVDTs into separate FCEs) | Right side of control stand (A27:7224) | RVDT (4) | RVDT data to FCE L, R, C1, C2 (A27:8264-8271) | Flap and slat position command (analog input to ACE) | A27:7224, 7353-7354, 8261-8271 | OK |
| Hydraulic pump controls and fault lights (P5) | ? | 1 panel | P5 overhead panel | ? | Hard-wired to pump controllers; HYDIF | EDP ON/OFF, EMP OFF/AUTO/ON, RAT deploy, fault lights | STM:17311-17322; A29:641-668 | OK |
| Engine driven pump | EDP | 2 (left engine, right engine) | Engine accessory gearbox | Engine mechanical drive | Solenoid and shutoff valve via P5 switch | Primary pump for left and right systems; 27-34 gpm at about 4775 psi | STM:17246-17247, 17311-17316; A29:1726-1727; LOC:3838 | OK |
| Engine supply shutoff valve | ESSOV | 2 (1 per EDP) | Left and right engine aft strut fairing | ? | ? | Shuts off EDP supply; closes on engine fire handle pull | A29:296; STM:17329-17330; LOC:3838 | OK |
| Electric motor pump | EMP | 4 (L and R demand; C1 and C2 center, one primary by calendar day) | L and R in engine aft strut fairings; C1 and C2 in aft wing-to-body fairing | L EMP and R EMP: R2 and L2 235 VAC buses (A29:437-438). C1 from L1, C2 from R1 (A29:440-441) | Motor controller; CDN (speed commands); P5 switch hard-wired to controller | Up to 37 gpm at 4750 psi; weight 112 lb; speeds 2700 and 5500 rpm, 6800 rpm during thrust reverser | A29:1793, 704-716, 1820-1902, 437-441; STM:17227-17233, 17316-17320 | OK |
| Ram air turbine hydraulic pump | RAT pump | 1 | RAT, right aft wing-to-body fairing | Air-driven by RAT | HYDIF | Center system pressure; 10 gpm at 2700-4850 psi | A29:3055-3070; STM:17184-17190 | OK |
| Ram air turbine | RAT | 1 | Right aft wing-to-body fairing, aft of right main wheel well | Air stream | HYDIF or BPCU commands deploy | Emergency hydraulic (center) and 10 kVA 235 VAC generator; 50 in turbine blade | A29:3043-3081, 3110-3121; STM:17608, 17743; LOC:3966-3993 | OK |
| RAT generator control unit | GCU | 1 | Forward EE bay (A29:3057) | RAT generator (10 kVA at 235 VAC) (A29:3067-3069) | ? | Regulates RAT generator output voltage (A29:3057-3058) | A29:3055-3058, 3067-3069 | NEAR |
| RAT stow switch (P57 panel) | ? | 1 | Left main wheel well electrical service panel (A29:3080-3081) | ? | ? (not stated) | Retracts RAT on ground only; center hydraulic pressure needed (A29:3076-3082) | A29:3075-3082 | NEAR |
| RAT deploy switch (P5) | ? | 1 | P5 overhead | ? | Discrete (guarded, momentary) | Manual RAT deploy; amber UNLKD and white PRESS lights | A29:649-666 | OK |
| Hydraulic interface function | HYDIF | 3 instances (Left, Right, Standby); Standby can replace L or R, not both | Hosted in CCS / CCR (GPM L1, L4, R4) | CCR | ARINC 664 data (A29:448); CDN | Controls and monitors hydraulics; sends indications to DCAS, EICAS, synoptic | A29:397-456, 743-755; STM:17577-17580, 17403-17412 | OK |
| Hydraulic system reservoir | ? | 3 (L, C, R) | L and R in engine aft strut fairing; C in left aft wing-to-body fairing | Not applicable | Quantity LVDT, temperature and pressure transducers to HYDIF | Holds fluid; pressurized by piston and bootstrap | STM:17231-17235; A29:802-803, 951-955 | OK |
| Hydraulic accumulator | ? | 4 (L, C, R, plus NLG circuit) | L and R in engine aft strut fairings; C in aft wing-to-body fairing; extra in nose wheel well (A29:278) | Nitrogen pre-charge, normal gas charge 2000 psi (A29:970) | Not applicable | Holds system pressure when pumps are off; keeps positive pressure for pump start (A29:967-969) | A29:278, 967-972, 1410-1414; STM:17246-17253 | NEAR |
| Hydraulic heat exchanger | ? | 4 (2 center, 1 L, 1 R) | Left and right main fuel tanks; center in fuel tank (2) | Not applicable | Not applicable | Cools hydraulic fluid before return | STM:17236-17241; A29:276 | OK |
| Landing gear control lever module | ? | 1 | P2 center instrument panel | RPDU 28 VDC (lock solenoid) | RDC ground discretes to CDN | UP and DN command; lock and LOCK OVRD | STM:18526-18540; A32:1468-1481 | OK |
| Landing gear actuation and NWS function | LGA/NWS | 1 function (2 control systems) | Hosted in CCS (CCR) | Not stated | CDN (commands); CAN bus via PSDC/RDC | Gear sequencing, door and truck control, nose wheel steering commands | A32:1460-1466, 1512-1513 | OK |
| Landing gear control proximity sensor | ? | NLG: down 2, up 2, door open 2; MLG per side: down 2, up 2, door open 2, truck stowed 2 | At gear, doors, truck | 28 VDC via RPDU | Analog to PSDC | Sequencing (target near/far) | A32:1018-1031, 1204-1215, 1486-1492 | OK |
| Landing gear indication proximity sensor | ? | ? (separate set from control sensors) | At gear and doors | ? | Analog to PSDC (A32:1486, 1511) | Indication to flight deck (A32:1019-1022, 1160-1161) | A32:1007-1008, 1019-1022, 1157-1161, 1486 | OK |
| Air/ground proximity sensor | ? | Per MLG truck: 2 truck tilt sensors (one in each truck position assembly) (A32:1210, 1234-1235) | MLG truck position assemblies (A32:1224-1226) | ? | To PSDC (A32:1176-1178); analog not stated | Air/ground status (A32:1176-1178, 1234-1235) | A32:1176-1178, 1210, 1224-1226, 1234-1235 | OK |
| Proximity sensing data concentrator | PSDC | 6 (2 in each MLG well: control and indication; 1 fwd cabin, 1 aft cabin) | 2 in left MLG wheel well (forward keel beam), 2 in right MLG wheel well (aft keel beam), 1 forward cabin above ceiling, 1 aft cabin (A32:1151-1152, 1170-1178) | ? | Analog in (prox sensors); CAN bus to RDC | Converts prox sensor analog data to digital | A32:1151-1158, 1170-1178, 1189, 1511-1513 | NEAR |
| Remote data concentrator | RDC | Many (see locator) | See locator (LOC:5542-5692) | Independent airplane power buses for system 1 and system 2 (A32:1716-1718) | CAN/discrete in; CDN out | Converts discrete and CAN data to CDN data | A32:1477-1478, 1512-1513, 1716-1718; LOC:5542-5692 | OK |
| Remote power distribution unit | RPDU | Many | Throughout aircraft | 28 VDC bus (upstream ?) | ? | Supplies 28 VDC to solenoids, PSDCs, lever lock | A32:1075-1076, 1716-1717; LOC:2329-2413 | OK |
| MLG retract actuator | ? | 2 (L, R) | MLG bay (A32:2150) | Center hydraulic, via MLG control valve module (A32:2346-2347) | Solenoid valves via CDN/RDC (A32:2341-2343, 2479) | Retracts MLG | A32:530-531, 2150, 2341-2347, 2479 | OK |
| MLG uplock actuator | ? | 2 (L, R) | MLG | Center hydraulic (A32:2346-2347) | Solenoid valves (A32:2479) | Locks MLG up | A32:537-538, 2346-2347 | OK |
| MLG downlock actuator | ? | 2 (L, R) | MLG | Center hydraulic (A32:2346-2347) | Solenoid valves | Locks MLG down | A32:530-531, 2124-2125, 2334-2335 | OK |
| MLG truck positioner actuator | ? | 2 (L, R) | MLG truck | Center hydraulic, direct from MLG control valve (A32:2355-2359) | Solenoid valves; truck tilt commands via CDN/RDC (A32:2341-2343) | Positions truck for retract and extend | A32:1572, 2291-2293, 2355-2359 | OK |
| MLG body door actuator | ? | 2 (L, R) | MLG | Center hydraulic | Solenoid valves | Opens and closes MLG body doors | STM:17980; A32:1845-1870 (approx) | OK |
| NLG retract actuator | ? | 1 | NLG bay, aft right side of NWW (A32:3047) | Center hydraulic (A32:1036) | Solenoid valves | Retracts NLG | A32:1036, 2947, 3025, 3047 | OK |
| NLG lock link actuator | ? | 1 | NLG (A32:3026, 3228) | Center hydraulic (A32:1036) | Solenoid valves | Locks NLG lock link (over-center fold and push) | A32:1036, 2890-2903, 3026, 3228 | OK |
| NLG forward door actuator | ? | 1 | NLG bay | Center hydraulic | Solenoid valves | Opens and closes forward NLG doors | STM:18351 (diagram label) | OK |
| Alternate extend power pack | ? | 1 | MLG/NLG release circuit (A32:3659-3662) | 28 VDC from hot battery bus (A32:3916-3917) | ? | Pressurizes center hydraulic fluid for alt-extend release actuators | A32:3659-3662, 3916-3917 | OK |
| Alternate extend release actuator | ? | 3 | Gear doors/uplocks | Center hydraulic, or alt-extend power pack | Solenoid | Releases doors and uplocks for free-fall extend | STM:18315, 18332, 18290-18303 | OK |
| Nose wheel steering remote electronics unit | REU | 1 | P40 panel, rear side of NLG shock strut (A32:4226-4227, 5849) | ? | CDN (commands from LGA/NWS function in CCS, A32:5821-5822); position transducer feedback (A32:5787-5789) | Controls EHSV and SOPV in NWS valve module (A32:5979-5980) | A32:5796, 5821-5823, 5849, 5979-5980 | NEAR |
| NWS valve module | ? | 1 | ? (not stated in lines cited) | Center hydraulic (A32:5780-5781) | From NWS REU (A32:5796-5798) | Directs center hydraulic pressure to NWS actuators; contains EHSV, SOPV, bypass relief/safety valve, DLD, pressure compensator, tow switches | ata32.txt:6082 | QUOTED |
| NWS actuator | ? | 2 | NLG steering collar (A32:5781, 5800) | Center hydraulic (A32:5780-5781) | Via commutator valves (A32:5784, 5798-5800) | Turns nose wheels up to +/-70 deg (A32:6017); tiller max 70 deg at low speed, 8 deg at 100 kn (A32:5890-5893) | A32:5780-5800, 5890-5893, 6017 | NEAR |
| Brake system control unit | BSCU | 2 (left, right), 2 channels each (inboard, outboard) | ? | 28 VDC (from EBPSU) | CAN bus; CDN via RDC | Pedal, parking, antiskid, autobrake and RTO control; commands EBACs | STM:18670-18685, 18798-18802; A32:6459-6499 | OK |
| Electric brake actuator controller | EBAC | 4 (1 per BSCU channel); each controls 8 EBAs (one forward/aft wheel pair) | Aft EE compartment (A32:6622, 6630-6631); locator page LOC:4806-4813 | +/-130 VDC from EBPSU (A32:6525) | CAN bus from BSCU (A32:6486-6487, 6562-6563); separate CAN to CCS via RDC (A32:6565) | Drives EBAs (A32:6535-6537) | A32:6486-6490, 6525, 6535-6537, 6562-6565, 6622, 6630-6631; LOC:4806-4813 | NEAR |
| Electric brake power supply unit | EBPSU | 4 (1 per BSCU channel) | ? | 28 VDC from TRU and hot battery bus (A32:6820-6821) | Power enable from BSCU (A32:6519-6523) | +/-130 VDC to EBACs; 28 VDC to BSCU (A32:6525-6527) | A32:6519-6527, 6814-6821 | OK |
| Electric brake actuator | EBA | 32 (4 per MLG wheel; 8 MLG wheels) | Each MLG brake | +/-130 VDC via EBAC (power only when changing position) | EBAC (CAN/motor control) | DC motor drives carbon disk clamp | STM:18676-18684, 18718-18719; A32:6465, 6536 | OK |
| Brake temperature sensor (thermocouple) | ? | 8 (1 per MLG brake) | Each MLG brake | ? | To ARDC | Brake temperature measurement | A32:6466, 6496-6497; STM:18473 | OK |
| Axle remote data concentrator | ARDC | 10 (8 MLG, 2 NLG) | Axle ends | 40 VDC (MLG, from BSCU); 28 VDC (NLG, from right BSCU) | CAN bus; CDN; wireless RF to TPS | Wheel speed, brake temp, tire pressure data; antiskid input | A32:6486-6488, 6526-6530, 6325-6330; STM:18662-18667, 18891-18896 | OK |
| Tire pressure sensor | TPS | 10 (1 per wheel) | Inside each wheel (A32:6325-6326, 6398-6399) | Inductive power from ARDC (A32:6330-6332, 6403-6404) | Wireless RF to ARDC (A32:6327-6328, 6401-6402) | Tire pressure and temperature (pressure corrected for ambient temperature) | A32:6325-6340, 6398-6413 | NEAR |
| Magnet ring (wheel speed target) | ? | 8 MLG (NLG count ?) | Axle flange of wheel (A32:6434) | Not applicable | Sensed by MLG ARDC (A32:6432-6433, 6540-6541) | Wheel speed (A32:6402-6403) | A32:6402-6403, 6432-6434, 6462, 6540-6541 | OK |
| Parking brake lever | ? | 1 | P9 forward aisle stand | ? | Discrete to BSCU | Sets and releases parking brake | STM:18530-18534 | OK |
| Autobrake selector switch | ? | 1 | P2 panel | ? | Discrete to left BSCU outboard channel | Landing autobrake level or RTO | STM:18547-18552; A32:6502-6504 | OK |

---

### 3. Surfaces and actuators

| Control surface | No. of actuators | Actuator type | Hydraulic system or power source | Controlling electronics | Source | Check |
|---|---|---|---|---|---|---|
| Aileron (L and R) | 2 PCU per aileron (4 total) | Hydraulic PCU with LVDT | Left wing aileron: L system (STM:17173-17174). Right wing aileron: R system (STM:17184-17185). RAT backup lists both ailerons (STM:17190-17193). A27:2472 says three 5,000 psi systems power the aileron actuation systems; per-PCU assignment not stated. | FCE -> REU (1 per PCU) -> PCU; ACE; FCM | A27:1848, 1883-1884, 2470-2472; STM:17173-17174, 17184-17185, 17190-17193; LOC:2857 | OK |
| Flaperon (L and R) | 2 PCU per flaperon (4 total) plus flaperon manifold (4) | Hydraulic PCU, electric control; manifold with REU | LIB and ROB PCUs: L system. LOB PCU: R system. RIB PCU: C system (A27). STM text conflicts (see Gaps). | FCE -> REU (8 total, 1 per PCU and manifold) | A27:2826, 2865-2871, 1883-1889; LOC:2845-2857; STM:17177-17178, 17190 | OK |
| Spoiler, hydraulic (10): inboard 6-9; outboard 1-3, 12-14 | 10 PCU (inboard 4, outboard 6) | Hydraulic PCU with LVDT, check valve | L: spoilers 3, 12. R: spoilers 2, 6, 9, 13. C: spoilers 1, 7, 8, 14. | SREU (10) -> PCU; FCE / ACE | A27:3020-3036, 3204-3249; LOC:2900-2912; STM:17173-17195 | OK |
| Spoiler, electric (4): spoilers 4, 5, 10, 11 | 4 EMA, each with own EMCU | Electro-mechanical actuator (EMA) | 235 VAC, 3-phase variable frequency to EMCU; 650 V DC motor in EMA | EMCU (4) from FCE; spoilers 4 and 11 alternate position from control wheel position sensor; spoilers 5 and 10 alternate from flap position sensor (droop) | A27:3412-3413, 3421-3426, 3451-3455, 3535-3536 | OK |
| Elevator (L and R) | 2 PCU per elevator (4 total) | Hydraulic PCU with LVDT | STM: left elevator on L system, right elevator on R system. A27 lists L, C and R systems as interfaces. Per-PCU pairing ? | 4 REU (1 per PCU); FCE; ACE | A27:4871-4890, 4983-4985, 5333-5345; STM:17175, 17188-17189 | OK |
| Rudder (1) | 3 PCU (3 REU) | Hydraulic PCU; trim actuator separate (electric) | Three different 5000 psi systems: L (left rudder), C (center rudder), R (right rudder) | 3 REU; FCE; ACE | A27:4015-4016, 4595; STM:17176, 17194 | OK |
| Rudder trim actuator | 1 | Electric (trim) | ? | ACE / FCM | A27:4369-4386 | OK |
| Horizontal stabilizer | 1 HSTA (2 DC motors, 2 secondary brakes, 1 primary brake) | Electric ballscrew | 235 VAC to HSTA EMCUs (L2 from L2 bus, R2 from R2 bus); DC motors driven by EMCUs | 2 EMCU (L2 and R2); FCE; alt trim direct to EMCU | A27:5770-5771, 6026-6041, 6106-6108, 6128-6133, 6389; LOC:3108-3124 | NEAR |
| Leading edge slats (12; 2 inboard, 10 outboard) | 12 slats, 24 GRA | Geared rotary actuators driven by torque tubes from PDU | Hydraulic (primary; center hydraulic via HCM) or electric (secondary; 270 VDC PDU motor from slat EMCU) | FCE (primary and secondary); slat EMCU | A27:7148-7149, 7226, 7233, 7238, 7161-7166, 7360-7362, 8284-8286 | OK |
| Krueger flaps (2) | 2 Krueger flap GRA (A27:7240) | Geared rotary actuator | ? | FCE | A27:7152, 7239-7241 | NEAR |
| Trailing edge flaps (4: 1 inboard + 1 outboard per wing) | 8 flap GRA (4 per wing) | Geared rotary actuator driven by torque tube | Hydraulic (primary, center hydraulic via HCM) or electric motor (270 VDC via TE flap EMCU), through TE flap PDU | FCE via HCM / EMCU | A27:7274, 7287-7288, 7360-7362, 8286-8290 | NEAR |
| Nose wheel steering | 2 NWS actuators | Hydraulic actuators with commutator valves | Center hydraulic system (A32:5780-5781, 5809) | NWS REU via CDN; EHSV and SOPV | A32:5780-5800, 5809, 5821-5823, 5979-5980, 6017 | NEAR |
| Main landing gear (retract/extend) | 2 retract, 2 uplock, 2 downlock, 2 truck positioner, 2 body door actuators (per diagram labels) | Hydraulic | Center hydraulic system | LGA/NWS (CCS) via RDC and solenoid valves | STM:17871-17873, 17940-18008; A32:1466-1481 | OK |
| Nose landing gear (retract/extend) | 1 retract, 1 lock link, 1 forward door actuator | Hydraulic | Center hydraulic system | LGA/NWS (CCS) via RDC and solenoid valves | STM:18164-18351 (diagram labels); A32:1452-1500 | OK |
| Wheel brakes (8 MLG wheels, 4 per gear: 2 inboard, 2 outboard) | 32 EBA (4 per MLG wheel) | Electric brake actuator (DC motor) | +/-130 VDC via EBPSU and EBAC | EBAC (4) from BSCU (2) channels; BSCU via CAN | STM:18676-18684, 18795-18797; A32:6459-6537 | OK |

---

### 4. Gaps and conflicts

Gaps (not in sources):
- FCOM chapters 9 (Flight Controls), 13 (Hydraulics) and 14 (Landing Gear) contain revision highlights, procedures and supplementary procedures only in this OCR. No system-description text for these chapters was found. Only FCOM facts recorded: autobrake deceleration table (p.311, SP.14.1), go-around inhibits ground spoilers and autobrake (p.300), Hydraulic Systems 2 and Primary Flight Controls Normal Mode for CAT III (p.296, SP.13.2), flight control guard below 2500 ft (p.108), hydraulic quantity check (p.115), cold-weather EMP selector preflight (p.316).
- Bus protocols not stated for: ACE/FCM intermodule bus, FCE-to-REU/SREU/EMCU digital command, EMCU/REU feedback, SREU power, and ACE and REU power.
- Locations not stated for: spoiler EMCUs (4), BSCU, EBPSU, LGA/NWS CCS hardware location, RPDU and RDC power source, most TE and LE PDUs (counts), Krueger PDU/GRA source. (SREU and EMA locations are now stated in A27:3207, 3370, 3412-3413.)
- Pump motor controller location and part, and the CMSC acronym expansion are not defined in the sources. A29 uses CMSC without expansion.
- Direct-mode rate sensor (DMRS) data protocol not stated.
- Brake pedal, parking brake and autobrake wiring types beyond "electrical"/"discrete" not stated.
- Magnet ring count on NLG not stated.
- Center and right elevator PCU pairing not stated.
- Gear control proximity sensor count on indication set not stated (A32 says "separate" only).
- Stabilizer position module (3) power/bus not stated.
- Brake temperature and tire pressure data rates not stated.
- Flap skew sensor count and slat skew sensor count not stated in the lines reviewed.
- Hydraulic fluid volumes are in conflict between sources; see Conflicts below.

Conflicts between sources:
- RAT full-power time (hydraulic and electrical):
  - STM:17743-17745: "The RAT deploys and supplies full rated hydraulic and electrical power in 10 seconds."
  - A29:3366-3367: "The RAT deploys and supplies full rated hydraulic power in 6 seconds maximum and electrical power in 10 seconds maximum from the command."
  - A29:3569: "...supplies full rated hydraulic power in 6.35 seconds maximum and electrical power in 10 seconds maximum from deployment command."
  - Electrical 10 s agrees across sources. Hydraulic time is 10 s (STM) vs 6 s (A29:3366) or 6.35 s (A29:3569). Not resolved; all three kept.
- Flaperon hydraulic source:
  - STM:17177 (left system list): "Left & right wing flaperons"; STM:17178 (center list): "Right flaperon"; STM:17190 (right system list): "Left wing flaperon".
  - A27:2865-2871: "the Control Modules on the LOB actuator powered by the Right hydraulic system and the RIB actuator powered by the Center hydraulic system"; "The flaperon LIB and ROB Control Modules powered by the Left hydraulic system remain in Active mode."
  - A27 used in the table (LIB and ROB: L; LOB: R; RIB: C). STM column text is OCR-interleaved and does not map one flaperon per system cleanly.
- FCE R location:
  - A27:708: "FCE right is in the aft equipment center and has these components:"
  - A27:733: "FCE Right is located in the aft EE bay."
  - STM:19240-19244: "There are four flight control electronics (FCE) cabinets. Three are located in the forward electronic equipment (EE) bay and one is located in the aft EE bay." (R not named.)
  - LOC:2742: "FCE cabinet R" (Loc: 1274, 5.5R, 146); no zone name.
  - All sources place R in the aft EE area. "Aft EE center" (A27:708) and "aft EE bay" (A27:733) are wording differences; not resolved further.
- Reservoir/system fluid volumes:
  - STM:17556-17558: "The left and right hydraulic systems contain 20 g (76 l) of fluid each, while the center system contains 45 g (170 l)." ("g" read as gal from OCR.)
  - A29:802-803: "- Left and right reservoir ~ 5 US gallons (19 liters)" and "- Center reservoir ~ 11 US gallons (42 liters)."
  - These are different quantities (system fluid vs reservoir capacity) and the sources do not reconcile them. Both kept; unit reading of STM "g" is an OCR assumption.
- HYDIF host: STM:17577 and STM:17403 say CCS; A29:225, 255 say CCR (CCS also used in A29:448). Treated as the same CCS (common computing resource).
- EMP bus for center: A29:440-441 says C1 from L1, C2 from R1. A29:437-438 says left and right EMPs from R2 and L2 buses, respectively. No conflict, but direction is ambiguous.
- ACE acronym: LOC:32 expands ACE as "Actuator Control Module". A27 and STM use "Actuator Control Electronics". A27 used.
- PCU acronym: LOC:499 expands PCU as "Passenger Control Unit" (OCR or source error). A27 and STM use "Power Control Unit". A27 used.
- Elevator hydraulic source: STM lists only L and R systems for elevator; A27 lists L, C and R interfaces. Left unresolved.
- Path categories requested do not include ARINC 664 (AFDX). A29:448 states HYDIF data flow is per ARINC 664, and LOC:84 says BPCU is on ARINC 664 network. Recorded as stated.
- Spoiler hydraulic sources: STM:17173-17195 and A27:2903-2906 agree on pairs; no conflict.
- Brake count: STM:18720-18721 says ten wheels (2 NLG, 8 MLG); consistent with A32:6459 and LOC.
- EBAC location: A32:6630-6631 says the four EBACs are in the aft EE compartment; the LOC:4806-4813 page is headed "Fwd Main Equipment Center" and gives coordinates for EBAC LI (Loc: 1279, 57L, 141). Recorded as the A32 location; LOC coordinates kept in the Source cell.

Counts: 79 box rows (section 1), 67 data-flow rows (section 2), 15 surface rows (section 3). Some rows group several items, and rows marked "approx" have line ranges that were not verified to the exact line.

## Environmental, ice and rain, fire, cabin, lights, doors, cargo

| Box (full name) | Acronym | Qty | Location | Power source | Network / buses | Function | Source | Check |
|---|---|---|---|---|---|---|---|---|
| Cabin air compressor | CAC | 2 per pack (stated "(2)" per pack); 4 total | Lower left/right fuselage inlet ducts; 2 CACs share one inlet duct | Variable speed AC electric motor (stm:20941); control power from CMSC (stm:20952-20953) | Enable/inhibit commands via CDN from CCS (stm:20939-20940); control commands from PCU (stm:20958-20959) | Compresses air for ACM and trim air | stm:20931-20960, 20941-20945 | NEAR |
| Common motor start controller | CMSC | 1 per CAC (4 total) (ata21:651; ata21:993-994) | ? | +/-270V DC from ATRU via 270V DC bus (stm:20954-20955) | Commands from PCU (stm:20958-20959); PCU to CMSC over CAN (ata21:757); CMSC does BIT (ata21:989) | Controls CAC motor speed to a reference set by PCU (ata21:651-652); protective shutdown (ata21:1005) | stm:20952-20959; ata21:651-652, 989, 993-994 | OK |
| Pack control unit | PCU | 1 per pack (2) (ata21:729) | Pack bays; packs are below center fuel tank (stm:21101-21102) | Each of two channels from a different RPDU (stm:20821-20822; ata21:754) | CAN between the two PCUs (stm:20829-20830; ata21:756-758); CCS data via two different RDCs (ata21:739-740); cooling air from lower recirculation HX (ata21:765-766) | Controls pack components and trim air system (stm:20816-20843, 21105) | stm:20816-20843, 21101-21106; ata21:729-730, 739-758 | OK |
| Air cycle machine | ACM | 1 per pack (stm:21101-21106) | Pack bay | ? (not stated; air-driven T1/T2 turbines) | PCU controls ACM valves (stm:21105-21122) | Cools CAC air via T1/T2 turbines (stm:21106-21122); air bearings, no regular service (stm:20768-20770) | stm:20768-20770, 21101-21122 | OK |
| Ram air inlet door, outlet door, and actuators | ? | Inlet and outlet actuator per pack (stm:20816-20823) | Pack ram air duct | ? | Controlled by PCU (stm:21119-21122; ata21:820-821 is CAC deflector, not ram) | Ram air flow control (pack discharge temp) (stm:21119-21122) | stm:20816-20823, 21119-21122 | OK |
| Ram air fan(s) with motor controllers | ? | ? | Ram air system | ? | Motor controllers; PCU controls ram air fans (ata21:747) | Ram air cooling flow (ata21:747) | ata21:747 | OK |
| CAC inlet deflector door actuator | ? | 1 per CAC inlet (stm:20816-20821) | CAC inlet (stm:20958-20960) | ? | PCU (stm:20816-20821; ata21:820-821) | Protects CAC inlet on ground (stm:20958-20960) | stm:20816-20821, 20958-20960; ata21:820-821 | OK |
| CAC variable diffuser actuator | VDA | 2 per pack (stm:20816-20819) | CAC (stm:20941-20942) | ? | PCU (ata21:963-965; ata21:1056) | Varies CAC diffuser (efficiency) | stm:20816-20819, 20941-20942; ata21:963-965, 1056 | OK |
| CAC add heat valve | AHV | 2 per pack (stm:20816-20820) | CAC discharge (stm:20944, 20952-20953) | ? | PCU (ata21:970-974; ata21:1080) | Opens to raise CAC outlet air temperature (ata21:970-974) | stm:20816-20820, 20944, 20952-20953; ata21:970-974 | OK |
| ACM bypass valve | ABV | 1 per pack | Pack | ? | PCU | Bypasses air around ACM | stm:21105-21116 | OK |
| ACM economy cooling valve | ECV | 1 per pack | Pack | ? | PCU | Economy cooling | stm:21116-21122 | OK |
| Trim air valve / trim air pressure regulating valve | trim valve / PRV | 3 trim valves per pack (flight deck 1, passenger zones 2) (stm:21417-21421, 21435-21436); 1 PRV per pack (stm:21425-21426, 21432-21433) | Pack trim air | ? | PCU controls trim valves; PCU gets temps via CCS (stm:21415-21427) | Adds hot CAC discharge air downstream of compact mixer (stm:21427-21433); PRV keeps trim air above cabin pressure (stm:21425-21426) | stm:21415-21440 | NEAR |
| Lower recirculation fan | ? | 2 (two lower recirculation systems) (stm:21688) | Mix bay area, aft of forward cargo (stm:21710-21712) | ? (not stated) | Fan motor controller (MC); CCS hosted applications control the system (stm:21703-21708); PCU controls fan speed (ata21:745, 753) | Recirculates cabin air; each system has APF, HFSOV, check valve, HX (stm:21688-21700) | stm:21688-21712; ata21:745, 753 | OK |
| Upper recirculation fan | ? | ? | Upper recirculation system | ? | MC under CCS hosted app | Upper cabin recirculation; HEPA filter | stm:21842-21864, 22002-22012 | OK |
| High flow shutoff valve | HFSOV | 1 per lower recirculation system (stm:21692) | Mix bay (stm:21585) | ? (not stated) | CCS hosted control of lower recirculation (stm:21703-21708) | Normally open; closes if a pack-bay duct fails or leaks, to limit cabin depressurization (stm:21704-21708) | stm:21585, 21692, 21704-21708 | NEAR |
| Integrated cooling system heat exchanger/pump | ICS | ? | Mix bay / pack bays | ? | ? | Sends fluid to galley cooling (stm:20757-20759); cold coolant cools lower recirc air (stm:21695-21697; lot_fcom:69406-69408) | stm:20757-20759, 21695-21697 | OK |
| Power electronics cooling system (PECS) HX, liquid TCV, pump packages | PECS | ? (pump packages left and right wheel wells) | HX in LGV path (stm:23263-23265); pump packages in wheel wells (stm:24104-24106) | ? | ? | Cools hot electrical components with fluid (stm:20753-20754) | stm:20753-20754, 23263-23265, 24104-24106 | NEAR |
| Overheat detection system controller | ODS | 1 controller; five dual-element detector loops (stm:21152) | Detector elements in pack bays and NGS compartment (stm:21148-21155) | ? | Alarm data to CCS (stm:21169-21170) | Detects hot-air leaks; sends pack overheat/NGS alarm; CCS shuts pack down (stm:21161-21170) | stm:21146-21170 | OK |
| Valve control unit | VCU | 2 (dual) (stm:24959) | Forward VCU in forward EE compartment; aft VCU in bulk cargo (stm:24963-24965) | ? | Connected to CCS (stm:20803-20805); VCUs share data over isolated data bus (stm:24964-24965); has BITE (stm:20803-20804) | Each controls one outflow valve (stm:20792-20793); each has two control channels (stm:24966-24967) | stm:20788-20805, 24959-24967 | NEAR |
| Outflow valve (forward) | OFV | 1 (stm:24960) | Forward fuselage (stm:24969-24971) | ? | Controlled by forward VCU (stm:24991-24998) | Cabin pressure outflow control; two valves split outflow equally (lot_fcom:69893-69894) | stm:24969-24971, 24991-24998; lot_fcom:69893-69894 | NEAR |
| Outflow valve (aft) | OFV | 1 (stm:24960) | Aft fuselage | ? | Controlled by aft VCU (stm:24997-24998) | Cabin pressure outflow control; LGV exhausts through aft OFV | stm:23238-23239, 24991-24998 | NEAR |
| Independent mechanical safety relief valves | ? | ? (not stated) | Fuselage (not stated in cited lines) | None (mechanical) (stm:20798) | None | Protect structure in any pressure mode (stm:20798-20801) | stm:20798-20801 | NEAR |
| Negative pressure relief valve | ? | 4 (stm:24980-24981) | Not stated in cited lines | None (mechanical, not stated) | None | Negative pressure relief for the cabin | stm:24980-24981 | NEAR |
| Manual backup pressure control | ? | 1 system (stm:20795-20796) | Flight deck; pilots operate manual motor (stm:24998-24999) | Not stated | None stated | Backup cabin pressure control | stm.txt:21077 | QUOTED |
| Alternate ventilation system inlet door and actuator | AVS | 1 inlet door (787-8 on lower left fuselage) | Lower left fuselage (787-8) | ? | CCS controls AVS (stm:22224) | Ventilation when packs off; opens automatically on loss of conditioned inflow | stm:22205-22231 | NEAR |
| Passenger cabin supply check valve | ? | ? | Passenger cabin distribution | None | None | Check valve in AVS path | UNSUPPORTED (AVS detail not re-verified) | NOCITE |
| Lavatory/galley vent fans | LGV | 2 fans (stm:23241-23242) | Aft outflow exhaust path; areas: all lavatories, all galleys, OFCR, OFAR (stm:23238-23244) | Operates automatically with power on airplane (stm:23257-23258) | Hosted application in CCS (stm:23252-23254); temperature/airflow sensors (stm:23251-23256) | Exhausts lavs/galleys; smoke removal from OFCR/OFAR (stm:23246-23248) | stm:23234-23262 | NEAR |
| Lavatory/galley vent heat exchanger | LGV HX | ? | Heat exchanger area | ? | ? | LGV pulls air through additional HX for PECS (stm:23263-23265) | stm:23263-23265, 23355-23362 | NEAR |
| Aft equipment cooling supply fans | ? | 2 (No.1 primary, No.2 backup) (stm:24110-24111, 24121) | Left cheek area of aft cargo (stm:24111-24112) | ? (not stated) | CCS controls aft EE cooling (stm:24104-24108); CPCS and smoke detector data (stm:24112-24117) | Cools aft EE compartment and PECS pump packages (stm:24102-24107) | stm:24098-24121 | NEAR |
| Aft equipment cooling exhaust fan | ? | 1 | Aft cargo area | ? | CCS | Exhausts to aft EE overboard valve or aft cargo heat valve | stm:24100-24120 | OK |
| Aft EE cooling overboard exhaust valve | ? | 1 | Aft | ? | CCS | Overboard exhaust mode | stm:24120-24140 | OK |
| Aft cargo heat valve | ? | 1 | Aft cargo | ? | CCS | Aft cargo heating | stm:24100-24140 | OK |
| Forward equipment cooling exhaust fan | ? | 1 | Forward cargo | ? | CCS | Forward cargo heat air supply | stm:24175-24230 | OK |
| Forward cargo heat supply SOV / exhaust SOV / inline heater | SOV | 1 supply SOV, 1 exhaust SOV, 1 inline heater, 1 exhaust fan (stm:24219-24226) | Forward cargo | Onboard airplane power only; does not operate on external power (stm:24212-24215) | CCS hosted function; P5 cargo temperature panel selector (stm:24212-24216) | Heats forward cargo to 70F (21C) target (stm:24216-24217) | stm:24208-24226 | NEAR |
| Forward cargo air conditioning boost fan / FCAC controller | FCAC | ? (optional system) (stm:20771-20772) | Forward cargo | ? | Controller (stm:24238) | Optional forward cargo cooling (evaporator, boost fan, compressor motor) (stm:24233-24241) | stm:20771-20772, 24233-24241 | NEAR |
| Miscellaneous equipment cooling fan and fan motor controller | ? | 2 filters (crown inlet), 1 fan, 1 exhaust fan | Left side of mix bay | Three-phase 235V AC, variable speed | CCS hosted app; CPCS and smoke data | Crown and IFE cooling (787-8 only) | stm:22860-22900 | OK |
| Miscellaneous equipment cooling smoke detector | ? | 1 | Left side of mix bay | ? | CCS | Smoke/dust detection, low airflow, overheat at 212F (100C) (thresholds not re-read) | stm:22860-22900 (partial) | NEAR |
| Pack/zone trim heater (cabin zone heaters) | ? | ? | Cabin zones; floor panel heaters | Electric (inline and radiant) | CCS; CAP control | Cabin heating | stm:21541-21566 (not re-verified in this pass) | NEAR |
| Engine fire detector loops | ? | 2 loops per engine (stm:25437-25438); 6 detector assemblies per engine, 2 elements each (ata26:240-247) | Engine fan and core sections; inside engine cowling and nacelle (stm:25428-25431) | Engine fire switches powered from hot battery bus (ata26:742); detector loop power not stated | MEDC interface to hosted application in CCS (stm:25433-25434); MEDC to RDC to PFPS function (ata26:204-207) | Overheat and fire detection; AND logic, both loops must sense (stm:25437-25440; ata26:263-265) | stm:25428-25452; ata26:204-207, 240-247, 263-265, 742 | OK |
| Engine fire extinguisher bottle | ? | 2 (fuselage; either engine) (stm:25283-25289) | Left sidewall of forward cargo (stm:25700-25702) | 28V DC to discharge squib (stm:25691-25693) | Squib controlled by CCS hosted applications (stm:25693-25696); squibs monitored by RDC (ata26:494) | Engine fire extinguish; Halon, 2 squibs per bottle (stm:25708; ata26:482) | stm:25283-25293, 25689-25714; ata26:482-494 | OK |
| Cargo fire extinguisher bottle | ? | 2 High-rate discharge (HRD) + 3 low-rate discharge (LRD) bottles in 180-min system; 4th and 5th LRD for 240/330 min (stm:26251-26253, 26273-26276) | Right sidewall of forward cargo (stm:26256-26258) | Squib per bottle (stm:26254-26255) | SMCU operates squibs (stm:26273-26275); CCS hosted apps can auto-discharge (stm:26273-26277) | Halon extinguishing; maintains 5% concentration (stm:26253-26256) | stm:26251-26258, 26273-26277 | NEAR |
| Cargo fire detection system | ? | ? | Forward, aft, bulk cargo (stm:25263-25267) | ? | Detection path not stated; CCS hosted apps trigger discharge (stm:26273-26277) | Detects smoke and fire in cargo | stm:25259-25268, 26273-26277 | OK |
| Squib monitor and control unit | SMCU | 1 (stm:26273-26275) | Cargo fire system (location not stated) | Not stated | Monitors squib condition; operates squibs for bottle discharge and flow valve control (stm:26273-26275) | Cargo fire extinguishing control | stm.txt:26594 | QUOTED |
| Cargo fire flow valves | ? | 2 (one forward cargo, one combined aft and bulk) (stm:26262-26264) | Cargo fire system | Squib in each flow valve (stm:26265-26266) | Controlled by SMCU (stm:26273-26275) | Routes Halon to the selected compartment (stm:26265-26270) | stm:26262-26270 | OK |
| Duct leak and overheat detector system controller | DLODS | 2 detector loops; 1 sensor per loop on fan case near EAI supply duct (stm:25572-25580) | Engine anti-ice supply duct (stm:25556-25560) | ? | Monitored by MEDC (stm:25563-25564) | Detects hot-air leak; MEDC closes EAI valve controller, PRSOV closes (stm:25557-25558, 25565-25566, 25581-25582) | stm:25555-25582 | NEAR |
| APU fire detector loops | ? | 2 loops, 3 detector elements each (stm:25849-25852) | APU compartment | ? | Applications via remote data concentrators (RDC) (stm:25840-25841); CCS sets alarm (stm:25858-25859) | Detects APU fire; APU shuts down automatically (stm:25298-25305, 25857-25860) | stm:25840-25860 | OK |
| APU fire extinguisher bottle | ? | 1 (stm:25309-25311) | Stainless steel bottle in 48 section in tail; forward side of APU compartment bulkhead/firewall (stm:25991-25995) | ? | Discharged by APU fire switch (stm:25989-25990); automatic discharge after 15 s if fire persists (stm:25997-26001) | Extinguishes APU fire | stm:25309-25316, 25989-26001 | NEAR |
| Wheel well fire detection elements | ? | 2 (main wheel wells) (stm:25284-25286) | Main wheel wells | ? (not stated) | ? (not stated) | Overheat and fire detection, thermal inverse resistance (stm:25289-25290); no extinguishers (stm:25293-25294) | stm.txt:25607 | QUOTED |
| Lavatory smoke detector | ? | 1 per lavatory (stm:25305-25306) | Lavatories | ? (not stated) | ? (not stated) | Smoke detection | stm.txt:25627 | QUOTED |
| OFCR/OFAR smoke detector | ? | OFCR: 4 (lot_fcom:66907-66908); OFAR: 8 (lot_fcom:67810-67811). stm:25306-25308 says one above each bunk (conflict noted) | Optional crew rest compartments | ? (not stated) | ? (not stated) | Smoke detection | lot_fcom:66907-66908, 67810-67811; stm:25306-25308 | OK |
| Hand-held fire extinguisher | ? | ? | Passenger cabin, crew rest (stm:25310-25315) | Not stated | None stated | Manual fire fighting | stm.txt:25632 | QUOTED |
| Primary ice detection system probe | PIDS | 2 probes, one each side (ata30:191) | Left and right side of nose (stm:26708-26710); ata30:224 says forward lower fuselage (conflict noted) | 115V AC via a different RPDU per detector (ata30:205; ata30:145-146) | Each detector connected to its own RDC (ata30:201-204); icing data to CCS (stm:26690-26691) | Detects icing conditions; operates from takeoff above 60 kt, not on ground (ata30:206-207, 240-241) | stm:26686-26713; ata30:187-209, 224, 240-241 | OK |
| PIDS heater, wet temp sensor, dry temp sensor | ? | 1 each per probe (stm:26712-26713) | PIDS probe | Probe heater and processor from on-side 115V AC main bus (ata30:145-146, 208-209) | Icing data to CCS (stm:26690-26691) | Heats detector body; dry sensor measures TAT, wet sensor measures evaporation temperature (stm:26686-26700; ata30:194) | stm:26686-26700; ata30:145-146, 194, 208-209 | OK |
| Engine anti-ice (EAI) valve assembly: EAI valve, controller air cooler (CAC), core valve, fan valve, temp sensor, pressure sensors | EAI | 1 set per engine (stm:26834-26840) | Engine nacelle (not stated in cited lines) | Air from 3rd stage HPC bleed air (stm:26817-26818); electrical power not stated | MEDC controls EAI valve (stm:26830-26834) | Hot bleed air heats engine inlet cowl lip (stm:26817-26821); core valve used as PRSOV (stm:26816-26817) | stm:26815-26840; ata30:316-322 | OK |
| EAI valve controller (torque motor) | ? | 1 per engine (ata30:316-322) | Engine (RR Trent 1000 training text) | Torque motor gets power and control from main system (ata30:433-436) | MEDC varying-current signal (ata30:436); MEDC controls power to valve controller (ata30:344) | Controls muscle air to EAI PRSOV (ata30:438-456) | ata30:428-456 | OK |
| EAI pressure sensors | ? | 2 per engine (ata30:596-598) | EAI duct: one on inlet and one on outlet of PRV (ata30:596-598) | ? | Sends duct pressure to MEDC (ata30:336-337; ata30:600) | Feedback for EAI pressure regulation (ata30:591-600) | ata30:568-575, 591-600; stm:26816-26819 | OK |
| Wing ice protection controller | WIPC | 24 control cards (stm:26951-26952) | Wing ice protection controller | ? | CCS (icing data) (stm:26959-26963) | Controls heater mats on 8 slats (4 per wing); each card controls same slat position on both wings (stm:26952-26955) | stm:26944-26963 | NEAR |
| Wing ice protection heater mats | WIPS | 48 mats (3 per slat, 4 slats per wing) (stm:26958-26969) | Bonded to inner surface of slats (stm:26958-26961) | 235V AC from P100, P150, P200 (stm:26961-26963) | Control from WIPC (stm:26951-26952) | Anti-ice, limited anti-ice, de-ice (stm:26953-26970) | stm:26944-26969 | OK |
| Wing slat center heater mat temperature sensor | ? | 1 per slat center mat (stm:26944-26946) | Slats | ? | WIPC | Controls heater power (stm:26946-26951) | stm:26944-26951 | NEAR |
| CAC inlet ice protection heating element and temp sensor | CIPS | 2 CAC inlets | CAC inlet | RPDU (heating element) | CCS | Removes ice at CAC inlet | stm:27036-27057 | OK |
| Air data sensor heater (pitot, AOA, TAT probes) | ADS heat | 3 pitot, 2 AOA, 1 TAT | Nose probes | 115V AC from RPDUs | ADRF operates sensor heat | Prevents ice on air data probes | stm:27146-27169 | OK |
| Flight deck window heater and window heat controller | WHC | 4 flight deck windows; forward windows have primary and backup heat (stm:27263-27265) | Flight deck windows | 115V AC via RPDUs (stm:27267-27268) to SSPCs in two PDPs (stm:27270-27273) | CCS hosted application, automatic; P5 switches de-select/reset (stm:27279-27286) | Ice and fog protection (forward windows, stm:27260-27261); fog only (side windows, stm:27261-27262); backup anti-fog only (lot_fcom:70352-70354) | stm:27260-27286; lot_fcom:70352-70354 | NEAR |
| Solid state power controller (window heat) | SSPC | Count not stated (stm:27270-27273) | Two power distribution panels (PDP) (stm:27272-27273) | 115V AC bus (stm:27270-27272) | CCS controls window heat (stm:27274, 27279-27280) | Switches window heat power | stm:27270-27280 | NEAR |
| Windshield wiper motor | ? | 2 (one per forward window) (stm:27375-27377, 27385-27386) | Flight deck windows (forward) | Electric motor; control power from WECU (stm:27385-27386) | WECUs send wiper data (stm:27390-27392) | Clears windshields (ground only) (stm:27374-27375) | stm.txt:1625 | QUOTED |
| Wiper electronic control unit | WECU | 2 (one per wiper) (stm:27385-27386) | Forward upper EE compartment (stm:27369-27370) | Power from RPDU; voltage not stated (stm:27387-27388) | Wiper control data to CCS through RDCs (stm:27390-27392); data on HDD maintenance pages (stm:27392-27394) | Wiper control | stm:27369-27394 | NEAR |
| Windshield washer pump | ? | 2 (one per window) (stm:27384-27385) | Closet in flight deck, left side behind captain (stm:27381-27383) | 28V DC from RPDU (stm:27388-27389) | Push-button wash switches (stm:27386-27387) | Sprays washer solution | stm.txt:27710 | QUOTED |
| Water and waste systems heater | ? | Eleven heated potable water hoses plus lavatory waste drain fittings (stm:27459-27460, 27457) | Water/waste tanks and lines | 115V AC with electrical power on airplane (stm:27457-27458) | ? | Prevents ice in potable water supply lines and waste (stm:27451-27458) | stm.txt:27801 | QUOTED |
| Flight crew oxygen bottle | ? | 1 gaseous (2 more optional) (stm:27483-27485); 4 masks (stm:27487-27489) | Right side, EE bay (stm:27619-27621) | ? | ? | Flight crew oxygen supply; 4 masks | stm:27481-27492, 27612-27621 | OK |
| Passenger service unit / lavatory service unit (cabin oxygen controller) | PSU/LSU | Service units: PSU, ASU, LSU, OFCR/OFAR (stm:27759-27763); count per seat/lav not stated | Overhead service units | ? | CCS opens service unit doors above 15,000 ft cabin altitude (stm:27775-27778) | Passenger oxygen; controller activates pyrotechnic disc cutter (stm:27768-27770) | stm:27757-27779 | NEAR |
| Cabin crew/walkaround oxygen bottle | ? | Walkaround bottles (count not stated) (stm:27517-27523) | Various locations through passenger cabin (stm:27521-27523) | ? | ? | Emergency crew oxygen (stm:27499-27503, 27525-27526) | stm:27499-27503, 27517-27523 | NEAR |
| Potable water tank | ? | 2 tanks, 2 pumps (stm:27479-27480) | Aft of bulk cargo, outboard of the two waste tanks (stm:27490-27492) | ? | ? | Water to galleys and lavatories (stm:27528-27531) | stm.txt:27824 | QUOTED |
| Vacuum waste tank | ? | 2 (left, right), one tank per system (stm:27496-27498) | Aft of bulk cargo, in center between potable tanks (stm:27501-27504) | ? | ? | Vacuum lavatory waste | stm.txt:21217 | QUOTED |
| Galley waste system | ? | 2 separate systems (stm:27508) | Galleys | ? | ? | Galley waste/gray water (stm:27508-27512) | stm.txt:27841 | QUOTED |
| Lavatory waste system | ? | 2 separate vacuum lavatory waste systems (stm:27516-27518) | Lavatories | ? | ? | Lavatory waste (stm:27514-27520) | stm.txt:27841 | QUOTED |
| Portable oxygen cylinder + SOV + solenoid valve | ? | ? | Passenger cabin | ? | ? | Medical/portable oxygen; electronic portable oxygen pulses a solenoid valve (stm:27804-27806) | stm:27795-27808 | NEAR |
| Passenger electronically dimmable window (EDW) | EDW | ? (passenger windows) | Passenger cabin windows | Power applied changes transmittance; normal power removed returns passenger EDWs to clear (stm:29832-29843) | CSS or switch/controller below window (stm:29840-29843); status at CAP (stm:29829-29830) | Electronic dimming (2 settings for PED, 5 for passenger) (stm:29834-29838) | stm.txt:30210 | QUOTED |
| Cabin services system | CSS | 1 (?) | ? | ? | CCS | Controls EDW operation (stm:29840-29843) | stm:29820-29823, 29840-29843 | NEAR |
| Passenger entry door (PED) | PED | 8 (stm:29649-29650) | Fuselage | ? | EPAS, PSDC proximity sensors (stm:29642-29646) | Passenger entry; slide/raft | stm:29085-29101, 29217-29237, 29649-29650 | OK |
| Emergency power assist system | EPAS | 1 per PED (stm:29651) | Each PED (stm:29651) | ? | EPAS modules send data to CCS (stm:29643-29646) | Assists door open; deploys slide/raft (stm:29233-29237); primary function emergency egress (stm:29652-29653) | stm:29233-29237, 29643-29653 | OK |
| Emergency escape slide/raft | ? | 1 per PED (crew escape area) | Inside PED | Inflation bottle (self-contained) | Girt bar; EPAS armed | Emergency egress | stm:29261-29288 | OK |
| Proximity sensor data concentrator | PSDC | Count not stated; each reads up to 16 proximity sensors (stm:29659-29660) | Not stated in cited lines | ? | Door status data to CCS (stm:29643-29646); proximity sensor system is hosted in CCS (stm:29638-29639) | Door position monitoring | stm.txt:30029 | QUOTED |
| Forward cargo door | ? | 1 | Forward fuselage | Electric-hydraulic power pack; ground power | RDC and RPDU | Opens/closes cargo door | stm:29490-29517 | OK |
| Aft cargo door | ? | 1 | Aft fuselage | Electric-hydraulic power pack; ground power | RDC and RPDU | Opens/closes cargo door | stm:29490-29517 | OK |
| Cargo handling power drive unit | PDU | ? (forward and aft compartments) | Forward and aft cargo | Remote power from RPDUs (stm:29999-30000) | Monitored by CMDU on CAN bus (stm:29992-29993); controlled by CMDU through RDC and RPDUs (stm:29988-29990) | Moves cargo into position (stm:29978-29980) | stm:29978-30000 | NEAR |
| Cargo handling controller (expansion not stated) | CMDU | 1 per compartment (not stated) | Main control panel (MCP) with joystick (stm:29998-29999) | ? | CCS interface (stm:29998-29999); CAN to PDUs (stm:29992-29993) | Acts as controller, interfacing with MCP, joystick, PDUs (stm:29994-29997) | stm:29993-30000 | NEAR |
| Cargo maintenance display unit | ? | 1 | Interior cargo | ? | ? | Cargo drive status | stm:29958-29976 | OK |
| Main engine data concentrator | MEDC | 1 per engine (ata26:349-352 describes left and right channels A and B) | Main engine area (ata26:363-369) | Left channel A from captain instrument bus; left channel B from left main DC bus; right channel A from F/O instrument bus; right channel B from right main DC bus (ata26:349-352) | Collects analog sensor signals, converts to digital, sends on CAN to RDC (ata26:340-342); RDC sends onto CDN (ata26:343-344) | Engine fire/overheat detectors, EAI control, sensor interface (ata26:363-374; stm:26830-26834) | ata26:340-374; stm:26830-26834 | OK |
| Remote data concentrator | RDC | UNSUPPORTED (count not stated) | UNSUPPORTED (location not stated) | UNSUPPORTED (bus not stated) | Puts digital data onto CDN (ata26:343-344); sends digital data to hosted applications in CCS (stm:25247-25248) | Data concentrator | ata26:343-344; stm:25247-25248 | OK |
| Remote power distribution unit | RPDU | UNSUPPORTED (count not re-verified; locator:2329-2413 not re-read) | Not stated in cited lines | Fed from on-side 115V AC main buses (ata30:145-146; ata30:208-209) | CCS causes RPDU to send 115V AC (ata30:202-203) | Distributes power; powers PCU channels (stm:20821-20822), window heat (stm:27267-27268), wiper control (stm:27387-27388), CIPS (stm:27043-27044) | stm:20821-20822, 27043-27044, 27267-27268; ata30:145-146, 202-205 | OK |
| Common core system | CCS | UNSUPPORTED (count not stated) | Core network cabinet (stm:5419-5424) | UNSUPPORTED (not stated) | Core network cabinet: NIM, EGM, CSM, CIS/MS FSM (stm:5462-5468); ODN and IDN (stm:5471-5474) | Hosted application processing (stm:5464-5480) | stm.txt:748 | QUOTED |

### 3. Exterior lights

All are controlled from P5 overhead panel switches (stm:28677-28678, 28680-28698). Power source is not stated in the text read: `?` for all rows.

| Light | Location | Type | Control path | Power | Source | Check |
|---|---|---|---|---|---|---|
| Body anti-collision (beacon), upper and lower | Upper and lower fuselage | LED, red (stm:28699-28700) | P5 BEACON switch (stm:28680-28681) | ? | stm:28680-28681, 28699-28700 | NEAR |
| Wing anti-collision | Wings | LED, white (stm:28675-28676, 28696) | P5 STROBE switch (stm:28683-28685) | ? | stm:28675-28676, 28683-28685 | NEAR |
| Wing forward position (green) and red | Leading edge of wing tips (stm:28678-28679) | LED, red/green (stm:28678-28679) | P5 NAV switch (stm:28686-28687) | ? | stm:28678-28679, 28686-28687 | NEAR |
| Wing rear position | Trailing edge of wing tips (stm:28681-28682) | White; type not stated | P5 NAV switch (stm:28686-28687) | ? | stm.txt:29037 | QUOTED |
| Tail position/anti-collision (2) | Left and right side of tailcone (stm:28684-28686) | LED, white | P5 NAV (position) and STROBE (anti-collision) (stm:28683-28687) | ? | stm:28683-28687 | OK |
| Logo lights (2) | Top of each horizontal stabilizer (stm:28689-28690) | HID, white (stm:28689-28690) | P5 LOGO switch (stm:28690-28691) | ? | stm:28689-28691 | NEAR |
| Wing illumination (2) | Fuselage forward of both wing LEs (stm:28692-28693) | HID, white (stm:28692-28693) | P5 WING switch (stm:28693-28694) | ? | stm:28692-28694 | NEAR |
| Taxi lights (2) | Nose gear (stm:28696-28697) | Type not stated | P5 TAXI switch (stm:28696-28698) | ? | stm:28696-28698 | NEAR |
| Landing lights (2 nose gear, 2 per wing root LE) | Nose gear (2 landing, 2 taxi); 2 per wing root LE (stm:28696-28698) | Type not stated | P5 LEFT, RIGHT, NOSE landing switches (stm:28696-28698) | ? | stm:28696-28698 | NEAR |
| Runway turnoff lights (2 total) | 1 in each wing root LE (stm:28697-28698) | Type not stated | P5 RUNWAY TURNOFF switches (stm:28696-28698) | ? | stm:28696-28698 | NEAR |

### Gaps

- Power sources are `?` for most boxes. Power is named in the text for: CAC motor (AC), CMSC (270V DC via ATRU), PCU channels (RPDU), window heat (115V AC via RPDU to SSPC), WIPS mats (235V AC from P100/P150/P200), PIDS (115V AC via RPDU), engine fire switches (hot battery bus, ata26:742), cargo PDUs (RPDU), exterior lights (not stated in any read line).
- Data network type (CDN/ACS/RDC, ARINC 429, CAN, discrete) is stated for PCU-to-PCU (CAN, stm:20829-20830; ata21:756-758), PCU to CMSC/recirculation fans (CAN, ata21:757), MEDC to RDC (CAN) and RDC to CDN (ata26:341-344), CMDU to PDUs (CAN, stm:29992-29993), and the CCS/ODN/IDN architecture (not re-verified).
- Source conflicts noted, not resolved: PIDS location (nose per stm:26708-26710 vs forward lower fuselage per ata30:224); OFCR/OFAR smoke detector count (stm:25306-25308 "one above each bunk" vs lot_fcom:66907-67811 counts); ata30 EAI text is for an RR Trent 1000 engine variant.
- Rows kept with their original citation and NOT re-verified in this pass (no text re-read): cabin zone heaters (stm:21541-21566), cargo door open toggle (stm:29495-29500), cargo fire detection path, misc equipment smoke thresholds (stm:22860-22900), AVS inlet door details (stm:22205-22231), and the cabin light control path.
- Not read in this pass: lot_fcom chapters 1 (Airplane General, Doors, Windows, Lighting), 2 (Air Systems), 3 (Anti-Ice, Rain), 8 (Fire Protection) beyond the grep hits cited above; ata21 beyond the grep hits; ata26 beyond the fire-detection and squib hits; ata30 window heat, wiper and WIPS sections (not found by grep); locator.txt install locations (RPDU counts, PCU/CAC/ODS/OFV/LGV locations).
- stm ch18-24 partly read; ACM/ram-door counts, forward equipment cooling flow paths, FCAC controller detail, and EDW controller count remain unconfirmed.
- OCR quality: multi-column layout garbles several passages. Citations are to lines as read.
