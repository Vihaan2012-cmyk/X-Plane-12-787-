# 34 — GNSS

Status: draft. **(verify)** means a source document is needed.
Signal travel through the atmosphere, terrain and interference comes from [shared-radio.md](shared-radio.md).

## Satellites
- **GPS:** every satellite on its real orbit for the sim's date, with each one's own power, clock and health.
- **SBAS and GBAS** if the 787-9 uses them **(verify)**. Galileo and others are left out unless it does **(verify)**.
- **Data:**
  - real broadcast navigation messages (RINEX);
  - precise orbits and clocks (IGS);
  - outage notices (NANUs);
  - an offline almanac shipped with the aircraft.
- **Time:** sim time converted to GPS time, including leap seconds.

## Signal: what each receiver gets from each satellite
- True range at the moment of transmission.
- Every IS-GPS-200 term: satellite clock, relativity, group delay, Earth's rotation during transit.
- Propagation effects from the shared radio plan.
- Signal strength from the antenna pattern rotated by the aircraft's attitude, plus body and terrain masking.

## Receivers (verify: 2, model, channels, antennas)
- **Correlator maths for each channel**, every few milliseconds, computed on the combined signal (real + reflections + jammer + spoofer). It gives:
  - lock and loss of lock;
  - multipath error;
  - spoofer capture and drag-off;
  - cycle slips;
  - carrier smoothing.
- **Acquisition.** Cold, warm and hot starts, from the stored almanac, last position and clock.
- **Clock.** A crystal oscillator whose drift depends on temperature, so a cold-soaked receiver is slower to get a fix.

## Navigation solution
- Least squares, then a Kalman filter: position, velocity, clock bias and clock drift.
- **Integrity (RAIM).** Fault detection and exclusion, protection levels, and the alert limit for each flight phase.
- SBAS/GBAS corrections and integrity **(verify)**.
- Coupling to the ADIRU, loose or tight **(verify)**.
- Altitude: GPS height converted to sea level with the EGM96 geoid.

## Outputs
| To | What |
|---|---|
| ADIRS | Position, velocity, or pseudoranges |
| FMS | Position; actual vs required navigation performance; integrity prediction for arrival time |
| ADS-B | Position, velocity and quality figures (NIC, NACp, SIL) |
| Clocks, central computers, recorders | UTC time |
| Displays and EICAS | GPS and integrity status |

## Failures (0–1)
- **Satellite:** clock jump, ephemeris error, signal distortion, outage.
- **Receiver:** channel, clock, processor, power.
- **Antenna and cable:** gain loss, open or short, water in the connector.
- **Environment:** jamming, spoofing, ionospheric storm, scintillation, solar radio burst.

## Temperature, wear and airframe differences
- **Temperature:** the receiver clock depends on temperature.
- **Wear:** antenna and cable aging.
- **Airframe tolerances:** cable loss, clock quality.

## Tests
1. A real date and place gives the real satellite visibility and dilution of precision.
2. A hot start takes seconds. A cold start after weeks shows the almanac download time.
3. A steep bank masks low satellites and the protection level rises.
4. A satellite clock runaway is detected and excluded. With too few satellites, an integrity alert fires instead.
5. In a jamming zone, satellites drop one by one, then the ADIRU goes inertial-only and ADS-B quality drops.
6. A spoofer pulls the position, and the inertial comparison catches it **(verify 787 behaviour)**.
7. Equatorial scintillation after sunset causes intermittent loss of lock.
8. Leap second and week rollover.
9. **Validation:** predicted pseudoranges and signal strength match real IGS station recordings within tolerance.
10. **Validation:** ADS-B quality matches recorded real flights over the same route and date.

## Documents
- IS-GPS-200, NAVCEN, Celestrak, IGS (all public).
- 787-9 receiver details: FCOM Vol 2 navigation, AMM 34, supplier sheets.
- Optional: RTCA DO-229, DO-316, DO-253; ICAO Annex 10.
- Textbooks: Misra & Enge; Kaplan & Hegarty.
