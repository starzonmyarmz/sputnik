use std::f64::consts::{LN_2, PI};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Sputter,
    Synth,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Division {
    Quarter,
    Eighth,
    EighthTriplet,
    Sixteenth,
    SixteenthTriplet,
}

impl Division {
    pub fn beats(self) -> f64 {
        match self {
            Division::Quarter => 1.0,
            Division::Eighth => 0.5,
            Division::EighthTriplet => 1.0 / 3.0,
            Division::Sixteenth => 0.25,
            Division::SixteenthTriplet => 1.0 / 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub mode: Mode,
    /// 0 to 1.
    pub fuzz: f64,
    /// 0 to 1: how much of a decaying note is cut, from none to most.
    pub gate: f64,
    /// 0 to 1: the tracked oscillator's share of the mix.
    pub synth: f64,
    /// 0 to 1: stutter depth in Sputter, filter sweep depth in Synth.
    pub movement: f64,
    pub level_db: f64,
    /// 0 to 1, Sputter only.
    pub sag: f64,
    /// Synth only.
    pub cutoff_hz: f64,
    /// 0 to 1, Synth only.
    pub resonance: f64,
    /// 0 to 1, where 0 is off.
    pub crush: f64,
    pub division: Division,
    /// Below this the signal passes clean.
    pub crossover_hz: f64,
    pub stomp: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Sputter,
            fuzz: 0.6,
            gate: 0.3,
            synth: 0.0,
            movement: 0.0,
            level_db: 0.0,
            sag: 0.3,
            cutoff_hz: 400.0,
            resonance: 0.4,
            crush: 0.0,
            division: Division::Sixteenth,
            crossover_hz: 80.0,
            stomp: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Transport {
    pub tempo: Option<f64>,
    pub beats: Option<f64>,
    pub playing: bool,
}

const FALLBACK_TEMPO: f64 = 120.0;
/// Fuzz output level at 0 dB Level, near a guitar DI's peaks.
const WET_GAIN: f64 = 0.3;
const SPUTTER_CUTOFF: f64 = 5500.0;
const SPUTTER_Q: f64 = 0.7;
const SWEEP_OCTAVES: f64 = 5.0;
/// Input level (about -10 dBFS) that opens the sweep and the oscillator fully.
const ENV_FULL: f64 = 0.3;

struct Smoothed {
    value: f64,
    target: f64,
}

impl Smoothed {
    fn new(value: f64) -> Self {
        Self { value, target: value }
    }

    fn next(&mut self, a: f64) -> f64 {
        self.value += (self.target - self.value) * a;
        self.value
    }

    fn snap(&mut self) {
        self.value = self.target;
    }
}

pub struct Engine {
    sample_rate: f64,
    settings: Settings,

    param_a: f64,
    stomp_a: f64,
    chop_a: f64,
    gate_att: f64,
    gate_rel: f64,
    env_rel: f64,
    sag_att: f64,
    sag_rel: f64,
    det_a: f64,
    det_hp: f64,
    min_period: f64,
    max_period: f64,
    dc_r: f64,

    fuzz: Smoothed,
    gate: Smoothed,
    synth: Smoothed,
    movement: Smoothed,
    level: Smoothed,
    sag: Smoothed,
    cutoff_log: Smoothed,
    resonance: Smoothed,
    crossover: Smoothed,
    sputter: Smoothed,
    stomp: Smoothed,

    lo1: f64,
    lo2: f64,
    env: f64,
    sag_env: f64,
    gate_open: f64,
    x1: f64,

    det1: f64,
    det2: f64,
    det_x1: f64,
    det_y: f64,
    det_env: f64,
    armed: bool,
    since: f64,
    period: f64,
    phase: f64,

    ic1: f64,
    ic2: f64,
    hold: f64,
    held: f64,
    beat: f64,
    beats_per_sample: f64,
    chop: f64,
    dc_x1: f64,
    dc_y1: f64,
}

impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let coef = |seconds: f64| 1.0 - (-1.0 / (sr * seconds)).exp();
        let s = Settings::default();
        let mut engine = Self {
            sample_rate: sr,
            settings: s,
            param_a: coef(0.01),
            stomp_a: coef(0.02),
            chop_a: coef(0.001),
            gate_att: coef(0.001),
            gate_rel: coef(0.02),
            env_rel: (-1.0 / (sr * 0.15)).exp(),
            sag_att: coef(0.005),
            sag_rel: coef(0.2),
            det_a: (-2.0 * PI * 900.0 / sr).exp(),
            det_hp: (-2.0 * PI * 25.0 / sr).exp(),
            min_period: (sr / 2000.0).max(2.0),
            max_period: sr / 30.0,
            dc_r: (-2.0 * PI * 10.0 / sr).exp(),
            fuzz: Smoothed::new(0.0),
            gate: Smoothed::new(0.0),
            synth: Smoothed::new(0.0),
            movement: Smoothed::new(0.0),
            level: Smoothed::new(1.0),
            sag: Smoothed::new(0.0),
            cutoff_log: Smoothed::new(0.0),
            resonance: Smoothed::new(0.0),
            crossover: Smoothed::new(0.0),
            sputter: Smoothed::new(1.0),
            stomp: Smoothed::new(1.0),
            lo1: 0.0,
            lo2: 0.0,
            env: 0.0,
            sag_env: 0.0,
            gate_open: 0.0,
            x1: 0.0,
            det1: 0.0,
            det2: 0.0,
            det_x1: 0.0,
            det_y: 0.0,
            det_env: 0.0,
            armed: false,
            since: 0.0,
            period: sr / 110.0,
            phase: 0.0,
            ic1: 0.0,
            ic2: 0.0,
            hold: 0.0,
            held: 0.0,
            beat: 0.0,
            beats_per_sample: FALLBACK_TEMPO / 60.0 / sr,
            chop: 1.0,
            dc_x1: 0.0,
            dc_y1: 0.0,
        };
        engine.set(s);
        engine.reset();
        engine
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate as f32
    }

    /// Clears the signal state and jumps every control to its setting.
    pub fn reset(&mut self) {
        for p in [
            &mut self.fuzz,
            &mut self.gate,
            &mut self.synth,
            &mut self.movement,
            &mut self.level,
            &mut self.sag,
            &mut self.cutoff_log,
            &mut self.resonance,
            &mut self.crossover,
            &mut self.sputter,
            &mut self.stomp,
        ] {
            p.snap();
        }
        self.lo1 = 0.0;
        self.lo2 = 0.0;
        self.env = 0.0;
        self.sag_env = 0.0;
        self.gate_open = 0.0;
        self.x1 = 0.4 * self.sag.value * self.sputter.value;
        self.det1 = 0.0;
        self.det2 = 0.0;
        self.det_x1 = 0.0;
        self.det_y = 0.0;
        self.det_env = 0.0;
        self.armed = false;
        self.since = 0.0;
        self.period = self.sample_rate / 110.0;
        self.phase = 0.0;
        self.ic1 = 0.0;
        self.ic2 = 0.0;
        self.hold = 0.0;
        self.held = 0.0;
        self.beat = 0.0;
        self.chop = 1.0;
        self.dc_x1 = 0.0;
        self.dc_y1 = 0.0;
    }

    pub fn set(&mut self, s: Settings) {
        let nyquist = self.sample_rate * 0.45;
        self.fuzz.target = s.fuzz;
        self.gate.target = s.gate;
        self.synth.target = s.synth;
        self.movement.target = s.movement;
        self.level.target = 10f64.powf(s.level_db / 20.0);
        self.sag.target = s.sag;
        self.cutoff_log.target = s.cutoff_hz.clamp(20.0, nyquist).ln();
        self.resonance.target = s.resonance;
        self.crossover.target = 1.0 - (-2.0 * PI * s.crossover_hz.min(nyquist) / self.sample_rate).exp();
        self.sputter.target = if s.mode == Mode::Sputter { 1.0 } else { 0.0 };
        self.stomp.target = if s.stomp { 1.0 } else { 0.0 };
        self.settings = s;
    }

    /// Call once per block, before `process`. While the host is playing the
    /// stutter locks to its beat position; otherwise it runs free at the
    /// host's tempo, or 120 BPM without one.
    pub fn set_transport(&mut self, t: Transport) {
        let tempo = t.tempo.filter(|t| *t > 0.0).unwrap_or(FALLBACK_TEMPO);
        self.beats_per_sample = tempo / 60.0 / self.sample_rate;
        if let (true, Some(beats)) = (t.playing, t.beats) {
            self.beat = beats;
        }
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let (out_l, out_r) = self.tick(*l as f64, *r as f64);
            *l = out_l as f32;
            *r = out_r as f32;
        }
    }

