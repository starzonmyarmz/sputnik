use crate::engine::{Division, Mode, Settings};
use serde_json::{json, Value};

pub struct Preset {
    pub name: &'static str,
    pub settings: Settings,
}

const BASE: Settings = Settings {
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
};

/// The factory presets, in the order the editor steps through them. Names
/// are what the panel shows, so keep them short.
pub const PRESETS: &[Preset] = &[
    Preset { name: "Init", settings: BASE },
    Preset {
        name: "Machine Gun",
        settings: Settings { fuzz: 0.8, gate: 0.6, movement: 1.0, sag: 0.2, level_db: -2.0, ..BASE },
    },
    Preset {
        name: "Dying Battery",
        settings: Settings { fuzz: 0.9, gate: 0.9, sag: 1.0, crush: 0.4, level_db: 1.0, ..BASE },
    },
    Preset {
        name: "Blown Speaker",
        settings: Settings { fuzz: 1.0, gate: 0.1, sag: 0.5, crush: 0.2, level_db: -4.0, ..BASE },
    },
    Preset {
        name: "Triplet Stutter",
        settings: Settings { fuzz: 0.7, gate: 0.3, movement: 0.9, sag: 0.1, division: Division::EighthTriplet, ..BASE },
    },
    Preset {
        name: "Bass Rocket",
        settings: Settings {
            mode: Mode::Synth,
            fuzz: 0.5,
            gate: 0.2,
            synth: 0.6,
            movement: 0.6,
            cutoff_hz: 220.0,
            resonance: 0.5,
            crossover_hz: 140.0,
            ..BASE
        },
    },
    Preset {
        name: "Wah Lead",
        settings: Settings {
            mode: Mode::Synth,
            fuzz: 0.7,
            gate: 0.1,
            synth: 0.3,
            movement: 0.9,
            cutoff_hz: 900.0,
            resonance: 0.7,
            level_db: -2.0,
            ..BASE
        },
    },
    Preset {
        name: "Static Choir",
        settings: Settings {
            mode: Mode::Synth,
            fuzz: 0.4,
            gate: 0.5,
            synth: 0.8,
            movement: 0.3,
            cutoff_hz: 1800.0,
            resonance: 0.3,
            crush: 0.7,
            level_db: -3.0,
            ..BASE
        },
    },
];

/// The presets as the page wants them: names and plain values keyed by
/// parameter ID, normalized by `normalize`.
pub fn describe(normalize: impl Fn(&Settings) -> Vec<(&'static str, f32)>) -> Value {
    PRESETS
        .iter()
        .map(|p| {
            let values: serde_json::Map<String, Value> =
                normalize(&p.settings).into_iter().map(|(id, v)| (id.to_string(), json!(v))).collect();
            json!({ "name": p.name, "values": values })
        })
        .collect()
}
