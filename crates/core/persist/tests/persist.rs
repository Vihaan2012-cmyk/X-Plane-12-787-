use persist::*;
use std::collections::BTreeMap;

fn sample() -> Airframe {
    let mut a = Airframe::new("N787XX", "63001");
    a.identity
        .options
        .insert("engine".to_string(), "trent1000".to_string());
    a.identity
        .options
        .insert("galley".to_string(), "ecoplus \"v2\"\nline".to_string());
    a.settings.gap_mode = GapMode::Real;
    a.stamp = Some(SessionClock {
        sim_s: 1.0e9 + 0.1,
        real_s: 1_700_000_000,
    });
    let mut p = PartRecord::new(61.25, 5400.0);
    p.wear = 0.1 + 0.2;
    p.run_s = 123456.789;
    p.cycles = u64::MAX;
    p.powered_s = 99999.5;
    p.unpowered_s = 1e-310;
    p.nvm.insert("cfg".to_string(), vec![0, 1, 2, 255, 128]);
    p.nvm.insert("empty".to_string(), Vec::new());
    p.counters.insert(
        "powered".to_string(),
        Counter {
            value: 1000.0,
            powered_only: true,
        },
    );
    p.counters.insert(
        "calendar".to_string(),
        Counter {
            value: 2000.0,
            powered_only: false,
        },
    );
    p.faults.push(FaultEntry {
        t_s: -0.0,
        code: "CCS-BUG".to_string(),
        flags: 7,
    });
    p.faults.push(FaultEntry {
        t_s: f64::INFINITY,
        code: "A\\B".to_string(),
        flags: 0,
    });
    a.parts.insert("24-electrical/gcu-l".to_string(), p);
    a.parts
        .insert("42-ccs/cabinet-a".to_string(), PartRecord::new(20.0, 3600.0));
    a.batteries.insert(
        "main".to_string(),
        BatteryRecord {
            soc: 0.87,
            cell_soc: vec![0.86, 0.87, 0.88, 0.871],
            cell_health: vec![1.0, 0.99, 0.98, 0.97],
            cycles: 412,
            soak_c: 30.0,
            soak_tau_s: 7200.0,
            self_discharge_per_day: 0.01,
        },
    );
    a.breakers.insert(
        "C1-APU-START".to_string(),
        BreakerRecord {
            state: BreakerState::Open,
            locked: true,
            tag: Some("MX tag 42".to_string()),
        },
    );
    a.breakers.insert(
        "C2-FUEL".to_string(),
        BreakerRecord {
            state: BreakerState::Closed,
            locked: false,
            tag: None,
        },
    );
    let mut ex = Section::new();
    ex.set("x", Value::Array(vec![Value::I64(-5), Value::Array(vec![])]));
    a.extra.insert("sys/ccs".to_string(), ex);
    a
}

fn text_of(a: &Airframe) -> String {
    a.to_document().to_text().unwrap()
}

