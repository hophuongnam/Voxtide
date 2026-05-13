use std::collections::VecDeque;

pub struct LatencyTracker {
    cap: usize,
    buf: VecDeque<u64>,
}

impl LatencyTracker {
    pub fn new(cap: usize) -> Self {
        Self { cap: cap.max(1), buf: VecDeque::with_capacity(cap.max(1)) }
    }

    pub fn observe(&mut self, ms: u64) {
        if self.buf.len() == self.cap { self.buf.pop_front(); }
        self.buf.push_back(ms);
    }

    pub fn median_ms(&self) -> Option<u64> {
        if self.buf.is_empty() { return None; }
        let mut s: Vec<u64> = self.buf.iter().copied().collect();
        s.sort_unstable();
        let n = s.len();
        Some(if n % 2 == 1 {
            s[n / 2]
        } else {
            (s[n / 2 - 1] + s[n / 2]) / 2
        })
    }
}
