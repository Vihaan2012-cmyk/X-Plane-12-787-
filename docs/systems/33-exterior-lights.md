# 33 — Exterior lights

Status: draft. **(verify)** means a source document is needed.

## Hardware (verify all: FCOM Vol 2 lights chapter and photos)
| Light | Where | Colour |
|---|---|---|
| Landing | Wing roots and nose gear | White |
| Taxi | Nose gear | White |
| Runway turnoff | Left and right | White |
| Navigation | Wingtips and tail | Red left, green right, white aft |
| Beacons | Top and bottom of fuselage | Red, flashing |
| Strobes | Wingtips and tail | White, flashing |
| Logo | Horizontal stabiliser, lighting the fin | White |
| Wing inspection | Fuselage, aimed at the leading edge and engine | White |
| Wheel well | Gear bays | White |

- **Each light records:** position, aim, beam shape, lamp type (LED or incandescent **(verify)**), driver, supply bus and breaker.

## Control path
- Switch → data concentrator → network → light control function → SSPC → driver → LED.
- **Any link down:** the light stops responding.
- **Backups:** hardwired paths, if any **(verify)**.
- **Automatic logic:**
  - Nose gear lights go off when the gear is up (gear sensors).
  - Any other automatic logic **(verify)**.

## Electrical, both ways
- **Brightness → current.** A dimmed light draws less. Driver efficiency curve: power in = light out + heat.
- **Voltage → brightness:**
  - LED with a constant-current driver: steady until the driver drops out.
  - Incandescent: brightness about ∝ V^3.4, so a sagging bus dims it.
- **Inrush** when the driver powers up.
- **Strobes:** a capacitor charges between flashes, so current pulses at the flash rate and adds to generator load.
- **Driver failure:** short, so the SSPC trips; or open, so the light is out.

## Heat
- **LED chip temperature.** Hot gives less light; cold gives a little more.
- **Cooling** comes from airflow, which depends on airspeed.
- **Parked with lights on:** they heat up, the driver cuts current, so the light dims and draws less **(verify)**.
- **Waste heat:** goes into the zone, and some of it reaches the lens.

## Lens
- **Ice.** Accretion rate against the lens's heat. LED lenses ice up more than incandescent ones.
- **Dirt** cuts the light that gets through. Washing restores it.
- **Plastic yellows** over the years.
- **Inside fogging** in cold, humid air.
- **Bird strike** can crack it.

## Light output
- Intensity in candela, beam angles, colour and colour temperature.
- **Minimums (FAR 25.1389–1401):**
  - navigation light coverage angles and colours;
  - anti-collision lights at least 400 cd and 40–100 flashes a minute.
- **Flash patterns** timed from real 787 night videos.
- **Output to X-Plane.** Each light publishes its actual brightness (0–1), plus its colour and beam.
  - **No 3D model yet:** we drive X-Plane's default light datarefs.
  - **With the model:** its lights read our values.
  - X-Plane draws the spill light on the ground.

## Aging and airframe differences
- LED output and colour drift with operating hours.
- Each airframe has its own LED and driver spread.
- Lens condition is kept between sessions.

## Failures (0–1)
LED string (partial dimming), driver short or open, SSPC trip, wiring, switch, control function, lens cracked, iced or dirty.

## Indications
- Maintenance messages from the driver's self-test.
- EICAS messages **(verify)**.

## Tests
1. Switch on: the right light comes on and draws the right current.
2. Dimmed: less current.
3. Gear up: nose lights go off.
4. Bus or breaker lost: only that light goes out.
5. Low voltage: an LED holds brightness, an incandescent dims.
6. Parked a long time with lights on: the light heats up and its output drops **(verify)**.
7. Icing: an LED lens ices up faster than an incandescent one.
8. Flash timing matches the video. Anti-collision lights meet the 400 cd and 40–100 per minute minimums.
9. 10,000 hours: the light is dimmer and its colour has shifted.
10. Strobe current pulses show on generator load.

## Documents
FCOM Vol 2 lights chapter, AMM 33, FAR 25.1389–1401, walkaround photos, night videos, light supplier datasheets **(verify part numbers)**.
