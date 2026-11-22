# Shared: radio propagation

This is the physics every radio uses. It is written once and shared.
Status: draft. **(verify)** means a source document is needed.

## Users
GNSS, VHF, HF, SATCOM, ILS, VOR, DME, marker beacons, radio altimeter, transponder/ADS-B, TCAS, weather radar, ELT, jammers.

## Signal path: every transmitter to every receiver, each tick
- **Free-space loss.**
- **Antennas.** Real gain pattern, rotated by the aircraft's attitude. Fuselage, wing and tail blockage. Cable loss.
- **Line of sight.** Blocked by terrain and buildings, over the Earth's curvature with refraction.
- **Diffraction over ridges** (ITU-R P.526).
- **Multipath.** Reflections off ground, water and buildings, using the surface type: water reflects strongly, forest weakly.
- **Troposphere.**
  - Ray bending and ducting, from X-Plane's weather profile.
  - Gas absorption (ITU-R P.676).
  - Rain and cloud loss (ITU-R P.838, P.840).
- **Ionosphere.**
  - Delay and scintillation, for GNSS.
  - Skywave reflection for HF: the layers change with day, night and solar activity.
  - Daytime and polar absorption.
  - Night skywave on NDB, which causes ADF wander.
- **Scattering.** Rain, hail, snow and ice for the weather radar. Ground clutter.
- **Doppler.**

## Noise and interference
- **Receiver noise.** Sky and cosmic noise.
- **Lightning static on HF.**
- **P-static.** Charge builds up in snow and ice crystals and puts noise on VHF and HF. Static wicks reduce it, and they can be missing or broken.
- **Co-channel.** Two stations on one frequency make heterodyne whistles. Two pilots transmitting at once block each other.
- **Busy 1090 MHz.** In dense traffic, transponder and TCAS replies overlap and garble.
- **Our own transmitters.** They interfere with our own receivers, for example VHF data link against VHF voice, or SATCOM against GPS **(verify which pairs matter)**.
- **5G C-band near US airports.** It interferes with the radio altimeter **(verify FAA AD details)**.
- **Jammers and spoofers.**

## Transmitters in the world
Each has a position, antenna height, power, pattern and status.
- **VOR, DME, ILS, NDB, markers:** X-Plane nav data. **ATC frequencies:** X-Plane airport data.
- **GBAS stations** **(verify list)**.
- **HF ground stations** **(verify)**.
- **Satellites:** GNSS and SATCOM, on real orbits.
- **Other aircraft:** transponder, ADS-B, TCAS and VHF radio, from AI and multiplayer traffic.
- **5G towers:** public FCC/FAA data **(verify)**.
- **Outages:** stations listed as out of service by NOTAM can be turned off.

## Our antennas (verify all)
3 VHF, HF in the fin, SATCOM on top, 2 GPS, radio altimeter antennas underneath, ILS localizer and glideslope in the nose, DME, transponder/TCAS top and bottom, weather radar in the nose, ELT.

## Receivers
- Sensitivity, selectivity, AGC, squelch.
- **Audio.** Static, fading and whistles, all from the signal math. Morse idents decoded from the real station.

## Terrain
- **Near the aircraft:** X-Plane's terrain probe.
- **Far from the aircraft:** our own reader for X-Plane's scenery files, following your scenery pack order.
- **No scenery installed:** fall back to the Copernicus 30 m elevation model.
- **Surface type and buildings:** from the same scenery files.

## Validation
- **VOR, DME and ILS:** reception range and multipath compared with published coverage.
- **HF:** propagation compared with published prediction tools for the same date and solar activity.
- **Weather radar:** returns compared with real radar imagery for the same weather data.

## Out of scope
- Raw waveforms.
- Full electromagnetic simulation of antennas. A measured gain pattern is used instead.
