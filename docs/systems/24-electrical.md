# 24 — Electrical, including every circuit breaker

Status: first pass. It gets the same full treatment as [34-adirs.md](34-adirs.md) when we come back to it. Anything marked **(verify)** needs a source document.

## Hardware

- **Generators.**
  - 4 engine starter-generators (VFSGs), two per engine: 250 kVA, 235 VAC, frequency varies with engine speed.
  - 2 APU starter-generators at 225 kVA.
  - RAT generator.
- **External power.** 3 sockets, 2 forward and 1 aft, at 115 VAC (90 kVA each **(verify)**).
- **Converters.**
  - ATRUs: 235 VAC to ±270 VDC for large motors.
  - ATUs: 235 VAC to 115 VAC.
  - TRUs: to 28 VDC.
  - Counts **(verify)**.
- **Batteries.** Main and APU lithium-ion batteries, 8 cells each. Each has a battery monitoring unit, a charger and contactors.
- **Distribution.**
  - Primary power panels in the aft equipment bay and secondary panels in the forward bay.
  - About 17 remote power distribution units (RPDUs) **(verify)** containing the electronic breakers (SSPCs).
  - The remaining thermal breakers.
- **Motor controllers (CMSCs).** They drive the starter-generators for engine start, then the cabin compressors and other large motors. Which motor uses which controller **(verify)**.
- **Control computers.** A generator control unit (GCU) for each generator, and bus power control units (BPCUs).

## Every circuit breaker

- **One entity per breaker.** Every electronic breaker channel and every thermal breaker on the 787 is its own entity. Each one records:
  - its real name;
  - its bus and panel, or its RPDU;
  - its rating and trip curve;
  - the wire it protects (gauge, length and zone);
  - the load or loads it feeds.
- **Every breaker feeds a real load.** Pulling or tripping a breaker removes power from exactly that load, and the function behind it degrades by itself. For example, pull a probe-heat breaker in icing and the pitot ices up, which leads to unreliable airspeed.
- **Electronic breakers (SSPCs)** have:
  - an I²t trip curve;
  - an instantaneous trip;
  - arc-fault detection;
  - commanded on/off for load shedding, and from the CBIC page **(verify which ones pilots can command)**;
  - status reported to the CBIC page.
- **Thermal breakers** trip on heat built up from current, depending on the surrounding temperature (a cold breaker trips later). They can be pulled and reset. They get 2D panels until the 3D cockpit exists.
- **Audits.** The consumer audit fails the build if any breaker has no load, or if pulling it changes nothing observable. "Observable" includes maintenance messages.
- **Data is the limit.** The full list of breakers with names, ratings, buses and loads comes from the wiring diagram manual or SSPC configuration list, plus the FCOM's CBIC description. We model every breaker on the list and invent none. Until the list is complete, we model the breakers we have documented.
- **Performance is not a concern.** Thousands of breaker entities stored as flat arrays cost microseconds per tick.

## Physics

- **Solving the network.**
  - Contactor positions split the network into islands each tick. Engine generators can't run in parallel, so each AC island has one source.
  - Each island's voltage is solved from the source's internal impedance against all its loads.
  - The DC side is solved as a full network with batteries and converters in parallel.
- **Load types.**
  - Constant power: motor controllers and converters, which draw more current at lower voltage.
  - Constant resistance: heaters.
  - Motors: inrush current when starting.

  Loads react to the voltage they actually get: they drop out below a threshold, and a motor can stall.
- **Generators.** Frequency follows gearbox speed. The GCU regulates voltage with a realistic response time. Overload builds heat and limits current.
- **Converters.** Power in equals power out divided by efficiency. The losses become heat in the bay's thermal zone.
- **Batteries.** Modelled per cell:
  - Resting voltage depends on charge and temperature. Internal resistance depends on temperature, charge and age.
  - Heat from charge and discharge, and a temperature for each cell coupled to its neighbours and the case.
  - Capacity fade with age, and self-discharge.
- **Transfers.** Contactors take real time to operate. During a power transfer that briefly cuts power, loads ride through on their hold-up time or reboot.

## Computers and software behaviour

- **GCU:** over- and under-voltage, over- and under-frequency, overcurrent and feeder-fault protection. It trips the generator breaker.
- **BPCU:** bus ties, source priority, and load shedding from the real measured load against available capacity.
- **BMU and charger:** cell monitoring. The charger refuses to charge outside its temperature limits.
- **GCU 248-day counter.** It accumulates powered time across sessions, and the generator fails safe when the counter overflows.

## Links to other systems

- **Supplies:** power to every system.
- **Takes:** engine and APU gearbox speed.
- **Engine start:** the generator acting as a starter is solved together with the engine spool.
- **RAT:** supplies wind-driven power.
- **Heat:** converter and wire losses go into the thermal zones.

## Indications

All through sensors:
- the ELEC synoptic page;
- EICAS messages (names from FCOM);
- the CBIC page;
- status and maintenance messages.

## Failures

- **Generators and controllers:** bearings, regulation, GCU internal fault.
- **Contactors:** stuck open, stuck closed, slow to operate.
- **Converters:** degraded output.
- **Battery cells:** internal short, leading to heat and thermal runaway spreading to neighbouring cells.
- **Breakers:** spurious trip, failing to trip.
- **Wiring:** chafe, arcing, open circuit.
- **Sensors:** offset or failed.

## Temperature, wear and airframe differences

- **Cold:** a cold battery has higher resistance and sags more, and the charger locks out. Breaker trip behaviour depends on temperature.
- **Wear:** battery capacity fade and generator bearing wear.
- **Airframe tolerances:** battery capacity, converter efficiency, sensor offsets.

## Test scenarios

1. Cold and dark, then battery on: which buses come alive, and how fast the battery drains.
2. APU start on battery after a +15 °C soak versus a −30 °C soak.
3. Engine start on two generators versus one, and from external power versus the APU.
4. Engine failure in cruise: transfers, load shedding, and cabin compressors reducing.
5. Loss of all engine generators: the RAT deploys and only essential buses stay powered.
6. Intermittent arc: the thermal breaker misses it and the SSPC's arc-fault detection catches it.
7. Power transfer in the middle of a boot: the right computers reboot.
8. The 248-day counter.
9. Pull each documented breaker: its load loses power and the expected function degrades. This one is generated for every breaker.

## Documents needed

- The bus diagram, and which load is on which bus.
- Which transfers briefly cut power.
- Load-shed priorities.
- Which motor uses which motor controller.
- Battery cell parameters and charger temperature limits.
- External power behaviour.
- RAT deployment logic.
- GCU protection thresholds.
- Computer hold-up times.
- **The full breaker list:** names, ratings, buses and loads, from the wiring diagram manual or SSPC configuration list and the FCOM's CBIC description.
