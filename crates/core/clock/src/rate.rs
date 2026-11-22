use crate::time::{SimDuration, SimTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RateId(u32);

impl RateId {
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick {
    pub time: SimTime,
    pub priority: u32,
    pub rate: RateId,
    pub index: u64,
}

#[derive(Debug, Clone)]
struct RateGroup {
    period: SimDuration,
    phase: SimDuration,
    priority: u32,
    next_index: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RateTable {
    groups: Vec<RateGroup>,
}

impl RateTable {
    pub fn new() -> Self {
        RateTable { groups: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn register(&mut self, period: SimDuration, phase: SimDuration, priority: u32) -> Option<RateId> {
        if period.is_zero() {
            return None;
        }
        let id = RateId(u32::try_from(self.groups.len()).ok()?);
        self.groups.push(RateGroup {
            period,
            phase,
            priority,
            next_index: 0,
        });
        Some(id)
    }

    pub fn period(&self, id: RateId) -> Option<SimDuration> {
        self.groups.get(id.0 as usize).map(|g| g.period)
    }

    pub fn phase(&self, id: RateId) -> Option<SimDuration> {
        self.groups.get(id.0 as usize).map(|g| g.phase)
    }

    pub fn priority(&self, id: RateId) -> Option<u32> {
        self.groups.get(id.0 as usize).map(|g| g.priority)
    }

    pub fn next_index(&self, id: RateId) -> Option<u64> {
        self.groups.get(id.0 as usize).map(|g| g.next_index)
    }

    pub fn tick_time(&self, id: RateId, n: u64) -> Option<SimTime> {
        let g = self.groups.get(id.0 as usize)?;
        let off = g.period.checked_mul(n)?.checked_add(g.phase)?;
        SimTime::ZERO.checked_add(off)
    }

    pub fn first_tick_at_or_after(&self, id: RateId, t: SimTime) -> Option<u64> {
        let g = self.groups.get(id.0 as usize)?;
        let ph = g.phase.as_micros();
        let tm = t.as_micros();
        if tm <= ph {
            return Some(0);
        }
        let p = g.period.as_micros();
        Some((tm - ph).div_ceil(p))
    }

    pub fn count_in(&self, id: RateId, from: SimTime, to: SimTime) -> Option<u64> {
        let a = self.first_tick_at_or_after(id, from)?;
        let b = self.first_tick_at_or_after(id, to)?;
        Some(b.saturating_sub(a))
    }

    pub fn collect_until(&mut self, t: SimTime) -> Vec<Tick> {
        let mut out = Vec::new();
        for (i, g) in self.groups.iter_mut().enumerate() {
            let rate = RateId(i as u32);
            loop {
                let time = g
                    .period
                    .checked_mul(g.next_index)
                    .and_then(|d| d.checked_add(g.phase))
                    .and_then(|d| SimTime::ZERO.checked_add(d));
                match time {
                    Some(time) if time <= t => {
                        out.push(Tick {
                            time,
                            priority: g.priority,
                            rate,
                            index: g.next_index,
                        });
                        g.next_index += 1;
                    }
                    _ => break,
                }
            }
        }
        out.sort_by_key(|k| (k.time, k.priority, k.rate, k.index));
        out
    }

    pub fn seek(&mut self, id: RateId, t: SimTime) -> bool {
        match self.first_tick_at_or_after(id, t) {
            Some(n) => {
                self.groups[id.0 as usize].next_index = n;
                true
            }
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub partition: u32,
    pub offset: SimDuration,
    pub duration: SimDuration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MajorFrameError {
    ZeroLength,
    ZeroDuration,
    ExceedsFrame,
    Overlap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowEventKind {
    Close,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowEvent {
    pub time: SimTime,
    pub kind: WindowEventKind,
    pub partition: u32,
    pub window: usize,
    pub frame: u64,
}

#[derive(Debug, Clone)]
pub struct MajorFrame {
    length: SimDuration,
    epoch: SimTime,
    windows: Vec<Window>,
}

impl MajorFrame {
    pub fn new(
        length: SimDuration,
        epoch: SimTime,
        mut windows: Vec<Window>,
    ) -> Result<Self, MajorFrameError> {
        if length.is_zero() {
            return Err(MajorFrameError::ZeroLength);
        }
        windows.sort_by_key(|w| (w.offset, w.partition));
        let mut prev_end = SimDuration::ZERO;
        for w in &windows {
            if w.duration.is_zero() {
                return Err(MajorFrameError::ZeroDuration);
            }
            let end = w
                .offset
                .checked_add(w.duration)
                .ok_or(MajorFrameError::ExceedsFrame)?;
            if end > length {
                return Err(MajorFrameError::ExceedsFrame);
            }
            if w.offset < prev_end {
                return Err(MajorFrameError::Overlap);
            }
            prev_end = end;
        }
        Ok(MajorFrame {
            length,
            epoch,
            windows,
        })
    }

    pub fn length(&self) -> SimDuration {
        self.length
    }

    pub fn epoch(&self) -> SimTime {
        self.epoch
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn idle_time(&self) -> SimDuration {
        let used = self
            .windows
            .iter()
            .fold(SimDuration::ZERO, |a, w| a.saturating_add(w.duration));
        self.length.saturating_sub(used)
    }

    pub fn frame_index(&self, t: SimTime) -> Option<u64> {
        let d = t.checked_duration_since(self.epoch)?;
        d.div_duration(self.length)
    }

    pub fn frame_start(&self, frame: u64) -> Option<SimTime> {
        self.epoch.checked_add(self.length.checked_mul(frame)?)
    }

    pub fn window_at(&self, t: SimTime) -> Option<(u64, usize, &Window)> {
        let frame = self.frame_index(t)?;
        let into = t.checked_duration_since(self.frame_start(frame)?)?;
        self.windows
            .iter()
            .enumerate()
            .find(|(_, w)| w.offset <= into && into < w.offset + w.duration)
            .map(|(i, w)| (frame, i, w))
    }

    pub fn events_in(&self, from: SimTime, to: SimTime) -> Vec<WindowEvent> {
        let mut out = Vec::new();
        if to <= from || to <= self.epoch {
            return out;
        }
        let first = self.frame_index(from).unwrap_or(0);
        let last = match self.frame_index(to) {
            Some(f) => f,
            None => return out,
        };
        for frame in first..=last {
            let Some(base) = self.frame_start(frame) else {
                break;
            };
            for (i, w) in self.windows.iter().enumerate() {
                let open = base.saturating_add(w.offset);
                let close = open.saturating_add(w.duration);
                if open >= from && open < to {
                    out.push(WindowEvent {
                        time: open,
                        kind: WindowEventKind::Open,
                        partition: w.partition,
                        window: i,
                        frame,
                    });
                }
                if close >= from && close < to {
                    out.push(WindowEvent {
                        time: close,
                        kind: WindowEventKind::Close,
                        partition: w.partition,
                        window: i,
                        frame,
                    });
                }
            }
        }
        out.sort_by_key(|e| (e.time, e.kind, e.window, e.frame));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn us(v: u64) -> SimDuration {
        SimDuration::from_micros(v)
    }

    fn t(v: u64) -> SimTime {
        SimTime::from_micros(v)
    }

    #[test]
    fn periodic_ticks_are_exact() {
        let mut r = RateTable::new();
        let a = r.register(us(1_000), us(0), 1).unwrap();
        let ticks = r.collect_until(t(5_000));
        assert_eq!(ticks.len(), 6);
        for (i, k) in ticks.iter().enumerate() {
            assert_eq!(k.time, t(i as u64 * 1_000));
            assert_eq!(k.index, i as u64);
            assert_eq!(k.rate, a);
        }
        assert!(r.collect_until(t(5_000)).is_empty());
        assert_eq!(r.collect_until(t(6_000)).len(), 1);
    }

    #[test]
    fn phase_offsets_first_tick() {
        let mut r = RateTable::new();
        let a = r.register(us(20_000), us(3_000), 0).unwrap();
        let ticks = r.collect_until(t(45_000));
        let times: Vec<_> = ticks.iter().map(|k| k.time.as_micros()).collect();
        assert_eq!(times, vec![3_000, 23_000, 43_000]);
        assert_eq!(r.tick_time(a, 2), Some(t(43_000)));
        assert_eq!(r.phase(a), Some(us(3_000)));
        assert_eq!(r.period(a), Some(us(20_000)));
    }

    #[test]
    fn zero_period_rejected() {
        let mut r = RateTable::new();
        assert!(r.register(us(0), us(0), 0).is_none());
        assert!(r.is_empty());
    }

    #[test]
    fn simultaneous_ticks_order_by_priority_then_id() {
        let mut r = RateTable::new();
        let slow = r.register(us(10), us(0), 5).unwrap();
        let fast_hi = r.register(us(5), us(0), 1).unwrap();
        let fast_lo = r.register(us(5), us(0), 5).unwrap();
        let ticks = r.collect_until(t(10));
        let at10: Vec<_> = ticks.iter().filter(|k| k.time == t(10)).map(|k| k.rate).collect();
        assert_eq!(at10, vec![fast_hi, slow, fast_lo]);
        assert_eq!(r.len(), 3);
        assert_eq!(r.priority(slow), Some(5));
    }

    #[test]
    fn collect_in_pieces_equals_collect_at_once() {
        let build = || {
            let mut r = RateTable::new();
            r.register(us(1_000), us(0), 2).unwrap();
            r.register(us(2_500), us(100), 1).unwrap();
            r.register(us(7_000), us(0), 0).unwrap();
            r
        };
        let mut a = build();
        let whole = a.collect_until(t(1_000_000));
        let mut b = build();
        let mut pieces = Vec::new();
        let mut now = 0;
        while now < 1_000_000 {
            now += 16_667;
            pieces.extend(b.collect_until(t(now.min(1_000_000))));
        }
        assert_eq!(whole, pieces);
    }

    #[test]
    fn counting_and_seek() {
        let mut r = RateTable::new();
        let a = r.register(us(10), us(3), 0).unwrap();
        assert_eq!(r.first_tick_at_or_after(a, t(0)), Some(0));
        assert_eq!(r.first_tick_at_or_after(a, t(3)), Some(0));
        assert_eq!(r.first_tick_at_or_after(a, t(4)), Some(1));
        assert_eq!(r.first_tick_at_or_after(a, t(13)), Some(1));
        assert_eq!(r.count_in(a, t(0), t(33)), Some(3));
        assert_eq!(r.count_in(a, t(3), t(13)), Some(1));
        assert!(r.seek(a, t(100)));
        assert_eq!(r.next_index(a), Some(10));
        let ticks = r.collect_until(t(113));
        assert_eq!(ticks.len(), 2);
        assert_eq!(ticks[0].time, t(103));
    }

    fn frame() -> MajorFrame {
        MajorFrame::new(
            us(25_000),
            t(0),
            vec![
                Window {
                    partition: 2,
                    offset: us(10_000),
                    duration: us(5_000),
                },
                Window {
                    partition: 1,
                    offset: us(0),
                    duration: us(10_000),
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn major_frame_sorts_and_reports_idle() {
        let f = frame();
        assert_eq!(f.windows()[0].partition, 1);
        assert_eq!(f.idle_time(), us(10_000));
        assert_eq!(f.length(), us(25_000));
        assert_eq!(f.epoch(), t(0));
    }

    #[test]
    fn major_frame_validation() {
        let w = |o, d| Window {
            partition: 0,
            offset: us(o),
            duration: us(d),
        };
        assert_eq!(
            MajorFrame::new(us(0), t(0), vec![]).unwrap_err(),
            MajorFrameError::ZeroLength
        );
        assert_eq!(
            MajorFrame::new(us(10), t(0), vec![w(0, 0)]).unwrap_err(),
            MajorFrameError::ZeroDuration
        );
        assert_eq!(
            MajorFrame::new(us(10), t(0), vec![w(5, 6)]).unwrap_err(),
            MajorFrameError::ExceedsFrame
        );
        assert_eq!(
            MajorFrame::new(us(10), t(0), vec![w(0, 6), w(5, 2)]).unwrap_err(),
            MajorFrameError::Overlap
        );
        assert!(MajorFrame::new(us(10), t(0), vec![w(0, 5), w(5, 5)]).is_ok());
    }

    #[test]
    fn window_lookup() {
        let f = frame();
        assert_eq!(f.window_at(t(0)).unwrap().2.partition, 1);
        assert_eq!(f.window_at(t(9_999)).unwrap().2.partition, 1);
        assert_eq!(f.window_at(t(10_000)).unwrap().2.partition, 2);
        assert!(f.window_at(t(15_000)).is_none());
        let (frame_no, _, w) = f.window_at(t(25_000 * 4 + 12_000)).unwrap();
        assert_eq!(frame_no, 4);
        assert_eq!(w.partition, 2);
        assert_eq!(f.frame_index(t(50_000)), Some(2));
        assert_eq!(f.frame_start(3), Some(t(75_000)));
    }

    #[test]
    fn window_events_order_close_before_open() {
        let f = frame();
        let ev = f.events_in(t(0), t(50_000));
        let desc: Vec<_> = ev
            .iter()
            .map(|e| (e.time.as_micros(), e.kind, e.partition))
            .collect();
        assert_eq!(
            desc,
            vec![
                (0, WindowEventKind::Open, 1),
                (10_000, WindowEventKind::Close, 1),
                (10_000, WindowEventKind::Open, 2),
                (15_000, WindowEventKind::Close, 2),
                (25_000, WindowEventKind::Open, 1),
                (35_000, WindowEventKind::Close, 1),
                (35_000, WindowEventKind::Open, 2),
                (40_000, WindowEventKind::Close, 2),
            ]
        );
    }

    #[test]
    fn window_events_split_equals_whole() {
        let f = frame();
        let whole = f.events_in(t(0), t(500_000));
        let mut parts = Vec::new();
        let mut a = 0;
        while a < 500_000 {
            let b = (a + 7_777).min(500_000);
            parts.extend(f.events_in(t(a), t(b)));
            a = b;
        }
        assert_eq!(whole, parts);
    }

    #[test]
    fn epoch_shifts_frames() {
        let f = MajorFrame::new(
            us(100),
            t(1_000),
            vec![Window {
                partition: 7,
                offset: us(10),
                duration: us(20),
            }],
        )
        .unwrap();
        assert!(f.window_at(t(500)).is_none());
        assert_eq!(f.window_at(t(1_015)).unwrap().2.partition, 7);
        let ev = f.events_in(t(0), t(1_200));
        assert_eq!(ev.len(), 4);
        assert_eq!(ev[0].time, t(1_010));
    }
}