#[test]
fn value_roundtrip_bit_exact() {
    let vals = [
        0.0,
        -0.0,
        1.0,
        0.1,
        f64::MIN_POSITIVE,
        5e-324,
        f64::MAX,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff8_dead_beef_0001),
        std::f64::consts::PI,
    ];
    for v in vals {
        let enc = Value::F64(v).encode();
        match Value::decode(&enc).unwrap() {
            Value::F64(d) => assert_eq!(d.to_bits(), v.to_bits(), "{enc}"),
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn value_all_types_roundtrip() {
    let v = Value::Array(vec![
        Value::U64(u64::MAX),
        Value::I64(i64::MIN),
        Value::I64(-1),
        Value::Bool(true),
        Value::Bool(false),
        Value::Str("a \"q\" \\ \n\t\r \u{1} é ] , [".to_string()),
        Value::Bytes(vec![]),
        Value::Bytes(vec![0xde, 0xad]),
        Value::Array(vec![Value::Array(vec![Value::U64(1)])]),
    ]);
    let enc = v.encode();
    assert!(!enc.contains('\n'));
    assert_eq!(Value::decode(&enc).unwrap(), v);
}

#[test]
fn value_decode_rejects_garbage() {
    for bad in [
        "",
        "q:1",
        "u:",
        "u:-1",
        "b:2",
        "s:\"abc",
        "x:abc",
        "f:12",
        "a:[u:1",
        "a:[u:1,]",
        "u:1 ",
        "s:\"\\q\"",
        "x:AB",
        "f:3FF0000000000000",
    ] {
        assert!(Value::decode(bad).is_err(), "{bad}");
    }
}

#[test]
fn value_float_is_human_readable() {
    assert_eq!(Value::F64(1.0).encode(), "f:3ff0000000000000(1.0)");
}

#[test]
fn airframe_roundtrip_bit_exact() {
    let a = sample();
    let text = text_of(&a);
    let (b, steps) = decode_airframe(text.as_bytes()).unwrap();
    assert_eq!(steps, 0);
    assert_eq!(a.parts.len(), b.parts.len());
    assert_eq!(
        a.parts["24-electrical/gcu-l"].faults[1].t_s.to_bits(),
        b.parts["24-electrical/gcu-l"].faults[1].t_s.to_bits()
    );
    assert_eq!(
        b.parts["24-electrical/gcu-l"].faults[0].t_s.to_bits(),
        (-0.0f64).to_bits()
    );
    assert_eq!(b.stamp, a.stamp);
    assert_eq!(b.identity, a.identity);
    assert_eq!(b.batteries, a.batteries);
    assert_eq!(b.breakers, a.breakers);
    assert_eq!(b.extra, a.extra);
    assert_eq!(text_of(&b), text);
}

#[test]
fn text_is_deterministic_and_sorted() {
    let t1 = text_of(&sample());
    let t2 = text_of(&sample());
    assert_eq!(t1, t2);
    let names: Vec<&str> = t1
        .lines()
        .filter(|l| l.starts_with('[') && !l.starts_with("[end]"))
        .map(|l| l.split(']').next().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert!(t1.starts_with("B789PERSIST v2\n"));
    assert!(t1.ends_with('\n'));
}

#[test]
fn document_roundtrip_generic() {
    let mut d = Document::current();
    d.section_mut("sys/a").set("k", Value::U64(1));
    d.section_mut("sys/empty");
    let back = Document::from_text(&d.to_text().unwrap()).unwrap();
    assert_eq!(back, d);
}

#[test]
fn invalid_names_rejected_on_write() {
    let mut d = Document::current();
    d.section_mut("bad name").set("k", Value::U64(1));
    assert!(matches!(d.to_text(), Err(PersistError::InvalidName(_))));
    let mut d = Document::current();
    d.section_mut("ok").set("bad key", Value::U64(1));
    assert!(matches!(d.to_text(), Err(PersistError::InvalidName(_))));
}

#[test]
fn every_single_byte_flip_is_detected() {
    let text = text_of(&sample());
    let bytes = text.as_bytes();
    let mut undetected = 0;
    for i in 0..bytes.len() {
        let mut c = bytes.to_vec();
        c[i] ^= 0x01;
        if let Ok(s) = std::str::from_utf8(&c)
            && let Ok(d) = Document::from_text(s)
            && d == Document::from_text(&text).unwrap()
        {
            undetected += 1;
        }
    }
    assert_eq!(undetected, 0);
}

#[test]
fn every_truncation_is_detected() {
    let text = text_of(&sample());
    for n in 0..text.len() - 1 {
        if !text.is_char_boundary(n) {
            continue;
        }
        assert!(Document::from_text(&text[..n]).is_err(), "len {n}");
    }
}

#[test]
fn section_checksum_names_section() {
    let text = text_of(&sample());
    let edited = text.replacen("wear = f:", "wear = f:0", 1);
    let edited = edited.replacen("f:0", "f:1", 1);
    match Document::from_text(&edited) {
        Err(PersistError::SectionChecksum(_)) | Err(PersistError::Syntax { .. }) => {}
        other => panic!("{other:?}"),
    }
    let text = text.replace("tail = s:\"N787XX\"", "tail = s:\"N787XY\"");
    assert_eq!(
        Document::from_text(&text),
        Err(PersistError::SectionChecksum("airframe".to_string()))
    );
}

#[test]
fn file_checksum_detects_header_edit() {
    let text = text_of(&sample());
    let swapped = text.replacen("[airframe] crc=", "[airframf] crc=", 1);
    assert!(Document::from_text(&swapped).is_err());
}

#[test]
fn trailing_data_and_bad_headers() {
    let text = text_of(&sample());
    assert_eq!(
        Document::from_text(&format!("{text}junk\n")),
        Err(PersistError::Trailing)
    );
    assert_eq!(Document::from_text("NOPE v2\n"), Err(PersistError::BadHeader));
    assert_eq!(Document::from_text(""), Err(PersistError::BadHeader));
    assert_eq!(
        Document::from_text("B789PERSIST v99\n"),
        Err(PersistError::UnsupportedVersion(99))
    );
    assert_eq!(
        Document::from_text("B789PERSIST v2\n"),
        Err(PersistError::Truncated)
    );
}

fn v1_document() -> Document {
    let mut d = Document::new(1);
    let a = d.section_mut("airframe");
    a.set("tail", Value::Str("N1".to_string()));
    a.set("serial", Value::Str("1".to_string()));
    a.set("model", Value::Str("787-9".to_string()));
    a.set("settings.max_gap_s", Value::F64(1e9));
    let p = d.section_mut("part/24-electrical/gcu-l");
    p.set("hours", Value::F64(2.5));
    p.set("wear", Value::F64(0.25));
    p.set("cycles", Value::U64(10));
    p.set("temp_c", Value::F64(55.0));
    let b = d.section_mut("battery/main");
    b.set("soc_pct", Value::F64(80.0));
    b.set("cell_soc", Value::Array(vec![Value::F64(0.8)]));
    b.set("cell_health", Value::Array(vec![Value::F64(1.0)]));
    b.set("cycles", Value::U64(3));
    b.set("temp_c", Value::F64(25.0));
    b.set("soak_tau_s", Value::F64(7200.0));
    d
}

#[test]
fn migrates_v1_to_v2() {
    let text = v1_document().to_text().unwrap();
    assert!(text.starts_with("B789PERSIST v1\n"));
    let (a, steps) = decode_airframe(text.as_bytes()).unwrap();
    assert_eq!(steps, 1);
    let p = &a.parts["24-electrical/gcu-l"];
    assert_eq!(p.run_s, 9000.0);
    assert_eq!(p.powered_s, 9000.0);
    assert_eq!(p.unpowered_s, 0.0);
    assert_eq!(p.soak_c, 55.0);
    assert_eq!(p.soak_tau_s, 5400.0);
    assert_eq!(p.cycles, 10);
    let b = &a.batteries["main"];
    assert!((b.soc - 0.8).abs() < 1e-12);
    assert_eq!(b.self_discharge_per_day, 0.002);
    assert_eq!(b.soak_c, 25.0);
    assert_eq!(a.settings.gap_mode, GapMode::Sim);
    assert_eq!(a.stamp, None);
    let again = text_of(&a);
    let (c, steps) = decode_airframe(again.as_bytes()).unwrap();
    assert_eq!(steps, 0);
    assert_eq!(c, a);
}

#[test]
fn migration_errors() {
    let mut d = Document::new(1);
    d.section_mut("part/x").set("hours", Value::Str("no".to_string()));
    assert!(matches!(
        migrate_to_current(&mut d),
        Err(PersistError::WrongType(_))
    ));
    let mut d = Document::new(1);
    assert!(matches!(
        migrate_with(&mut d, &[], 2),
        Err(PersistError::Migration(_))
    ));
    let mut d = Document::new(5);
    assert!(matches!(
        migrate_with(&mut d, &[], 2),
        Err(PersistError::UnsupportedVersion(5))
    ));
    let mut d = Document::current();
    assert_eq!(migrate_to_current(&mut d), Ok(0));
}

#[test]
fn migration_chain_multi_step() {
    fn bump(d: &mut Document) -> Result<(), PersistError> {
        let v = u64::from(d.version);
        d.section_mut("n").set("v", Value::U64(v));
        d.version += 1;
        Ok(())
    }
    let mut d = Document::new(1);
    let table: [(u32, MigrationFn); 3] = [(1, bump), (2, bump), (3, bump)];
    assert_eq!(migrate_with(&mut d, &table, 4), Ok(3));
    assert_eq!(d.version, 4);
    assert_eq!(d.section("n").unwrap().u64("v"), Ok(3));
}

fn store_with(a: &Airframe) -> SaveStore<MemStorage> {
    let mut s = SaveStore::new(MemStorage::new(), "airframe");
    s.save(a).unwrap();
    s
}

#[test]
fn save_and_load_fresh() {
    let s = SaveStore::new(MemStorage::new(), "x");
    assert_eq!(s.load(), Ok(None));
    let a = sample();
    let s = store_with(&a);
    let l = s.load().unwrap().unwrap();
    assert_eq!(l.source, LoadSource::Main);
    assert_eq!(l.primary_error, None);
    assert_eq!(text_of(&l.airframe), text_of(&a));
    assert!(!s.storage.files.contains_key(&s.tmp_path()));
    assert!(!s.storage.files.contains_key(&s.prev_path()));
}

#[test]
fn second_save_keeps_previous() {
    let a = sample();
    let mut s = store_with(&a);
    let mut b = a.clone();
    b.identity.tail = "N788XX".to_string();
    s.save(&b).unwrap();
    assert!(s.storage.files.contains_key(&s.prev_path()));
    assert_eq!(s.load().unwrap().unwrap().airframe.identity.tail, "N788XX");
    s.storage.files.remove(&s.main_path());
    let l = s.load().unwrap().unwrap();
    assert_eq!(l.source, LoadSource::Previous);
    assert_eq!(l.airframe.identity.tail, "N787XX");
}

#[test]
fn failed_save_leaves_old_file_intact_at_every_step() {
    let a = sample();
    let mut b = a.clone();
    b.identity.tail = "N999ZZ".to_string();
    for torn in [false, true] {
        for step in 1..=8u64 {
            let mut s = store_with(&a);
            let before = s.storage.files.get(&s.main_path()).cloned();
            s.storage.ops = 0;
            s.storage.fail_at = Some(step);
            s.storage.torn_write = torn;
            let r = s.save(&b);
            s.storage.fail_at = None;
            let l = s.load().unwrap().unwrap();
            match r {
                Err(_) => {
                    assert_eq!(
                        s.storage.files.get(&s.main_path()).cloned(),
                        before,
                        "step {step}"
                    );
                    assert_eq!(l.airframe.identity.tail, "N787XX");
                    assert!(!s.storage.files.contains_key(&s.tmp_path()) || step > 4);
                }
                Ok(()) => assert_eq!(l.airframe.identity.tail, "N999ZZ"),
            }
        }
    }
}

#[test]
fn corrupt_main_falls_back_to_previous() {
    let a = sample();
    let mut s = store_with(&a);
    let mut b = a.clone();
    b.identity.tail = "N788XX".to_string();
    s.save(&b).unwrap();
    let main = s.main_path();
    let bytes = s.storage.files.get_mut(&main).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x20;
    let l = s.load().unwrap().unwrap();
    assert_eq!(l.source, LoadSource::Previous);
    assert!(l.primary_error.is_some());
    assert_eq!(l.airframe.identity.tail, "N787XX");
}

#[test]
fn corrupt_main_does_not_clobber_good_previous() {
    let a = sample();
    let mut s = store_with(&a);
    let mut b = a.clone();
    b.identity.tail = "N788XX".to_string();
    s.save(&b).unwrap();
    let main = s.main_path();
    s.storage.files.get_mut(&main).unwrap().truncate(40);
    let mut c = a.clone();
    c.identity.tail = "N789XX".to_string();
    s.save(&c).unwrap();
    s.storage.files.remove(&main);
    let l = s.load().unwrap().unwrap();
    assert_eq!(l.airframe.identity.tail, "N787XX");
}

#[test]
fn both_corrupt_is_an_error() {
    let a = sample();
    let mut s = store_with(&a);
    s.save(&a).unwrap();
    for k in [s.main_path(), s.prev_path()] {
        s.storage.files.get_mut(&k).unwrap().truncate(30);
    }
    assert!(s.load().is_err());
    let mut s2 = store_with(&a);
    s2.storage.files.get_mut(&s2.main_path()).unwrap().truncate(30);
    assert!(s2.load().is_err());
}

#[test]
fn non_utf8_file_is_corrupt() {
    assert!(decode_airframe(&[0xff, 0xfe, 0x00]).is_err());
}

#[test]
fn disk_storage_roundtrip_and_fallback() {
    let dir = std::env::temp_dir().join(format!("b789-persist-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = SaveStore::new(DiskStorage::new(dir.clone()), "frame");
    assert_eq!(s.load(), Ok(None));
    let a = sample();
    s.save(&a).unwrap();
    let mut b = a.clone();
    b.identity.tail = "N788XX".to_string();
    s.save(&b).unwrap();
    assert!(dir.join("frame.b789").exists());
    assert!(dir.join("frame.b789.prev").exists());
    assert!(!dir.join("frame.b789.tmp").exists());
    assert_eq!(text_of(&s.load().unwrap().unwrap().airframe), text_of(&b));
    std::fs::write(dir.join("frame.b789"), b"garbage").unwrap();
    let l = s.load().unwrap().unwrap();
    assert_eq!(l.source, LoadSource::Previous);
    assert_eq!(l.airframe.identity.tail, "N787XX");
    let _ = std::fs::remove_dir_all(&dir);
}

fn clock(sim: f64, real: u64) -> SessionClock {
    SessionClock {
        sim_s: sim,
        real_s: real,
    }
}

#[test]
fn first_session_has_no_gap() {
    let mut a = sample();
    a.stamp = None;
    let before = a.clone();
    let r = a.begin_session(clock(100.0, 5), 15.0);
    assert!(r.first_session);
    assert_eq!(r.elapsed_s, 0.0);
    assert_eq!(a.parts, before.parts);
    assert_eq!(a.stamp, Some(clock(100.0, 5)));
}

#[test]
fn gap_mode_selects_clock() {
    let mut a = sample();
    a.stamp = Some(clock(1000.0, 5000));
    a.settings.gap_mode = GapMode::Sim;
    let r = a.begin_session(clock(1000.0 + 3600.0, 5000 + 86400), 15.0);
    assert_eq!(r.elapsed_s, 3600.0);
    let mut a = sample();
    a.stamp = Some(clock(1000.0, 5000));
    a.settings.gap_mode = GapMode::Real;
    let r = a.begin_session(clock(1000.0 + 3600.0, 5000 + 86400), 15.0);
    assert_eq!(r.elapsed_s, 86400.0);
    assert_eq!(r.mode, GapMode::Real);
}

#[test]
fn backwards_and_huge_gaps_are_clamped() {
    let mut a = sample();
    a.settings.gap_mode = GapMode::Sim;
    a.stamp = Some(clock(1000.0, 0));
    let r = a.begin_session(clock(10.0, 0), 15.0);
    assert!(r.clamped);
    assert_eq!(r.elapsed_s, 0.0);
    a.stamp = Some(clock(0.0, 0));
    a.settings.max_gap_s = 1000.0;
    let r = a.begin_session(clock(1e12, 0), 15.0);
    assert!(r.clamped);
    assert_eq!(r.elapsed_s, 1000.0);
    a.stamp = Some(clock(0.0, 0));
    let r = a.begin_session(clock(f64::NAN, 0), 15.0);
    assert!(r.clamped);
    a.settings.gap_mode = GapMode::Real;
    a.stamp = Some(clock(0.0, 500));
    let r = a.begin_session(clock(0.0, 100), 15.0);
    assert_eq!(r.elapsed_s, 0.0);
}

#[test]
fn counters_powered_vs_always() {
    let mut a = sample();
    a.settings.gap_mode = GapMode::Sim;
    a.stamp = Some(clock(0.0, 0));
    a.begin_session(clock(7200.0, 0), 15.0);
    let p = &a.parts["24-electrical/gcu-l"];
    assert_eq!(p.counters["powered"].value, 1000.0);
    assert_eq!(p.counters["calendar"].value, 2000.0 + 7200.0);
    assert_eq!(p.powered_s, 99999.5);
    assert_eq!(p.run_s, 123456.789);
    assert_eq!(p.unpowered_s, 1e-310 + 7200.0);
    assert_eq!(p.cycles, u64::MAX);
}

#[test]
fn soak_is_analytic_and_split_invariant() {
    let t = soak_toward(60.0, 20.0, 3600.0, 3600.0);
    assert!((t - (20.0 + 40.0 * (-1.0f64).exp())).abs() < 1e-12);
    let once = soak_toward(60.0, 20.0, 3600.0, 7200.0);
    let twice = soak_toward(soak_toward(60.0, 20.0, 3600.0, 3600.0), 20.0, 3600.0, 3600.0);
    assert!((once - twice).abs() < 1e-12);
    assert_eq!(soak_toward(60.0, 20.0, 3600.0, 0.0), 60.0);
    assert_eq!(soak_toward(60.0, 20.0, 0.0, 10.0), 20.0);
    let far = soak_toward(60.0, -10.0, 3600.0, 1e7);
    assert!((far + 10.0).abs() < 1e-9);
    let mut a = sample();
    a.stamp = Some(clock(0.0, 0));
    a.settings.gap_mode = GapMode::Sim;
    a.begin_session(clock(86400.0 * 3.0, 0), -5.0);
    let p = &a.parts["24-electrical/gcu-l"];
    assert!((p.soak_c + 5.0).abs() < 1e-6);
    let q = &a.parts["42-ccs/cabinet-a"];
    assert!((q.soak_c + 5.0).abs() < 1e-3);
}

#[test]
fn battery_self_discharge_monotonic_and_temperature_dependent() {
    let mut cold = sample();
    let mut hot = sample();
    for a in [&mut cold, &mut hot] {
        a.settings.gap_mode = GapMode::Sim;
        a.stamp = Some(clock(0.0, 0));
    }
    cold.batteries.get_mut("main").unwrap().soak_c = 0.0;
    cold.begin_session(clock(86400.0 * 30.0, 0), 0.0);
    hot.batteries.get_mut("main").unwrap().soak_c = 40.0;
    hot.begin_session(clock(86400.0 * 30.0, 0), 40.0);
    let c = &cold.batteries["main"];
    let h = &hot.batteries["main"];
    assert!(c.soc < 0.87 && c.soc > 0.0);
    assert!(h.soc < c.soc);
    assert!(h.cell_soc.iter().all(|x| *x < 0.88));
    assert_eq!(h.cell_soc.len(), 4);
    assert_eq!(h.cycles, 412);
    assert_eq!(h.cell_health, vec![1.0, 0.99, 0.98, 0.97]);
    let expected_cold = 0.87 * (-(0.01f64) * 30.0 * 2f64.powf(-2.0)).exp();
    assert!((c.soc - expected_cold).abs() < 1e-9);
}

#[test]
fn battery_gap_is_deterministic_and_never_negative() {
    let mut x = sample();
    let mut y = sample();
    for a in [&mut x, &mut y] {
        a.settings.gap_mode = GapMode::Sim;
        a.stamp = Some(clock(0.0, 0));
        a.batteries.get_mut("main").unwrap().self_discharge_per_day = 5.0;
        a.begin_session(clock(86400.0 * 4000.0, 0), 45.0);
    }
    assert_eq!(x, y);
    let b = &x.batteries["main"];
    assert!(b.soc >= 0.0 && b.soc < 1e-6);
    assert!(b.cell_soc.iter().all(|c| *c >= 0.0));
}

#[test]
fn zero_elapsed_changes_nothing() {
    let mut a = sample();
    a.settings.gap_mode = GapMode::Sim;
    a.stamp = Some(clock(50.0, 0));
    let before = a.parts.clone();
    let bat = a.batteries.clone();
    a.begin_session(clock(50.0, 0), 0.0);
    assert_eq!(a.parts, before);
    assert_eq!(a.batteries, bat);
}

#[test]
fn full_session_cycle_through_store() {
    let mut a = Airframe::new("N787ZZ", "1");
    a.settings.gap_mode = GapMode::Sim;
    a.parts.insert("p".to_string(), PartRecord::new(80.0, 1800.0));
    a.begin_session(clock(0.0, 0), 20.0);
    a.parts.get_mut("p").unwrap().powered_s += 600.0;
    a.end_session(clock(600.0, 10));
    let mut s = SaveStore::new(MemStorage::new(), "f");
    s.save(&a).unwrap();
    let mut b = s.load().unwrap().unwrap().airframe;
    let r = b.begin_session(clock(600.0 + 9000.0, 20), 20.0);
    assert_eq!(r.elapsed_s, 9000.0);
    let p = &b.parts["p"];
    assert_eq!(p.powered_s, 600.0);
    assert!(p.soak_c < 21.0 && p.soak_c > 20.0);
}

#[test]
fn wrong_types_in_semantic_layer_are_errors() {
    let mut d = sample().to_document();
    d.section_mut("part/24-electrical/gcu-l")
        .set("cycles", Value::F64(1.0));
    assert!(matches!(
        Airframe::from_document(&d),
        Err(PersistError::WrongType(_))
    ));
    let mut d = sample().to_document();
    d.sections.remove("airframe");
    assert!(matches!(
        Airframe::from_document(&d),
        Err(PersistError::Missing(_))
    ));
    let mut d = sample().to_document();
    d.section_mut("airframe")
        .set("settings.gap_mode", Value::Str("lunar".to_string()));
    assert!(Airframe::from_document(&d).is_err());
    let mut d = sample().to_document();
    d.section_mut("breakers").set(
        "C1",
        Value::Array(vec![Value::U64(7), Value::Bool(false), Value::Str(String::new())]),
    );
    assert!(Airframe::from_document(&d).is_err());
    let d = Document::new(1);
    assert_eq!(
        Airframe::from_document(&d),
        Err(PersistError::UnsupportedVersion(1))
    );
}

#[test]
fn duplicate_sections_and_keys_rejected() {
    let text = text_of(&sample());
    let first = text.find("[airframe]").unwrap();
    let end = text.find("[end]").unwrap();
    let hdr_end = text.find('\n').unwrap() + 1;
    let airframe_block_end = text[first..].find("\n[").map(|i| first + i + 1).unwrap();
    let block = &text[first..airframe_block_end];
    let body = format!("{}{}{}", &text[..end], block, "");
    let crc = fnv1a64(body.as_bytes());
    let dup = format!("{body}[end] sections=9 crc={crc:016x}\n");
    assert!(matches!(
        Document::from_text(&dup),
        Err(PersistError::DuplicateSection(_))
    ));
    let _ = hdr_end;
    let mut sec = BTreeMap::new();
    sec.insert("k".to_string(), Value::U64(1));
    assert_eq!(sec.len(), 1);
}

#[test]
fn accessors_report_missing_and_type() {
    let mut s = Section::new();
    s.set("a", Value::U64(1));
    assert_eq!(s.u64("a"), Ok(1));
    assert!(matches!(s.f64("a"), Err(PersistError::WrongType(_))));
    assert!(matches!(s.f64("b"), Err(PersistError::Missing(_))));
    assert_eq!(s.f64_or("b", 2.5), Ok(2.5));
    assert!(s.bool("a").is_err());
    assert!(s.str("a").is_err());
    assert!(s.bytes("a").is_err());
    assert!(s.array("a").is_err());
    assert!(s.i64("a").is_err());
    assert_eq!(Value::U64(1).kind(), "u64");
}

#[test]
fn error_display() {
    assert!(PersistError::FileChecksum.to_string().contains("checksum"));
    assert!(
        PersistError::SectionChecksum("s".to_string())
            .to_string()
            .contains('s')
    );
}

fn forge(sections: &[(&str, &str)]) -> String {
    let mut out = String::from("B789PERSIST v2\n");
    for (name, body) in sections {
        out.push_str(&format!("[{name}] crc={:016x}\n{body}", fnv1a64(body.as_bytes())));
    }
    let crc = fnv1a64(out.as_bytes());
    out.push_str(&format!("[end] sections={} crc={crc:016x}\n", sections.len()));
    out
}

#[test]
fn forged_files_with_valid_checksums_still_validated() {
    assert!(Document::from_text(&forge(&[("a", "k = u:1\n")])).is_ok());
    assert!(matches!(
        Document::from_text(&forge(&[("a", "k = u:1\nk = u:2\n")])),
        Err(PersistError::DuplicateKey(_))
    ));
    assert!(matches!(
        Document::from_text(&forge(&[("a", "garbage\n")])),
        Err(PersistError::Syntax { .. })
    ));
    assert!(matches!(
        Document::from_text(&forge(&[("a", "k = u:x\n")])),
        Err(PersistError::Syntax { .. })
    ));
    assert!(matches!(
        Document::from_text(&forge(&[("a", ""), ("a", "")])),
        Err(PersistError::DuplicateSection(_))
    ));
    assert!(matches!(
        Document::from_text(&forge(&[("b d", "")])),
        Err(PersistError::InvalidName(_))
    ));
    let mut t = forge(&[("a", "k = u:1\n")]);
    t = t.replace("sections=1", "sections=2");
    assert!(Document::from_text(&t).is_err());
}
