mod support;

use sputnik::engine::{Division, Engine, Mode, Settings, Transport};

const SR: f32 = 48_000.0;

fn playing_at(tempo: f64) -> Transport {
    Transport { tempo: Some(tempo), beats: Some(0.0), playing: true }
}

/// Runs mono input through the engine in host-sized blocks, advancing the
/// host's beat position per block, and returns the left output.
fn render_at(input: &[f32], settings: Settings, transport: Transport, sr: f32) -> Vec<f32> {
    let mut engine = Engine::new(sr);
    engine.set(settings);
    engine.reset();
    let mut left = input.to_vec();
    let mut right = input.to_vec();
    let mut beats = transport.beats;
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.set_transport(Transport { beats, ..transport });
        engine.process(l, r);
        if let (Some(b), Some(tempo)) = (beats, transport.tempo) {
            beats = Some(b + l.len() as f64 * tempo / 60.0 / sr as f64);
        }
    }
    left
}

fn render(input: &[f32], settings: Settings) -> Vec<f32> {
    render_at(input, settings, Transport::default(), SR)
}

fn rms(x: &[f32]) -> f32 {
    (x.iter().map(|s| s * s).sum::<f32>() / x.len() as f32).sqrt()
}

/// Notes that decay like plucked strings, one every `every` seconds.
fn plucks(freqs: &[f64], every: f64) -> Vec<f32> {
    let note = support::tone(1.0, every, SR).len();
    freqs
        .iter()
        .flat_map(|&f| {
            support::tone(f, every, SR)
                .into_iter()
                .enumerate()
                .map(move |(i, s)| s * (-(i as f32) / (0.35 * SR)).exp())
                .take(note)
        })
        .collect()
}

fn rising_crossings(x: &[f32]) -> usize {
    x.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count()
}

#[test]
fn silence_stays_exactly_silent() {
    let settings = Settings { synth: 0.5, movement: 1.0, crush: 0.5, sag: 1.0, ..Settings::default() };
    let out = render(&vec![0.0; SR as usize], settings);
    assert!(out.iter().all(|&s| s == 0.0));
}

#[test]
fn bypass_passes_input_through() {
    let input = support::tone(110.0, 0.2, SR);
    let out = render(&input, Settings { stomp: false, ..Settings::default() });
    assert_eq!(out, input);
}

#[test]
fn stutter_chops_on_the_host_grid() {
    let input = support::tone(110.0, 1.0, SR);
    let settings = Settings { movement: 1.0, gate: 0.0, sag: 0.0, crossover_hz: 20.0, ..Settings::default() };
    let out = render_at(&input, settings, playing_at(120.0), SR);
    // At 120 BPM a sixteenth is 125 ms: on for the first half, off for the second.
    let step = (SR * 0.125) as usize;
    let margin = (SR * 0.005) as usize;
    for k in 1..7 {
        let on = rms(&out[k * step + margin..k * step + step / 2 - margin]);
        let off = rms(&out[k * step + step / 2 + margin..(k + 1) * step - margin]);
        assert!(on > 0.05, "step {k}: on {on}");
        assert!(off < on * 0.01, "step {k}: on {on}, off {off}");
    }
}

#[test]
fn stutter_runs_free_without_a_host_tempo() {
    let input = support::tone(110.0, 1.0, SR);
    let settings = Settings { movement: 1.0, gate: 0.0, sag: 0.0, division: Division::Quarter, ..Settings::default() };
    let out = render(&input, settings);
    let half_beat = (SR * 0.25) as usize;
    assert!(rms(&out[half_beat + 500..2 * half_beat - 500]) < 0.001);
    assert!(rms(&out[2 * half_beat + 500..3 * half_beat - 500]) > 0.05);
}

#[test]
fn oscillator_tracks_guitar_and_bass_notes() {
    for freq in [41.2, 110.0, 329.6] {
        let input = support::tone(freq, 1.0, SR);
        let settings = Settings {
            mode: Mode::Synth,
            synth: 1.0,
            movement: 0.0,
            cutoff_hz: 4000.0,
            resonance: 0.0,
            gate: 0.0,
            crossover_hz: 20.0,
            ..Settings::default()
        };
        let out = render(&input, settings);
        let tail = &out[(SR * 0.5) as usize..];
        let heard = rising_crossings(tail) as f64 / 0.5;
        assert!((heard / freq - 1.0).abs() < 0.03, "{freq} Hz tracked as {heard} Hz");
    }
}

#[test]
fn gate_cuts_a_decaying_note() {
    let input = plucks(&[110.0], 2.0);
    let tail = SR as usize..;
    let open = render(&input, Settings { gate: 0.0, sag: 0.0, crossover_hz: 20.0, ..Settings::default() });
    let shut = render(&input, Settings { gate: 1.0, sag: 0.0, crossover_hz: 20.0, ..Settings::default() });
    assert!(rms(&open[tail.clone()]) > 0.02);
    assert!(shut[tail].iter().all(|s| s.abs() < 1e-3));
}

#[test]
fn low_sample_rate_stays_finite() {
    let sr = 22_050.0;
    let input = support::tone(82.4, 1.0, sr);
    let settings = Settings {
        mode: Mode::Synth,
        synth: 0.5,
        movement: 1.0,
        cutoff_hz: 4000.0,
        resonance: 1.0,
        crossover_hz: 400.0,
        crush: 1.0,
        ..Settings::default()
    };
    let out = render_at(&input, settings, Transport::default(), sr);
    assert!(out.iter().all(|s| s.is_finite() && s.abs() < 4.0));
}

#[test]
fn sputter_riff_matches_golden() {
    let (input, sr) = support::read_wav(&support::path("di", "riff.wav"));
    let settings = Settings { fuzz: 0.8, gate: 0.4, sag: 0.5, movement: 0.6, synth: 0.2, ..Settings::default() };
    let out = render_at(&input, settings, playing_at(120.0), sr as f32);
    support::check_golden("sputter-riff", &out, sr);
}

#[test]
fn synth_bass_matches_golden() {
    let input = plucks(&[41.2, 55.0, 61.7, 49.0], 0.75);
    let settings = Settings {
        mode: Mode::Synth,
        synth: 0.5,
        movement: 0.7,
        cutoff_hz: 250.0,
        resonance: 0.6,
        ..Settings::default()
    };
    support::check_golden("synth-bass", &render(&input, settings), SR as u32);
}
