# 42 — Common Core System (CCS) and Common Data Network (CDN)

Status: draft. Sources: System Technical Manual ch. 5 (STM), ATA 46, AD 2020-06-14. **(verify)** means not confirmed yet.

## Hardware (STM ch. 5)
- **2 CCR cabinets (L, R).** Each has:
  - 8 GPMs (L1–L8, R1–R8);
  - 2 PCMs;
  - 2 FOX fibre translators;
  - 2 ACS switches (one channel A, one channel B);
  - 2 graphics generators (displays plan).
- **PCMs:** turn 28 VDC into 12.5 VDC. Each is fed from one of the four DC buses plus the hot battery bus. One PCM can power the whole cabinet.
- **ACS ports:** 20 copper at 100 Mbps inside the cabinet, 4 copper at 10 Mbps to external systems.
- **6 ARS remote switches:** 3 channel A, 3 channel B; 2 forward, 4 aft.
- **21 RDCs:** each has one channel A and one channel B port. They bridge ARINC 429, analog, discretes and CAN onto the CDN. Pin programming sets each one's position.
- **Links:** fibre at 100 Mbps; copper at 100 Mbps inside cabinets and 10 Mbps outside.
- **What connects where:**
  - Through the ACSs: P300/P400 panels, flight control electronics (FCE) cabinets, RPDUs.
  - Through the FOXs: AGUs, FCMs, recorders, ACPs, EECs, core network, cabin services, ISS.
- **CCR reset switches:** one per cabinet, flight deck.
- Every module, switch, port, cable and connector is an entity with temperature, wear, tolerance, failures and zone. Positions come from the B787 Locator.

## Network: ARINC 664 Part 7, simulated frame by frame
- **Virtual links (VLs).**
  - Each data flow is one VL: one source, one or more destinations, routed statically through switch tables.
  - Each VL has a BAG (1–128 ms), a maximum frame size and a priority.
  - The real VL table isn't public. Ours is generated from who actually sends what to whom in our aircraft, and marked `APPROX`.
- **Frames.**
  - Real framing sizes, a sequence number per VL, and a CRC.
  - The payload is real packed bytes: parameters plus a status for each.
  - Consumers decode the bytes, so a corrupted bit is a real corrupted bit.
- **Dual network.** Every frame goes out on both A and B. The receiver keeps the first valid copy, checks the sequence numbers, and enforces a maximum skew between A and B.
- **Switches.**
  - Store-and-forward: each hop takes frame bits ÷ link rate, plus the switch latency.
  - Output-port queues with 2 priorities.
  - Policing per VL (BAG, jitter, size). Frames that break the rules are dropped and counted, so one LRU spamming the network can't take the rest down.
  - Frames from unknown VLs are dropped.
- **End systems.**
  - **Transmit:** shaped to the BAG, with real jitter.
  - **Receive:** sampling ports keep the latest value plus its age; queuing ports are FIFOs that can overflow.
- **Emergent behaviour:** latency, jitter, queue build-up, drops, stale data and A/B skew all come out of the frame simulation.
- **Cost:** tens of thousands of frames per second, handled as events within each tick. Target under 1 ms per tick.

## Full data movement: no shortcuts
- **The only way data crosses between systems is the way it crosses on the real aircraft:** a CDN frame, an ARINC 429 word, a CAN frame, or a discrete or analog wire.
- **Example path:** sensor → wire → RDC → ARS → FOX → ACS → GPM partition → ACS → graphics generator → display. Every hop has real latency, so a dead switch shows up as a frozen value, then an aged-out flag.
- **ARINC 429:** words carry label, SDI, data, SSM and parity, at 12.5 or 100 kbit/s, with real word rates.
- **CAN:** frames with arbitration, error frames, and bus-off.
- **Discretes and analog:**
  - Voltage depends on wire resistance (wiring plan).
  - The RDC's ADC has its own resolution and noise.
  - An open wire reads as an open wire.
- **Enforced in code:**
  - Avionics-function crates can't see physical model state. They only get their ports.
  - An audit test fails the build if any consumer reads anything that isn't a port.
- **Not simulated:** bits on the wire (PHY signalling, fibre light). Bit errors come from each link's optical or electrical margin as a bit error rate, which has the same visible effect.
- **Message formats:** the real formats (ICDs) are proprietary. Where documents give rates and formats we use them; the rest are engineered and marked `APPROX`.

## Packing: real bytes, not variables
- **Senders encode, receivers decode.** Every value is encoded into real bits and placed in the frame or word, and receivers decode it. Nothing is passed as a plain value.
- **Encodings:**
  - BNR: two's complement, with a range and resolution per parameter.
  - BCD.
  - Discrete bit fields.
  - Floats where the format uses them.
  - Fixed byte order.
- **Status:** a validity status travels with every parameter. ARINC 429 uses its SSM. On the CDN we use a functional status per data set, the common ARINC 664 pattern, because Boeing's exact layout isn't public **(verify)**.
- **What this produces:**
  - Quantisation to each parameter's resolution.
  - Saturation at its range limit.
  - A bit flip that slips past the CRC gives a plausible-but-wrong value, such as a jump when the top bit flips.
  - Stale or invalid status is handled the way the receiver really handles it.
