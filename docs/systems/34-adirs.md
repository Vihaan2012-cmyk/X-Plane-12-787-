# 34 — Inertial (IRS) and air data (ADRS)

Status: draft. Hardware comes from LOT FCOM 10.20.17–18. **(verify)** means a source document is needed.

## Approach
- X-Plane gives the true motion and atmosphere. We add realistic sensor and computer error on top.
- **Inertial:** the real navigation error equations (Schuler 84.4 min, Earth rate 24 h), driven by modelled sensor errors.
- **Air data:** the true pressure at each port and probe, passed through our pneumatic line, ADM and computation models.

## Hardware (LOT FCOM 10.20.17–18)
- **IRS:**
  - **2 IRUs:** the primary source for inertial navigation position, from their own gyros and accelerometers plus GPS inputs.
  - **2 AHRUs:** the primary source for attitude and heading.
  - Details are in FCOM ch. 11.20 **(verify: read it)**.
- **ADRS:** primary, secondary and standby air data.
  - **6 static ports**, 3 on each side. A left and a right port are paired through tubing to each of the left, centre and right ADMs, so sideslip errors cancel.
  - **3 pitot probes:** right and centre on the right forward fuselage, left on the left. Each has its own ADM.
  - **6 ADMs:** 3 static, 3 pitot. Each converts pressure to digital data.
  - **2 AoA sensors**, one on each side of the forward fuselage.
  - **TAT:**
    - 1 TAT probe, plus each EEC's TAT sensor, gives 6 inputs. They're combined as a weighted average into a "trusted TAT".
    - On the ground below 50 kt CAS, the CAC inlet sensors provide TAT (2 per inlet), and the lower of the two is used.
  - **Where the ADRS computations run:** CCS or FCMs **(verify)**.
- **Standby air data:** source and display **(verify)**.
- **GPS:** fully modelled in [34-gnss.md](34-gnss.md).

## Inertial
- **Sensor errors.** Each gyro and accelerometer has bias, scale factor, misalignment and noise. Bias depends on temperature, so a cold unit drifts more until it warms.
- **Alignment is a filter, not a timer.** It ends when heading is certain enough, so its time depends on:
  - latitude: Earth rate × cos(latitude), and a limit latitude **(verify)**;
  - gyro quality;
  - gusts and loading.

  It's calibrated to real timings.
- **Navigation.** Wander-azimuth formulation. GPS blending inside the IRU.
- **Gravity.** The unit assumes standard gravity, but the truth is EGM2008, so the small real errors appear.
- **AHRUs** supply attitude and heading independently of the IRUs. Their own sensor quality **(verify)**.

## Heading
- True heading minus the unit's own dated magnetic variation table. The truth is the current World Magnetic Model.
- Reverts to true heading at high latitude **(verify)**.

## Air data
- **Pitot:** each probe is a small pressure chamber (inlet, drain, line volume, heater).
  - Inlet blocked, drain open: airspeed drops to 0.
  - Both blocked: airspeed rises in a climb.
  - Dead heater in icing: the probe blocks by itself.
- **Static:**
  - Each ADM's L+R port pair averages out sideslip error.
  - Position error from Mach, AoA and flaps, with a correction table **(verify)**.
  - One port blocked skews its pair. Both blocked freezes that ADM.
  - A line leak inside the hull reads cabin pressure.
- **AoA vanes:** vane dynamics and friction. They can freeze, or be bent and keep the offset.
- **TAT:**
  - Recovery factor, heater error, ice-crystal errors on the probe.
  - The weighted average across all 6 inputs rejects one bad sensor.
  - The ground CAC-inlet switchover happens below 50 kt.
- **ADMs:** transducer offset and drift with temperature, plus noise.
- **Computed outputs:** pressure altitude, baro-corrected altitude, CAS, Mach, TAS, SAT (shown on the CDU PROGRESS page), baro-inertial vertical speed.
- **Voting:** left, centre and right sources are compared, and a bad source is excluded.
- **EICAS:** NAV AIRSPEED DATA and NAV AIR DATA SYS are the names from FCOM.

