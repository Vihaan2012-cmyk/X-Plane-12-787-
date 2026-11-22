# Reference: AMDB Bridge client interface (v1.2.15)

Source: the user's AMDB Bridge project session, read from its code on 2026-11-19. AMDB Bridge is the user's own tool, MIT licensed.

## Transport
- **Address:** plain HTTP on `http://127.0.0.1:8770`, loopback only (`--http-port` changes it). The HTTPS listener on 443 is only for Navigraph-hostname redirects; don't use it.
- **Format:** request/response JSON, PNG or Lua. No WebSocket, no auth, CORS open.
- **Detection:** `GET /v1` returns `{"service":"amdb-bridge","version":"…","loaded":N}`. There's no capability list: a 404 means not supported.
- **Starting it:** the user runs the tray app, or the plugin runs `amdb-bridge serve --xplane --no-hosts` itself. That mode needs no admin.
- **First request for an unbuilt airport** blocks for 1–25 s while it downloads or builds. Ask for one airport at a time and drop stale replies. This is the cause of the FBW OANS race bug.

## Airport mapping (the Navigraph AMDB API, 1:1)
- **Requests:**
  - `GET /v1/{ICAO}?include=&exclude=&projection=&precision=` returns `{layer: GeoJSON FeatureCollection}`. An empty `include` returns all 36 layers.
  - `GET /v1/{ICAO}/{layer}` returns one layer.
- **Projection:** `EPSG:4326` (the default), or `NAVIGRAPH:ARP_AZEQ`: metres, centred on the aerodrome reference point, x east and y north.
- **Layers:** the DO-272/ED-99 model, including the ASRN routing network (`asrnnode`, `asrnedge`), stands, holding positions, centrelines, hotspots and structures.
- **Conventions:** elevations in metres. Sentinels: −32767 means unknown, −32765 means not applicable.
- **Search:** `/v1/search?q=`, and the bridge-only `/v1/nearest?lat=&lon=&radius_km=&limit=`.
- **Taxi routing (bridge only):**
  - `GET /amdb/taxi-route?icao=&lat=&lon=&clearance=A B K 04L` returns the route legs and ARP-frame points.
  - `GET /amdb/taxi-route/current` and `GET /amdb/taxi-route/clear`.

## Charts (the Navigraph charts v2 shape)
- **List:** `GET /v2/charts/{ICAO}`. Each chart includes id, category (APT, DEP, ARR, APP), procedures, runways, image URLs, and georeferencing (`is_georeferenced`, `bounding_boxes.planview`).
- **Image:** `GET /v2/charts/{ICAO}/{id}.png?theme=day|night`, about 1488×2105 px, plus a thumbnail.
- **Cycle:** `/v1/cycle`.
- **Where the charts come from:**
  - Drawn from MSFS navdata plus terrain. That's personal use only: never redistribute or bundle them.
  - Without MSFS, only charts already filed are listed: FAA d-TPP PDFs and CIFP-drawn US charts. **An X-Plane-only user gets few charts.**
- **Enroute tiles:** `GET /enroute/styles/{ifr.lo|ifr.hi|vfr|world}.{day|night}/{z}/{x}/{y}@2x.png`.

## Flight plans
- **SimBrief-compatible:** `GET /api/xml.fetcher.php?json=1` returns the last planned OFP. It returns HTTP 400 if there's no plan.
- **Planner:** `/plan` is the web page. The API is `/plan/start?from=&to=&aircraft=…`, `/plan/events?since=N`, `/plan/types` and `/plan/airframes`.

## Nav data
- None live for an FMS. Our FMS uses X-Plane's CIFP.

## Licensing (what the EFB must do)
- **Every map view shows** "© OpenStreetMap contributors" (ODbL), with a link where possible.
- **Credit "X-Plane Scenery Gateway"** (GPL 2.0 data; must stay redistributable).
- **FAA data and OurAirports** are public domain.
- **MSFS-derived charts and tiles** are personal use only.
- **Navigraph:** the EFB needs its own client credentials plus the user's subscription. Never reuse other add-ons' client IDs.
