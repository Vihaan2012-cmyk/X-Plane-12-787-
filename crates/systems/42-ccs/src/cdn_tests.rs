use crate::cdn::{Cdn, DevKind, RATE_100M};
use crate::data::CcsData;
use crate::icd::{Dataset, Encoding, FS_NORMAL, crc32};
use crate::network;
use std::path::PathBuf;

fn net() -> Cdn {
    let d =
        CcsData::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aircraft/42-ccs")).unwrap();
    let mut c = network::build(&d.rdcs, &d.hosted, 3);
    for dev in c.devices.iter_mut() {
        dev.powered = true;
    }
    c
}

fn feed(c: &mut Cdn, until: u64) {
    let mut t = c.now_us;
    while t < until {
        t = (t + 10_000).min(until);
        let ids: Vec<u16> = c.vls.iter().map(|v| v.id).collect();
        for id in ids {
            let payload = vec![FS_NORMAL, (t / 10_000) as u8, 7, 9];
            c.publish(id, payload);
        }
        c.run_until(t);
    }
}

fn total_accepted(c: &Cdn, ch: usize) -> u64 {
    let mut n = 0;
    for v in &c.vls {
        for d in &v.dests {
            if let Some(r) = c.rx(*d, v.id) {
                n += r.accepted[ch];
            }
        }
    }
    n
}

#[test]
fn icd_round_trips_with_quantisation_and_saturation() {
    let ds = Dataset::new(
        "TEST",
        3,
        &[
            (
                "v",
                Encoding::Bnr {
                    bits: 12,
                    range: 64.0,
                    signed: false,
                },
            ),
            (
                "t",
                Encoding::Bnr {
                    bits: 12,
                    range: 128.0,
                    signed: true,
                },
            ),
            ("on", Encoding::Discrete),
        ],
    );
    let buf = ds.encode(&[("v", 28.04), ("t", -40.3), ("on", 1.0)], FS_NORMAL);
    let res = Encoding::Bnr {
        bits: 12,
        range: 64.0,
        signed: false,
    }
    .resolution();
    assert!((ds.decode(&buf, "v").unwrap() - 28.04).abs() <= res / 2.0 + 1e-12);
    assert!((ds.decode(&buf, "t").unwrap() + 40.3).abs() < 0.07);
    assert_eq!(ds.decode(&buf, "on"), Some(1.0));
    assert_eq!(Dataset::version_of(&buf), 3);
    let sat = ds.encode(&[("v", 500.0)], FS_NORMAL);
    assert!((ds.decode(&sat, "v").unwrap() - (64.0 - res)).abs() < 1e-9);
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
}

#[test]
fn every_vl_reaches_every_destination_on_both_channels_within_bound() {
    let mut c = net();
    feed(&mut c, 2_000_000);
    for v in &c.vls {
        for d in &v.dests {
            let r = c
                .rx(*d, v.id)
                .unwrap_or_else(|| panic!("{} never reached {}", v.name, c.devices[*d].name));
            assert!(r.accepted[0] + r.accepted[1] > 0, "{}", v.name);
            assert!(
                r.max_latency_us < 2_000,
                "{} latency {}",
                v.name,
                r.max_latency_us
            );
        }
    }
    assert_eq!(c.police_drops(), 0);
    assert_eq!(c.queue_drops(), 0);
    assert_eq!(c.unknown_vl_drops, 0);
}

#[test]
fn channel_a_switch_failure_is_hidden_by_channel_b() {
    let mut c = net();
    feed(&mut c, 1_000_000);
    let b0 = total_accepted(&c, 1);
    let acs = c.device("ACS L A").unwrap();
    c.devices[acs].failed = true;
    let ars = c.device("ARS 5").unwrap();
    c.devices[ars].failed = true;
    feed(&mut c, 2_000_000);
    let b1 = total_accepted(&c, 1);
    assert!(b1 > b0);
    for v in &c.vls {
        for d in &v.dests {
            let r = c.rx(*d, v.id).unwrap();
            assert!(
                r.rx_us.unwrap() + 200_000 > 2_000_000,
                "{} stale at {}",
                v.name,
                c.devices[*d].name
            );
        }
    }
}