    fn tick(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let a = self.param_a;
        let fuzz = self.fuzz.next(a);
        let gate = self.gate.next(a);
        let synth = self.synth.next(a);
        let movement = self.movement.next(a);
        let level = self.level.next(a);
        let sputter = self.sputter.next(a);
        let sag = self.sag.next(a) * sputter;
        let cutoff_log = self.cutoff_log.next(a);
        let resonance = self.resonance.next(a);
        let crossover = self.crossover.next(a);
        let stomp = self.stomp.next(self.stomp_a);

        let mono = (in_l + in_r) * 0.5;
        self.lo1 += (mono - self.lo1) * crossover;
        self.lo2 += (self.lo1 - self.lo2) * crossover;
        let low = self.lo2;
        let high = mono - low;

        let mag = high.abs();
        self.env = mag.max(self.env * self.env_rel);
        let sag_coef = if mag > self.sag_env { self.sag_att } else { self.sag_rel };
        self.sag_env += (mag - self.sag_env) * sag_coef;

        // A dead zone that Gate sets and Sag pushes up with playing level:
        // as a note decays only its peaks get through, so it sputters out.
        let threshold = gate * gate * 0.08 + sag * 0.6 * self.sag_env;
        let dz = high.signum() * (mag - threshold).max(0.0);
        let open = if self.env > threshold { 1.0 } else { 0.0 };
        let gate_coef = if open > self.gate_open { self.gate_att } else { self.gate_rel };
        self.gate_open += (open - self.gate_open) * gate_coef;

        let drive = 10f64.powf((12.0 + 42.0 * fuzz) / 20.0);
        let bias = 0.4 * sag;
        let x = dz * drive + bias;
        let rail = 1.0 - 0.5 * sag * (self.sag_env * 4.0).min(1.0);
        let fuzzed = (adaa_tanh(x, self.x1) - bias.tanh()) * rail;
        self.x1 = x;

        let osc = self.oscillator(high, sputter) * (self.det_env / ENV_FULL).min(1.0) * self.gate_open;
        let mixed = fuzzed * (1.0 - synth) + osc * synth;

        let sweep = movement * (1.0 - sputter) * (self.env / ENV_FULL).min(1.0);
        let synth_cutoff = (cutoff_log + sweep * SWEEP_OCTAVES * LN_2).exp();
        let synth_q = 0.6 + resonance * resonance * 14.0;
        let cutoff = lerp(synth_cutoff.ln(), SPUTTER_CUTOFF.ln(), sputter).exp().min(self.sample_rate * 0.45);
        let q = lerp(synth_q, SPUTTER_Q, sputter);
        let filtered = self.svf(mixed, cutoff, q).tanh();

        let crushed = self.crush(filtered);

        let length = self.settings.division.beats();
        let on = (self.beat / length).rem_euclid(1.0) < 0.5;
        self.beat += self.beats_per_sample;
        let chop_target = if on { 1.0 } else { 1.0 - movement * sputter };
        self.chop += (chop_target - self.chop) * self.chop_a;

        let blocked = crushed - self.dc_x1 + self.dc_r * self.dc_y1;
        self.dc_x1 = crushed;
        self.dc_y1 = blocked;

        let wet = (blocked * WET_GAIN + low) * self.chop * level;
        self.flush();
        (lerp(in_l, wet, stomp), lerp(in_r, wet, stomp))
    }

