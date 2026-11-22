# b789-store (lib `store`) API

Modules: `rng`, `codec`, `common`, `store`. Everything re-exported at crate root.

## rng
- `SplitMix64::new(seed)`, `next_u64()`, `next_f64()` in [0,1), `range(lo, hi)`, `gaussian()`
- `hash_str(&str) -> u64` (FNV-1a), `seed_for(serial, name) -> u64`
- `tolerance_factor(serial, name, sigma, clip_sigmas) -> f64` = 1 + sigma * clipped gaussian, same serial gives same value

## codec (text snapshot tokens, tab separated, f64 as exact hex bits)
- `Encoder::new()`, `.str() .u64() .f64() .bool()` (chainable), `.finish() -> String`
- `Decoder::new(&str)`, `.str() .u64() .f64() .bool() -> Option`, `.rest()`

## common
- `PartId(pub u32)`
- `Grade { Verified, Derived, Standard, Approx }`, `as_str()`, `Grade::parse()`
- `Source(pub String)`, `new()`, `is_empty()`
- `Location { station_in, buttline_in, waterline_in }`, `new()`, `distance_in()`
- `AMBIENT_K`
- `PartCommon { name, kind, zone, location: Option<Location>, temperature_k, wear, tolerances: BTreeMap<String,f64>, serial, installed_at_us, hours, cycles, grade, source }`
  - `new(name, kind, zone)`, `with_location()`, `with_serial()`, `with_grade(grade, source)`, `with_install(at_us, hours, cycles)`
  - `draw_tolerance(name, sigma, clip)`, `tolerance(name)` (1.0 if absent), `set_wear()`, `add_wear()` (clamped 0-1), `accrue(hours, cycles)`, `age_us(now)`, `has_source()`
  - `encode(&mut Encoder)`, `decode(&mut Decoder) -> Option<Self>`

## store
- traits `Part { common(), common_mut() }` and `Persist { encode(), decode() }` (both implemented for `PartCommon`)
- `StoreError { DuplicateName, UnknownId, Parse }`
- `Store<T: Part>`
  - `new()`, `len()`, `is_empty()`, `next_id()`
  - `insert(part) -> Result<PartId, StoreError>`, `remove(id)`, `contains(id)`; ids never reused
  - `get(id)`, `get_mut(id)` (marks dirty, wakes), `id_of(name)`, `by_name()`, `by_name_mut()`
  - `ids()`, `iter()` in id order; `ids_of_kind()`, `ids_in_zone()`, `kind_names()`, `zone_names()`
  - `rekey(id)` after changing name, kind or zone through `get_mut`
  - dirty: `mark_dirty()`, `is_dirty()`, `dirty_count()`, `take_dirty()`, `clear_dirty()`
  - sleep: `sleep()`, `wake()`, `is_asleep()`, `wake_zone()`, `wake_kind()`, `wake_all()`, `awake_ids()`, `awake_count()`, `sleeping_count()`
  - `snapshot() -> String`, `Store::restore(&str) -> Result<Store<T>, StoreError>` for `T: Part + Persist` (keeps ids, dirty and sleep flags; detects truncation and duplicates)
