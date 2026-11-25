mod support;

use nice_plug::prelude::Param;
use sputnik::engine::{Engine, Mode, Settings};
use sputnik::params::SputnikParams;
use sputnik::presets::{describe, PRESETS};

const SR: f32 = 48_000.0;

fn render(settings: Settings) -> Vec<f32> {
    let input = support::tone(110.0, 1.0, SR);
    let mut engine = Engine::new(SR);
    engine.set(settings);
    engine.reset();
    let (mut left, mut right) = (input.clone(), input);
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.process(l, r);
    }
    left
}

#[test]
fn names_are_unique_and_short() {
    for (i, p) in PRESETS.iter().enumerate() {
        assert!(p.name.len() <= 16, "{} is too long for the panel", p.name);
        assert!(PRESETS[i + 1..].iter().all(|q| q.name != p.name), "{} is repeated", p.name);
    }
}

#[test]
fn the_first_preset_is_the_default_sound() {
    let params = SputnikParams::default();
    let init = describe(|s| params.normalized(s));
    let first = &init[0]["values"];
    assert_eq!(init[0]["name"], "Init");
    for (id, v) in params.normalized(&Settings::default()) {
        assert_eq!(first[id].as_f64().unwrap() as f32, v, "{id}");
    }
}

#[test]
fn every_preset_stays_inside_the_controls() {
    let params = SputnikParams::default();
    for p in PRESETS {
        let values = params.normalized(&p.settings);
        assert_eq!(values.len(), 12, "{} leaves a knob out", p.name);
        for (id, v) in values {
            assert!((0.0..=1.0).contains(&v), "{}: {id} = {v} is off the knob", p.name);
        }
    }
}

#[test]
fn presets_survive_the_trip_through_the_params() {
    let params = SputnikParams::default();
    for p in PRESETS {
        let s = &p.settings;
        let n: std::collections::HashMap<_, _> = params.normalized(s).into_iter().collect();
        let near = |a: f32, b: f64| (a as f64 - b).abs() < 1e-3 * b.abs().max(1.0);
        assert!(near(params.fuzz.preview_plain(n["fuzz"]), s.fuzz), "{} fuzz", p.name);
        assert!(near(params.level.preview_plain(n["level"]), s.level_db), "{} level", p.name);
        assert!(near(params.cutoff.preview_plain(n["cutoff"]), s.cutoff_hz), "{} cutoff", p.name);
        assert!(near(params.crossover.preview_plain(n["crossover"]), s.crossover_hz), "{} crossover", p.name);
        assert_eq!(n["mode"] >= 0.5, s.mode == Mode::Synth, "{} mode", p.name);
    }
}

#[test]
fn every_preset_makes_a_sound_that_stays_finite() {
    for p in PRESETS {
        let out = render(p.settings);
        assert!(out.iter().all(|s| s.is_finite() && s.abs() < 4.0), "{} blew up", p.name);
        let energy: f32 = out.iter().map(|s| s * s).sum();
        assert!(energy > 1.0, "{} is silent", p.name);
    }
}
