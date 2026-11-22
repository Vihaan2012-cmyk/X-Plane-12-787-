use std::collections::BTreeMap;

use crate::time::{SimDuration, SimTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(u64);

impl EventId {
    pub const fn sequence(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fired<E> {
    pub id: EventId,
    pub time: SimTime,
    pub priority: u32,
    pub event: E,
}

type Key = (SimTime, u32, u64);

#[derive(Debug, Clone)]
pub struct Scheduler<E> {
    now: SimTime,
    next_seq: u64,
    queue: BTreeMap<Key, E>,
    index: BTreeMap<u64, (SimTime, u32)>,
    fired_count: u64,
}

impl<E> Default for Scheduler<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> Scheduler<E> {
    pub fn new() -> Self {
        Scheduler {
            now: SimTime::ZERO,
            next_seq: 0,
            queue: BTreeMap::new(),
            index: BTreeMap::new(),
            fired_count: 0,
        }
    }

    pub fn starting_at(now: SimTime) -> Self {
        let mut s = Self::new();
        s.now = now;
        s
    }

    pub fn now(&self) -> SimTime {
        self.now
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn fired_count(&self) -> u64 {
        self.fired_count
    }

    pub fn schedule(&mut self, at: SimTime, priority: u32, event: E) -> EventId {
        let time = if at < self.now { self.now } else { at };
        let seq = self.next_seq;
        self.next_seq += 1;
        self.queue.insert((time, priority, seq), event);
        self.index.insert(seq, (time, priority));
        EventId(seq)
    }

    pub fn schedule_after(&mut self, delay: SimDuration, priority: u32, event: E) -> EventId {
        let at = self.now.saturating_add(delay);
        self.schedule(at, priority, event)
    }

    pub fn cancel(&mut self, id: EventId) -> Option<E> {
        let (time, priority) = self.index.remove(&id.0)?;
        self.queue.remove(&(time, priority, id.0))
    }

    pub fn is_pending(&self, id: EventId) -> bool {
        self.index.contains_key(&id.0)
    }

    pub fn scheduled_time(&self, id: EventId) -> Option<SimTime> {
        self.index.get(&id.0).map(|(t, _)| *t)
    }

    pub fn peek(&self) -> Option<(SimTime, u32, EventId, &E)> {
        self.queue
            .iter()
            .next()
            .map(|(&(t, p, s), e)| (t, p, EventId(s), e))
    }

    pub fn peek_time(&self) -> Option<SimTime> {
        self.queue.keys().next().map(|k| k.0)
    }

    fn take_first(&mut self) -> Option<Fired<E>> {
        let (key, event) = self.queue.pop_first()?;
        self.index.remove(&key.2);
        self.fired_count += 1;
        Some(Fired {
            id: EventId(key.2),
            time: key.0,
            priority: key.1,
            event,
        })
    }

    pub fn pop_next(&mut self) -> Option<Fired<E>> {
        let fired = self.take_first()?;
        if fired.time > self.now {
            self.now = fired.time;
        }
        Some(fired)
    }

    pub fn pop_due(&mut self, t: SimTime) -> Option<Fired<E>> {
        if self.peek_time()? > t {
            return None;
        }
        self.pop_next()
    }

    pub fn advance_to(&mut self, t: SimTime) -> Vec<Fired<E>> {
        let mut out = Vec::new();
        while let Some(f) = self.pop_due(t) {
            out.push(f);
        }
        if t > self.now {
            self.now = t;
        }
        out
    }

    pub fn advance_by(&mut self, d: SimDuration) -> Vec<Fired<E>> {
        let t = self.now.saturating_add(d);
        self.advance_to(t)
    }

    pub fn finish_to(&mut self, t: SimTime) {
        if t > self.now {
            self.now = t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(us: u64) -> SimTime {
        SimTime::from_micros(us)
    }

    #[test]
    fn orders_by_time_then_priority_then_sequence() {
        let mut s = Scheduler::new();
        s.schedule(t(10), 5, "c");
        s.schedule(t(5), 9, "a");
        s.schedule(t(10), 1, "b");
        s.schedule(t(10), 5, "d");
        s.schedule(t(10), 5, "e");
        let order: Vec<_> = s.advance_to(t(100)).into_iter().map(|f| f.event).collect();
        assert_eq!(order, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn tie_break_is_insertion_order() {
        let mut s = Scheduler::new();
        for i in 0..100u32 {
            s.schedule(t(7), 0, i);
        }
        let got: Vec<_> = s.advance_to(t(7)).into_iter().map(|f| f.event).collect();
        assert_eq!(got, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn cancel_removes_and_reports() {
        let mut s = Scheduler::new();
        let a = s.schedule(t(1), 0, 'a');
        let b = s.schedule(t(2), 0, 'b');
        assert!(s.is_pending(a));
        assert_eq!(s.cancel(a), Some('a'));
        assert!(!s.is_pending(a));
        assert_eq!(s.cancel(a), None);
        assert_eq!(s.len(), 1);
        assert_eq!(s.scheduled_time(b), Some(t(2)));
        let fired = s.advance_to(t(10));
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].event, 'b');
        assert_eq!(s.cancel(b), None);
    }

    #[test]
    fn cancel_does_not_disturb_ties() {
        let mut s = Scheduler::new();
        let ids: Vec<_> = (0..5).map(|i| s.schedule(t(3), 0, i)).collect();
        s.cancel(ids[2]);
        let got: Vec<_> = s.advance_to(t(3)).into_iter().map(|f| f.event).collect();
        assert_eq!(got, vec![0, 1, 3, 4]);
    }

    #[test]
    fn ids_are_never_reused() {
        let mut s = Scheduler::new();
        let a = s.schedule(t(1), 0, 0);
        s.cancel(a);
        let b = s.schedule(t(1), 0, 1);
        assert_ne!(a, b);
        assert!(b > a);
    }

    #[test]
    fn peek_and_pop() {
        let mut s = Scheduler::new();
        assert!(s.peek().is_none());
        assert!(s.pop_next().is_none());
        s.schedule(t(9), 2, "x");
        s.schedule(t(4), 3, "y");
        let (time, prio, _, ev) = s.peek().unwrap();
        assert_eq!((time, prio, *ev), (t(4), 3, "y"));
        assert_eq!(s.peek_time(), Some(t(4)));
        let f = s.pop_next().unwrap();
        assert_eq!(f.event, "y");
        assert_eq!(s.now(), t(4));
        assert_eq!(s.fired_count(), 1);
    }

    #[test]
    fn advance_to_leaves_future_events() {
        let mut s = Scheduler::new();
        s.schedule(t(5), 0, 1);
        s.schedule(t(15), 0, 2);
        let f = s.advance_to(t(10));
        assert_eq!(f.len(), 1);
        assert_eq!(s.now(), t(10));
        assert_eq!(s.len(), 1);
        assert!(s.advance_to(t(10)).is_empty());
        assert!(s.advance_to(t(3)).is_empty());
        assert_eq!(s.now(), t(10));
    }

    #[test]
    fn event_exactly_at_boundary_fires() {
        let mut s = Scheduler::new();
        s.schedule(t(10), 0, ());
        assert_eq!(s.advance_to(t(10)).len(), 1);
    }

    #[test]
    fn past_schedule_clamps_to_now() {
        let mut s = Scheduler::starting_at(t(100));
        let id = s.schedule(t(5), 0, ());
        assert_eq!(s.scheduled_time(id), Some(t(100)));
        assert_eq!(s.advance_to(t(100)).len(), 1);
    }

    #[test]
    fn cascading_events_via_pop_due() {
        let mut s = Scheduler::new();
        s.schedule(t(1), 0, 0u32);
        let mut log = Vec::new();
        while let Some(f) = s.pop_due(t(10)) {
            log.push((f.time, f.event));
            if f.event < 5 {
                s.schedule(f.time.saturating_add(SimDuration::from_micros(2)), 0, f.event + 1);
            }
        }
        s.finish_to(t(10));
        assert_eq!(log.len(), 5);
        assert_eq!(log[4].0, t(9));
        assert_eq!(s.now(), t(10));
        assert_eq!(s.peek_time(), Some(t(11)));
    }

    #[test]
    fn schedule_after_uses_now() {
        let mut s = Scheduler::new();
        s.advance_to(t(50));
        let id = s.schedule_after(SimDuration::from_micros(7), 0, ());
        assert_eq!(s.scheduled_time(id), Some(t(57)));
        let f = s.advance_by(SimDuration::from_micros(7));
        assert_eq!(f.len(), 1);
        assert_eq!(s.now(), t(57));
    }

    fn run_million(seed: u64) -> (u64, u64) {
        let mut s = Scheduler::new();
        let mut x = seed;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut ids = Vec::new();
        for i in 0..1_000_000u64 {
            let r = next();
            let id = s.schedule(t(r % 50_000), (r >> 40) as u32 % 4, i);
            if i % 10 == 0 {
                ids.push(id);
            }
        }
        for id in ids.iter().step_by(3) {
            s.cancel(*id);
        }
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        let mut count = 0u64;
        let mut last = (SimTime::ZERO, 0u32, 0u64);
        for f in s.advance_to(t(1_000_000)) {
            let key = (f.time, f.priority, f.id.sequence());
            assert!(key >= last);
            last = key;
            hash = (hash ^ f.time.as_micros()).wrapping_mul(0x100_0000_01b3);
            hash = (hash ^ f.event).wrapping_mul(0x100_0000_01b3);
            count += 1;
        }
        (hash, count)
    }

    #[test]
    fn determinism_over_a_million_events() {
        let a = run_million(0x9e37_79b9_7f4a_7c15);
        let b = run_million(0x9e37_79b9_7f4a_7c15);
        assert_eq!(a, b);
        assert_eq!(a.1, 1_000_000 - 33_334);
        let c = run_million(12345);
        assert_ne!(a.0, c.0);
    }
}
