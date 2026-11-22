# Shared: computer (LRU) model

Every box with electronics on the aircraft is built on this model. Each system plan covers what a box does; this plan covers the box itself.
Status: draft. **(verify)** means a source document is needed.
The inventory of every box is in [computers.md](computers.md).

## Hardware, per box
- **Power inputs:**
  - Each input's bus and voltage range. Dual inputs are diode-OR'd.
  - An internal PSU whose hold-up time comes from its capacitor energy and load.
  - Brownout threshold, inrush current, and power draw by mode (boot, run, heater).
- **Core:** processors, RAM with ECC, NVM, a crystal oscillator (drift depends on temperature and age), a watchdog.
- **I/O:**
  - CDN end system (A and B ports).
  - ARINC 429 transmit and receive channels.
  - CAN.
  - Discretes.
  - Analog, through ADCs and DACs with their own resolution and noise.
  - Pin-programming straps that set identity and position.
- **Connectors:** every pin is wired to a real wire (wiring plan).
- **Heat:**
  - Dissipation by mode goes into its zone.
  - Cooling dependency: forced air from equipment cooling, or a liquid cold plate for power electronics.
  - Internal temperature, derating, overtemperature shutdown, and a minimum temperature for start.
- **Identity:** part number, serial number, location (rack and zone).

## Software behaviour, per box
- **Boot sequence:**
  1. Power good.
  2. POST: RAM test and load CRC.
  3. Pin-programming and configuration check.
  4. Software part number checked against its ICD.
  5. Init, then operational.

  Durations per box **(verify)**.
- **Modes:** off, booting, operational, degraded, failed, BITE/maintenance, data load.
- **Scheduling:**
  - Rate groups or partitions in a major frame.
  - An overrun goes to the watchdog or health monitor, which restarts the partition or resets the box.
- **Software load:** part number, ICD version and configuration tables, loaded through the data loader with a real load time. A mismatch is a fault or a no-go.
- **BITE:**
  - Power-up, continuous, and initiated from CMCF.
  - Produces fault codes, which become maintenance messages.
  - Intermittent-fault counters.
- **NVM, kept between sessions:**
  - fault history;
  - hour and cycle counters;
  - the real counters (GCU 248-day, CCS 51-day);
  - configuration;
  - last state, such as the ADIRU's last position.
- **Time:** the box's own clock runs from its oscillator and is synced from GPS or CCS time. Between syncs it drifts.

## Environment effects
- **Single-event upsets (SEU).**
  - Cosmic-ray neutrons flip bits.
  - The rate rises with altitude and latitude and falls with solar activity (public atmospheric neutron flux models).
  - ECC fixes single bits. An uncorrectable error resets the box or corrupts its data.
  - Rare, but real.
- **Power quality:** dips, gaps and frequency changes. A gap longer than the hold-up time causes a reboot.
- **Lightning and EMI:** transients cause bit errors or resets (environment plan).
- **Vibration:** connector fretting causes intermittent faults.
- **Water in the E/E bay** (AD 2024-01-04, AD 2025-09-12): a leak path drips onto specific boxes, which short or fault.

## Failures (0–1)
- PSU: hold-up loss, total loss.
- Processor.
- RAM: stuck bits.
- NVM corruption.
- Oscillator drift or failure.
- Watchdog stuck.
- Each I/O channel.
- Each connector pin: open, short, intermittent.
- Cooling fan.
- Software load: corrupt or wrong version.
- Pin-programming error.

## Wear
- **Electrolytic capacitors age** (their life halves for every 10 °C hotter), so hold-up time shrinks. An old airframe reboots on power transfers that a new one rides through.
- Fan bearings, connector mating cycles, NVM write cycles, oscillator aging.

## Airframe differences and maintenance
- **Every box is a serialised unit** with its own tolerances: oscillator ppm, hold-up, ADC offsets.
- **Boxes can be swapped** through maintenance pages. A swapped-in box brings its own history and its own software load. Swapping needs a data load and a post-install test.

## Inspector (per box)
Power, temperatures, mode, boot timeline, software and ICD versions, partition loads, BITE history, counters.

## Tests
1. A power-transfer gap: a box with new capacitors rides through, one with aged capacitors reboots.
2. Boot times match video, per box.
3. A watchdog miss resets the box, it recovers, and a maintenance message is logged.
4. SEU rate at FL410 at high latitude is far above the rate at sea level, and matches the flux model.
5. A water leak path faults exactly the boxes beneath it.
6. A wrong software load raises a configuration mismatch, and that function is no-go.
7. Loss of equipment cooling: boxes overheat in order of thermal margin, then shut down.
8. Cold-soaked boot: boxes below their minimum temperature start slowly or wait for their heaters.

## Documents
- Per-box details are in [computers.md](computers.md) and its sources.
- JEDEC JESD89 summary (neutron flux).
- AMM data-load tasks.
- The two E/E bay water ADs (have).
