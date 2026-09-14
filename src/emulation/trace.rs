//! Logic-analyzer-style trace capture for deterministic regression tests.

use super::{LogicLevel, Tick};

/// One resolved net observation at a specific emulated tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceSample<N> {
    pub tick: Tick,
    pub net: N,
    pub level: LogicLevel,
}

/// Append-only trace used to compare emulator waveforms with golden captures.
#[derive(Debug, Clone, Default)]
pub struct Trace<N> {
    samples: Vec<TraceSample<N>>,
}

impl<N> Trace<N> {
    pub fn push(&mut self, sample: TraceSample<N>) {
        self.samples.push(sample);
    }

    pub fn samples(&self) -> &[TraceSample<N>] {
        &self.samples
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}
