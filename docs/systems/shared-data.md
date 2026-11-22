# Shared: data dictionary (ICD)

Every piece of data on the aircraft, written in one place. It's the single source of truth for packing ([42-ccs-cdn.md](42-ccs-cdn.md)) and for what every box ([computers.md](computers.md)) sends and receives.
Status: draft. **(verify)** means a source document is needed.

## Every parameter records
- **Name, meaning, units.**
- **Range, resolution, encoding:** BNR, BCD, discrete, float, enum.
- **Source box and its function.**
- **Every consumer.**
- **Path:**
  - CDN: VL, data set and byte or bit offset.
  - ARINC 429: label, SDI, rate.
  - CAN: ID and rate.
  - Discrete or analog: pin, wire and RDC.
- **Rate and latency budget.**
- **Validity rules:** status bits, freshness timeout, what a consumer does when the data is invalid.
- **Source citation:** document and page. Anything not from a document is marked `APPROX`.

## Every message records
- **VL / label / CAN ID:** its BAG or rate, size and priority.
- **Its parameters, in order.**
- **The ICD version.**

## How it's filled in
1. **Data flows from the documents:** STM and ATA interface sections, block diagrams, FCOM descriptions. Being collected now into `data_flows.md`.
2. **Real values where public:**
   - ARINC 429 general labels (public standard tables);
   - rates and ranges stated in FCOM, STM and ATA.
3. **Gaps get engineered** from physics and display needs, for example the resolution needed to show what the PFD shows. They're marked `APPROX`.

## Generated from it
- Encoders and decoders for every box.
- VL and switch routing tables.
- RDC pin maps.
- The inspector's decoders.
- Audit tests:
  - every parameter has a source and at least one consumer;
  - every consumer's input exists;
  - every rate fits its BAG.

## Versioning
- Each box's software load names the ICD version it was built for.
- Mixed versions decode wrong, which is a real failure mode.

## Tests
1. Round-trip every parameter: encode then decode, within its resolution.
2. Every display value traces back to a sensor through real paths.
3. A mismatched ICD version on one box gives the expected wrong decode.
4. Every documented data flow in `data_flows.md` exists in the ICD.
