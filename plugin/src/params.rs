use crate::engine::{Division, Mode, Settings};
use nice_plug::prelude::*;

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum ModeParam {
    Sputter,
    Synth,
}

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum RateParam {
    #[name = "1/4"]
    Quarter,
    #[name = "1/8"]
    Eighth,
    #[name = "1/8T"]
    EighthTriplet,
    #[name = "1/16"]
    Sixteenth,
    #[name = "1/16T"]
    SixteenthTriplet,
}

#[derive(Params)]
pub struct SputnikParams {
    #[id = "mode"]
    pub mode: EnumParam<ModeParam>,
    #[id = "fuzz"]
    pub fuzz: FloatParam,
    #[id = "gate"]
    pub gate: FloatParam,
    #[id = "synth"]
    pub synth: FloatParam,
    #[id = "move"]
    pub movement: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "sag"]
    pub sag: FloatParam,
    #[id = "cutoff"]
    pub cutoff: FloatParam,
    #[id = "resonance"]
    pub resonance: FloatParam,
    #[id = "crush"]
    pub crush: FloatParam,
    #[id = "rate"]
    pub rate: EnumParam<RateParam>,
    #[id = "crossover"]
    pub crossover: FloatParam,
    #[id = "stomp"]
    pub stomp: BoolParam,
}

fn percent(name: &str, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_unit("%")
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

fn hz(name: &str, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Skewed { min, max, factor: FloatRange::skew_factor(-2.0) })
        .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
        .with_string_to_value(formatters::s2v_f32_hz_then_khz())
}

impl Default for SputnikParams {
    fn default() -> Self {
        let s = Settings::default();
        Self {
            mode: EnumParam::new("Mode", ModeParam::Sputter),
            fuzz: percent("Fuzz", s.fuzz as f32),
            gate: percent("Gate", s.gate as f32),
            synth: percent("Synth", s.synth as f32),
            movement: percent("Move", s.movement as f32),
            level: FloatParam::new("Level", s.level_db as f32, FloatRange::Linear { min: -24.0, max: 12.0 })
                .with_step_size(0.1)
                .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_rounded(1)),
            sag: percent("Sag", s.sag as f32),
            cutoff: hz("Cutoff", s.cutoff_hz as f32, 60.0, 4000.0),
            resonance: percent("Resonance", s.resonance as f32),
            crush: percent("Crush", s.crush as f32),
            rate: EnumParam::new("Rate", RateParam::Sixteenth),
            crossover: hz("Crossover", s.crossover_hz as f32, 20.0, 400.0),
            stomp: BoolParam::new("Stomp", true),
        }
    }
}

impl SputnikParams {
    pub fn settings(&self) -> Settings {
        Settings {
            mode: match self.mode.value() {
                ModeParam::Sputter => Mode::Sputter,
                ModeParam::Synth => Mode::Synth,
            },
            fuzz: self.fuzz.value() as f64,
            gate: self.gate.value() as f64,
            synth: self.synth.value() as f64,
            movement: self.movement.value() as f64,
            level_db: self.level.value() as f64,
            sag: self.sag.value() as f64,
            cutoff_hz: self.cutoff.value() as f64,
            resonance: self.resonance.value() as f64,
            crush: self.crush.value() as f64,
            division: match self.rate.value() {
                RateParam::Quarter => Division::Quarter,
                RateParam::Eighth => Division::Eighth,
                RateParam::EighthTriplet => Division::EighthTriplet,
                RateParam::Sixteenth => Division::Sixteenth,
                RateParam::SixteenthTriplet => Division::SixteenthTriplet,
            },
            crossover_hz: self.crossover.value() as f64,
            stomp: self.stomp.value(),
        }
    }
}