## AoA SPD: backup airspeed (FCOM 10.20.18)
- **When it's used:** pitot or static data is bad.
- **How it's computed:** speed from AoA plus inertial data. Its errors with configuration changes, manoeuvring and speedbrake use emerge from that.
- **What changes while it's shown:**
  - Flight controls use simplified laws, so pitch is more sensitive (flight controls plan).
  - Maximum speed indications drop from 360 kt / .90 M to 300 kt / .85 M, and overspeed alerts are lowered.
- **Triggers:**
  - automatically, on NAV AIRSPEED DATA or NAV AIR DATA SYS;
  - manually, when both pilots select ALTN on the AIR DATA / ATT source selector.
- An anomaly at high speed can bring the OVERSPEED siren until the indicated speed drops.

## Software behaviour
- IRU and AHRU modes: power-up self-test, align, nav, attitude-only, fault **(verify names: ch. 11.20)**.
- Self-test finds latent faults left from the previous session.
- **Power loss:** the unit rides through its hold-up time, otherwise navigation is lost. Battery backup **(verify)**.
- Shutdown stores the last position.
- Outputs go on the CDN with real packing and validity ([42-ccs-cdn.md](42-ccs-cdn.md)).

## Links
- **Takes:**
  - Electrical: unit power and heater power.
  - Ice and rain: icing.
  - GNSS.
  - Displays: altimeter settings.
  - EECs: TAT.
  - CACs: inlet temperatures.
- **Gives:**
  - displays and HUD;
  - flight controls: AoA, air data, attitude;
  - autoflight and FMS;
  - engines;
  - pressurisation;
  - transponder and ADS-B;
  - weather radar stabilisation;
  - maintenance;
  - recorders.

## Failures (0–1)
- **IRU and AHRU:** each gyro and accelerometer, processor, power.
- **Pitot:** inlet, drain, heater, leak.
- **Static:** each port blocked (ice, or tape left on), line leak, disconnected.
- **ADM:** drift, offset, failed.
- **AoA:** jammed, bent, heater, resolver.
- **TAT:** probe heater, element. EEC TAT sensors via the engines. CAC inlet sensors via air conditioning.
- **Ground events:** insects in an uncovered pitot, pitot cover left on.

## Temperature, wear and airframe differences
- **Temperature:** cold units have larger biases. Heater output depends on voltage and element resistance.
- **Wear:** heater elements and vane bearings.
- **Airframe tolerances:** each tail number keeps its own sensor errors, vane offsets and static residuals.

## Tests
1. Alignment time at 51° N against 78° N.
2. Gusts slow alignment.
3. A cold-soaked IRU drifts more at first.
4. 10 hours without GPS: Schuler oscillation, drift within spec.
5. Left pitot blocked: voting excludes it and there's no crew-visible change.
6. Two pitots blocked: NAV AIRSPEED DATA, AoA SPD shown, simplified laws, limits reduced to 300 kt / .85 M.
7. Both pitot and drain blocked: airspeed rises in a climb.
8. One static port taped: its pair skews and voting catches it.
9. Pitot heat failure in icing: the blockage emerges by itself.
10. TAT probe fails: the weighted average from the 5 remaining inputs stays good.
11. On the ground below 50 kt: TAT comes from the CAC inlets, lower of the two.
12. An AoA vane frozen on the ground affects the protections.
13. Two airframes give slightly different altitudes, each repeatable session to session.

## Documents
- **LOT FCOM:** 10.20 (have), 11.20 IRU/AHRU (have, read next).
- **QRH:** NAV AIR DATA SYS, AIRSPEED UNRELIABLE.
- **ATA 34:** not in our set **(get if possible)**. ATA 30 (have) covers probe heat.
- **FAA ADs:** air data and AoA.
- **Public models:** NOAA WMM, EGM2008 and EGM96.
- **Videos:** alignment times.
