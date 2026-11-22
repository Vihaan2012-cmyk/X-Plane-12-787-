# 46 — EFB (Boeing installed EFB) and the sim tablet

Status: draft. Hardware facts are pending from the information-systems reading agent. **(verify)** means a source document is needed.

## Two separate things
1. **The Boeing EFB.** Real 787 equipment, modelled like every other system.
2. **The sim tablet.** It isn't aircraft equipment and is kept off the Boeing EFB: failures, ground services, payload and fuel loading, time and soak settings, the system inspector for every system ([shared-inspector.md](shared-inspector.md)), settings.

## Boeing EFB: hardware (verify: ATA 46, STM ch. 7)
- **Two EFB electronic units, one per pilot,** with displays at the side positions **(verify)**.
- Each unit is a box built on [shared-computer.md](shared-computer.md): power feed, boot, self-test, heat, failures.
- **Network.**
  - It sits on the core network.
  - Avionics data (position, fuel, weights, time) only arrives through the real gateway between the core network and the CDN, which is read-only in that direction **(verify)**.
- **When a unit loses power or its link,** its apps go down. The other unit keeps working.

## Boeing EFB: applications
| App | What it does | Data source |
|---|---|---|
| **Performance (OPT)** | Takeoff and landing calculations: V-speeds, flaps, assumed temperature, derate, landing distance | Our performance model, calibrated to the LOT FCOM PD/PI chapters |
| **Charts** | Airport, SID, STAR, approach | Chart providers, below |
| **Airport moving map** | Own-ship position on the airport layout | AMDB Bridge (AMDB / routing network) |
| **Documents** | Our aircraft manual and procedures | Our own text. Boeing's FCOM can't be shipped |
| **Electronic logbook** | Write-ups and deferrals | Feeds the maintenance and MEL state, which carries over between sessions |
| **Video surveillance** | Flight deck door and cabin cameras | Sim camera views |
| **Flight plan / OFP** | Loads the plan | AMDB Bridge planner (SimBrief-compatible), or SimBrief |
| **Weather** | METAR, TAF, SIGMET | AMDB Bridge or public weather feeds |

## Chart providers: all of them, behind one interface
- **Navigraph:** uses the user's own subscription (OAuth device login). It needs Navigraph's developer approval for the add-on.
- **AMDB Bridge:** the user's own tool. It provides airport diagrams and rendered SID/STAR/approach charts built from free data, needing no account.
- **Official free sources:**
  - FAA d-TPP (US);
  - national eAIPs that publish freely (for example UK, India, Australia and others) **(verify each country's terms)**.
- **Order:** the user picks the preferred provider. Missing charts fall back to the next provider, and each chart shows which provider it came from.
- **Possible extra provider:** the user's government-charts-to-Jeppesen-style project **(cloud session)**, if it produces a usable output.

## Data paths (no shortcuts)
- The EFB's own data (charts, AMDB, plans) comes from outside services over the EFB's network link. That's like the real aircraft's Gatelink or cell connection, which only works on the ground or where connectivity exists **(verify 787 connectivity options)**.
- Aircraft data reaches the EFB only through the core network gateway.

## Failures (0–1)
- Unit power, processor, display, touch, storage.
- Network link, gateway.
- App crash.
- Stale chart or database cycle.

## Tests
1. Pulling one EFB's power kills only that side's apps.
2. With the gateway failed, the EFB can't see position or fuel, but its charts still work.
3. OPT takeoff numbers match the LOT FCOM PD tables within tolerance.
4. Every chart shows its provider, and fallback works when a provider is missing.
5. Logbook deferrals appear in the MEL state the next session.

## AMDB Bridge integration (interface: [ref-amdb-bridge.md](ref-amdb-bridge.md))
- **Connect** on `http://127.0.0.1:8770` and detect it with `GET /v1`. The EFB can start it itself with `amdb-bridge serve --xplane --no-hosts`, which needs no admin.
- **Moving map:** `/v1/{ICAO}` in the ARP_AZEQ projection (metres), requesting one airport at a time and dropping stale replies.
- **Taxi routing:** the bridge's ASRN router (`/amdb/taxi-route`) drawn on the map.
- **Charts:** `/v2/charts`, used as one provider among several. Charts are georeferenced, so own-ship can be drawn on them.
- **OFP:** the SimBrief-compatible endpoint.
- **Attribution shown on screen:** "© OpenStreetMap contributors" and "X-Plane Scenery Gateway".
- **Limit:** without MSFS, the bridge has few charts (FAA and CIFP-drawn US charts only). Navigraph and the official free sources cover the rest.

## Pending
- EFB hardware facts: information-systems agent.
- EFB hardware facts: information-systems agent.
