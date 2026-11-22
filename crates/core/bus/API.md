# b789-bus (lib `bus`)

All times are u64 microseconds (AFDX internals use ns, converted with ceil). No unsafe, no randomness, BTreeMap only.

## a429
- `Word(u32)`: `pack(label_octal, sdi, data19, ssm)`, `label()` (wire bit-reversed), `label_octal()`, `sdi()`, `data()`, `ssm_bits()`, `parity_ok()` (odd), `corrupt(bit)`, `bnr_ssm()`, `bcd_ssm()`, `discrete_ssm()`, `discrete_bit(11..=29)`, `with_discrete_bit`
- `BnrSsm`, `BcdSsm`, `DiscreteSsm` with `bits()` / `from_bits()`
- `BnrFormat{total_bits, range}`: `new`, `resolution`, `encode`, `decode`, `pack_word`, `read_word` (None unless NO + parity ok); two's complement, MSB-aligned, saturating
- `BcdFormat{digits, resolution}`: `new`, `max_counts`, `encode`, `decode`, `pack_word`, `read_word`; sign in SSM
- `Speed::{Low,High}`: `bits_per_second`, `bit_time_us`, `word_time_us`, `slot_time_us` (32 + 4 gap bits), `max_words_per_second`
- `Transmitter`: `new(speed)`, `add_label(label, sdi, interval_us, offset_us)`, `set_word`, `poll(now) -> Vec<(start, end, Word)>`, `utilisation`
- `WordReceiver`: `new(timeout_us)`, `receive(time, word)`, `freshness -> Freshness{Fresh,Stale,Never}`, `get`, `age_us`, `parity_errors`

## afdx
- consts `LINK_BPS`, `OVERHEAD` (46), `PREAMBLE_IFG` (20), `MIN_FRAME` (64); `frame_bytes`, `wire_bytes`, `wire_time_ns`
- `Priority::{High,Low}`; `VirtualLink{id, bag_us, lmax, priority}`: `new`, `max_payload`, `max_bandwidth_bps`
- `EndSystem`: `schedule(vl, net, payload, ready_ns) -> (start, end)`, `commit` (BAG shaping + port serialisation)
- `Switch`: `new(fabric_latency_ns)`, `failed`, `enqueue(port, rx_complete_ns, Frame)`, `run(until_ns)`; store-and-forward, per-port priority queues
- `RedundancyManager`: `new(window)`, `accept(dest, vl, seq)` first-valid-wins on u8 sequence, `discarded`
- `AfdxNetwork`: `new(fabric_latency_ns)`, `add_virtual_link(vl, src, dests)`, `send(vl, payload, now_us)`, `poll(until_us) -> Vec<Delivery>`, `set_network_failed(net, bool)`, `redundancy_discards`, public `switches: [Switch; 2]`
- `FunctionalStatus` (NO 0x03, NCD 0x0C, FT 0x30, FW 0x00; assumed 8-bit FS bytes), `ParamEncoding`, `ParamDef`, `MessageLayout{size_bytes, fs_base_byte, params}`: `add`, `encode`, `decode`; `write_bits`, `read_bits` (MSB-first)

## can
- `CanId::{Standard, Extended}`: `arbitration_key`, `is_extended`
- `CanFrame`: `new`, `payload`, `stuffed_bits`, `bits_exact` (real CRC-15 and stuffing), `bits_worst_case`, `time_us_worst_case`; `crc15`
- `CanNode`: `tec`, `rec`, `state() -> CanState{ErrorActive,ErrorPassive,BusOff}`, `tx_error`, `tx_success`, `rx_error`, `rx_success`, `recessive_bits` (bus-off recovery 128x11), `may_transmit`
- `CanBus`: `new(bitrate)`, `submit(ready_us, node, frame)`, `process(until_us) -> Vec<(start, end, node, frame)>`

## wire
- `DiscreteWire::{Open,Ground,Volts28}`; `DiscreteInput`: `new(on, off)`, `new_28v`, `update(volts)`, `update_wire`, `ground_sense`
- `AnalogInput`: `new(bits, min_v, max_v)`, `lsb`, `sample -> AdcReading{code, value, valid}`

## transport
- `Message{link, seq, sent_us, deliver_us, payload}`; `SerialLink::{a429, can}`; `Transport`: `register_serial`, `post`, `post_a429`, `post_can`, `deliver_due(now)` ordered by (time, link, seq), `pending`, `next_delivery_us`