- **One ICD file is the source of truth.** It lists every message, parameter, encoding, offset, rate and VL, and the encoder and decoder code are generated from it, so senders and receivers agree.
- **ICD versions per box.** Each box's software load carries its own ICD version. A wrong load on one box is a real mismatch: it decodes the right bytes into the wrong values. That's a failure mode.
- **The inspector decodes the bytes for you,** so you can read any frame as values or as raw hex.

## Core network (ATA 46): a separate IP network
- This is the IT side: EFB, cabin systems, wireless and Gatelink, maintenance laptop.
- It uses real IP/UDP/TCP with routing and TTL, the style of your screenshot, behind its own gateway or firewall to the CDN **(verify: ATA 46)**.

## GPMs and hosted applications
- Every GPM has the same hardware and core OS.
- **Partitioning:** applications run in time-and-space partitions, each with a fixed window inside a major frame.
- **Hosting:** which applications run on which GPM comes from the STM hosted-application table (L1–R8) **(verify: read the table page as an image)**. Hosted functions include CAC temperature control, equipment cooling, CBIC, electrical distribution, fire protection, fuel quantity, hydraulic control, window heat, DCAS, gear indication, communication management and more.
- **Overruns:** an application that overruns its window becomes a partition fault. The health monitor restarts that partition.
- **Boot sequence:** PCM power → GPM self-test → core OS → configuration → partitions start. Times come from video **(verify)**.
- **GPM loss:** its applications are lost, or the redundant instance on the other cabinet carries on **(verify each function from the table)**.

## Message age monitoring and the 51-day bug (AD 2020-06-14)
- **How it works:** the CCS checks every message's age, so stale data is flagged invalid.
- **The bug:** after 51 days of continuous power the stale-data monitor is lost.
- **The danger:** combined with a CDN switch failure, stale flight-critical data (airspeed, altitude, attitude, engine) is routed and shown as valid.
- **How we model it:**
  - The monitor's counter persists across sessions.
  - One switch failure mode replays or holds old frames.
  - Normally the age check catches it. Past 51 days it doesn't.

## RDCs
- Each RDC converts its pins and buses to and from VLs.
- Every sensor and discrete is wired to a specific RDC **(verify mapping: STM / ATA books)**.
- **RDC loss:** all of its inputs go invalid in every consumer.
- **Pin programming:** a wrong or open pin gives a wrong position, so the RDC gets the wrong configuration.
- **Software loadable:** the loaded software version is part of airframe state.

## Links
- **Takes:**
  - Electrical: PCMs from the DC buses and hot battery; ARS and RDC power **(verify)**.
  - Equipment cooling (air conditioning plan): cabinet temperature.
- **Gives:** every system's data path. Displays read only from here.
- **Heat:** each module dissipates into the forward E/E bay zone.

## Failures (0–1)
- **GPM:** processor, memory, partition crash, health-monitor fault.
- **PCM:** degraded output, total loss.
- **ACS/ARS:** a port, the whole switch, a corrupted config table, replaying or stuck queues.
- **FOX:** optical power loss.
- **Fibre and copper:** open, intermittent (bit errors), dirty or bent fibre (attenuation), connector corrosion.
- **RDC:** a channel, one I/O type, pin programming.
- **End system:** babbling (spamming), stuck sequence numbers.
- **Configuration:** wrong software load.

## Temperature, wear and airframe differences
- **Temperature:** modules and switches have real temperatures. Overtemperature leads to faults **(verify thresholds)**.
- **Wear:** connectors and fibre lose margin with age and cycles.
- **Airframe tolerances:** each airframe has its own oscillator drift and link margins.

## Network inspector (EFB / maintenance page)
- **Frame log:** time, VL, source → destination, the hop it's on, channel A/B, sequence number, size and decoded payload. Filter by VL, LRU or parameter.
- **Per-VL statistics:** rate, latency, jitter, drops, A/B skew, payload age.
- **Per-switch ports:** frames, CRC errors, policing drops, queue depth.
- **Per-GPM:** partition load, overruns, health events.
- **Live topology:** link and switch states.
- **Real 787 CMCF pages:** CCS L/R CCR, ACS A/B, PCM and GPM status, following the STM screens **(verify layouts)**.

## Tests
1. Cold power-up: PCMs → GPM boot → applications come up in order → displays live. Timings match video.
2. Channel A switch fails: everything continues on B, and a maintenance message appears.
3. Intermittent fibre: CRC errors climb, A/B redundancy hides it, and counters show it.
4. A babbling RDC: policing drops its excess, and other VLs are unaffected.
5. Queue overflow: drops leave stale sampling ports, and consumers flag the data invalid.
6. GPM loss: exactly its hosted functions degrade.
7. One PCM lost: the cabinet stays up. Both lost: the cabinet goes down.
8. 51 days powered, plus a replaying switch: stale data is displayed as valid.
9. CCR reset: the cabinet reboots and functions recover after their boot time.
10. RDC loss: its sensors go invalid everywhere.
11. Every VL stays inside its latency bound under normal load.

## Documents
- STM ch. 5 (have).
- ATA 46 (have): the core network, a separate IT network.
- ATA 31 (have): DCAS.
- AMM 42-21-02 (have): GPM swap.
- AD 2020-06-14 (have).
- B787 Locator (have): positions.
- MFD.pdf (have): CMCF menu.
- ARINC 664 Part 7 is paid; public summaries are enough.