#[test]
fn bit_errors_are_caught_by_crc_and_redundancy_hides_them() {
    let mut c = net();
    let fox_link = c
        .links
        .iter()
        .position(|l| {
            l.fibre
                && c.devices[l.a].kind == DevKind::Fox
                && c.devices[l.b].kind == DevKind::Fox
                && l.channel == 0
        })
        .unwrap();
    c.links[fox_link].ber = 1.0e-4;
    feed(&mut c, 2_000_000);
    assert!(c.links[fox_link].crc_errors > 10);
    let gpm = c.device("GPM R1").unwrap();
    let vl = c.vl_by_name("RDC 11 DATA").unwrap();
    assert!(c.rx(gpm, vl).unwrap().rx_us.unwrap() > 1_900_000);
}

#[test]
fn babbling_rdc_is_policed_and_others_keep_their_latency() {
    let mut c = net();
    let rdc = c.device("RDC 11").unwrap();
    c.devices[rdc].babbling = true;
    feed(&mut c, 2_000_000);
    assert!(c.police_drops() > 100);
    for v in &c.vls {
        if v.src == rdc {
            continue;
        }
        for d in &v.dests {
            let r = c.rx(*d, v.id).unwrap();
            assert!(
                r.max_latency_us < 2_000,
                "{} latency {}",
                v.name,
                r.max_latency_us
            );
        }
    }
}

#[test]
fn replaying_switches_on_both_channels_freeze_the_data() {
    let mut c = net();
    feed(&mut c, 1_000_000);
    for s in ["ARS 5", "ARS 6"] {
        let d = c.device(s).unwrap();
        c.devices[d].replay = true;
    }
    let gpm = c.device("GPM L1").unwrap();
    let vl = c.vl_by_name("RDC 11 DATA").unwrap();
    let before = c.rx(gpm, vl).unwrap().data[1];
    feed(&mut c, 2_000_000);
    let r = c.rx(gpm, vl).unwrap();
    assert!(
        r.data[1].wrapping_sub(before) <= 3,
        "content moved on: {} -> {}",
        before,
        r.data[1]
    );
    assert!(r.seq_errors + r.duplicates > 10);
}

#[test]
fn corruption_before_crc_delivers_plausible_wrong_data() {
    let mut c = net();
    let rdc = c.device("RDC 11").unwrap();
    c.devices[rdc].corrupt_before_crc = true;
    feed(&mut c, 500_000);
    let gpm = c.device("GPM L1").unwrap();
    let vl = c.vl_by_name("RDC 11 DATA").unwrap();
    let r = c.rx(gpm, vl).unwrap();
    assert_eq!(r.data[3], 9 ^ 0x01);
    assert!(c.links.iter().all(|l| l.crc_errors == 0));
}

#[test]
fn queue_overflow_drops_low_priority_first() {
    let mut c = Cdn::new(1);
    let a = c.add_device("A", DevKind::EndSystem);
    let s = c.add_device("S", DevKind::Switch);
    let b = c.add_device("B", DevKind::EndSystem);
    c.connect(a, s, RATE_100M, false, 0);
    let slow = c.connect(s, b, 1.0e5, false, 0);
    for d in c.devices.iter_mut() {
        d.powered = true;
    }
    let mut ids = Vec::new();
    for k in 0..80 {
        ids.push(c.add_vl(&format!("V{k}"), a, vec![b], 1_000, 1500, k < 5));
    }
    c.build_routes();
    for id in &ids {
        c.publish(*id, vec![FS_NORMAL; 1400]);
    }
    for k in 1..=40u64 {
        for id in &ids {
            c.publish(*id, vec![FS_NORMAL; 1400]);
        }
        c.run_until(k * 1_000);
    }
    assert!(c.queue(s, slow).unwrap().drops > 0);
    assert!(c.queue(s, slow).unwrap().max_depth <= crate::cdn::APPROX_QUEUE_FRAMES);
}

#[test]
fn topology_respects_switch_port_counts() {
    use crate::params::{ACS_FAST_PORTS, ACS_PORTS, ARS_FIBRE_PORTS, ARS_PORTS};
    let c = net();
    for d in &c.devices {
        if d.kind != DevKind::Switch {
            continue;
        }
        let fibre = d.ports.iter().filter(|l| c.links[**l].fibre).count();
        let fast = d
            .ports
            .iter()
            .filter(|l| c.links[**l].rate_bps >= RATE_100M)
            .count();
        if d.name.starts_with("ACS") {
            assert!(d.ports.len() <= ACS_PORTS, "{}", d.name);
            assert!(fast <= ACS_FAST_PORTS, "{}", d.name);
            assert_eq!(fibre, 0, "{} has no fibre ports", d.name);
        } else {
            assert!(d.ports.len() <= ARS_PORTS, "{}", d.name);
            assert!(fibre <= ARS_FIBRE_PORTS, "{} fibre {}", d.name, fibre);
        }
    }
}