    /// A zero-crossing tracker with no octave correction: chords, slides and
    /// noise make it jump, and that's part of the sound.
    fn oscillator(&mut self, input: f64, sputter: f64) -> f64 {
        self.det1 = self.det1 * self.det_a + input * (1.0 - self.det_a);
        self.det2 = self.det2 * self.det_a + self.det1 * (1.0 - self.det_a);
        self.det_y = self.det_hp * (self.det_y + self.det2 - self.det_x1);
        self.det_x1 = self.det2;
        self.det_env = self.det_y.abs().max(self.det_env * self.env_rel);
        let hyst = self.det_env * 0.25;
        if self.det_y < -hyst {
            self.armed = true;
        }
        self.since += 1.0;
        if self.armed && self.det_y > hyst {
            self.armed = false;
            let since = self.since.clamp(self.min_period, self.max_period);
            self.period = self.period * 0.5 + since * 0.5;
            self.since = 0.0;
        }

        let dt = 1.0 / self.period;
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let t = self.phase;
        let saw = 2.0 * t - 1.0 - poly_blep(t, dt);
        let half = (t + 0.5).rem_euclid(1.0);
        let square = (if t < 0.5 { 1.0 } else { -1.0 }) + poly_blep(t, dt) - poly_blep(half, dt);
        lerp(saw, square, sputter)
    }

    /// Topology-preserving state-variable low-pass.
    fn svf(&mut self, x: f64, cutoff: f64, q: f64) -> f64 {
        let g = (PI * cutoff / self.sample_rate).tan();
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let v3 = x - self.ic2;
        let v1 = a1 * self.ic1 + a2 * v3;
        let v2 = self.ic2 + a2 * self.ic1 + a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        v2
    }

    fn crush(&mut self, x: f64) -> f64 {
        let crush = self.settings.crush;
        if crush <= 0.0 {
            return x;
        }
        self.hold -= 1.0;
        if self.hold <= 0.0 {
            self.hold += 1.0 + crush * crush * 40.0;
            let steps = 2f64.powf(15.0 - 12.0 * crush);
            self.held = (x * steps).round() / steps;
        }
        self.held
    }

    fn flush(&mut self) {
        for v in [
            &mut self.lo1,
            &mut self.lo2,
            &mut self.env,
            &mut self.sag_env,
            &mut self.gate_open,
            &mut self.det1,
            &mut self.det2,
            &mut self.det_x1,
            &mut self.det_y,
            &mut self.det_env,
            &mut self.ic1,
            &mut self.ic2,
            &mut self.dc_x1,
            &mut self.dc_y1,
        ] {
            if v.abs() < 1e-30 {
                *v = 0.0;
            }
        }
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn ln_cosh(x: f64) -> f64 {
    let a = x.abs();
    a + (-2.0 * a).exp().ln_1p() - LN_2
}

/// First-order antiderivative antialiasing of tanh, so the near-square fuzz
/// doesn't fold its harmonics back down.
fn adaa_tanh(x: f64, x1: f64) -> f64 {
    let d = x - x1;
    if d.abs() < 1e-5 {
        ((x + x1) * 0.5).tanh()
    } else {
        (ln_cosh(x) - ln_cosh(x1)) / d
    }
}

fn poly_blep(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}
