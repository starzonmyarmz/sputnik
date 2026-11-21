use serde_json::{json, Value};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

pub const POINTS: usize = 256;
const WINDOW_SECONDS: f32 = 0.03;
/// Without a trigger the scope captures anyway this often, so silence
/// shows a flat line instead of the last wave.
const FREE_RUN_SECONDS: f32 = 0.05;
const ARM_LEVEL: f32 = 0.002;

/// The latest captured frame of output, shared between the audio thread and
/// the editor. A frame can tear while the editor reads it; that only shows
/// as a flicker.
pub struct Scope {
    points: [AtomicU32; POINTS],
    frames: AtomicU64,
}

impl Scope {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { points: std::array::from_fn(|_| AtomicU32::new(0)), frames: AtomicU64::new(0) })
    }

    pub fn frames(&self) -> u64 {
        self.frames.load(Ordering::Acquire)
    }

    pub fn points(&self) -> [f32; POINTS] {
        std::array::from_fn(|i| f32::from_bits(self.points[i].load(Ordering::Relaxed)))
    }
}

/// An editor feed that sends each new frame as a `scope` message.
pub fn feed(scope: Arc<Scope>) -> Arc<dyn Fn() -> Option<Value> + Send + Sync> {
    let seen = AtomicU64::new(0);
    Arc::new(move || {
        let frames = scope.frames();
        if seen.swap(frames, Ordering::Relaxed) == frames {
            return None;
        }
        let points: Vec<f64> = scope.points().iter().map(|&p| (p as f64 * 1000.0).round() / 1000.0).collect();
        Some(json!({ "type": "scope", "points": points }))
    })
}

/// Captures from the audio thread like a triggered oscilloscope: each frame
/// starts at a rising zero crossing, so a steady note holds still.
pub struct ScopeWriter {
    scope: Arc<Scope>,
    step: usize,
    free_run: usize,
    count: usize,
    index: usize,
    capturing: bool,
    armed: bool,
    idle: usize,
}

impl ScopeWriter {
    pub fn new(scope: Arc<Scope>, sample_rate: f32) -> Self {
        let mut writer =
            Self { scope, step: 1, free_run: 1, count: 0, index: 0, capturing: false, armed: false, idle: 0 };
        writer.set_sample_rate(sample_rate);
        writer
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.step = ((sample_rate * WINDOW_SECONDS / POINTS as f32).round() as usize).max(1);
        self.free_run = (sample_rate * FREE_RUN_SECONDS) as usize;
    }

    pub fn push(&mut self, samples: &[f32]) {
        for &x in samples {
            if !self.capturing {
                self.idle += 1;
                if x < -ARM_LEVEL {
                    self.armed = true;
                }
                if (self.armed && x > 0.0) || self.idle >= self.free_run {
                    self.capturing = true;
                    self.armed = false;
                    self.index = 0;
                    self.count = 0;
                }
            }
            if self.capturing {
                if self.count == 0 {
                    self.scope.points[self.index].store(x.to_bits(), Ordering::Relaxed);
                    self.index += 1;
                    if self.index == POINTS {
                        self.capturing = false;
                        self.idle = 0;
                        self.scope.frames.fetch_add(1, Ordering::Release);
                    }
                }
                self.count = (self.count + 1) % self.step;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_start_on_a_rising_crossing() {
        let scope = Scope::new();
        let mut writer = ScopeWriter::new(scope.clone(), 48_000.0);
        let wave: Vec<f32> = (0..48_000).map(|i| (i as f32 * 110.0 / 48_000.0 * std::f32::consts::TAU + 1.0).sin()).collect();
        writer.push(&wave);
        assert!(scope.frames() > 10);
        let points = scope.points();
        assert!(points[0] > 0.0 && points[0] < 0.1, "starts at {}", points[0]);
        assert!(points[4] > points[0]);
    }

    #[test]
    fn silence_free_runs_to_a_flat_line() {
        let scope = Scope::new();
        let mut writer = ScopeWriter::new(scope.clone(), 48_000.0);
        writer.push(&[0.5; 2000]);
        writer.push(&vec![0.0; 48_000]);
        assert!(scope.frames() > 5);
        assert!(scope.points().iter().all(|&p| p == 0.0));
    }

    #[test]
    fn feed_sends_only_new_frames() {
        let scope = Scope::new();
        let mut writer = ScopeWriter::new(scope.clone(), 48_000.0);
        let feed = feed(scope);
        assert!(feed().is_none());
        writer.push(&vec![0.0; 4800]);
        assert!(feed().is_some());
        assert!(feed().is_none());
    }
}
