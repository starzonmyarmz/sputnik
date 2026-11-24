# Sputnik

A sputtering synth fuzz for guitar and bass, with an oscilloscope: a VST3 and CLAP plugin

The plan and its reasoning live in the Keep Up idea "Sputnik: synth fuzz pedal" (`keepup idea show 66`).

Read before working:
- `DESIGN.md` before touching anything people see.

## Commands

```sh
cargo test                                  # engine tests and golden renders, offline
BLESS=1 cargo test                          # rewrite golden renders after an intended change to the sound
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo xtask bundle sputnik --release     # target/bundled/Sputnik.{vst3,clap}
scripts/install.sh                          # bundle, sign ad hoc and install on macOS as X.Y.Z-dev+<sha>
scripts/clap-validator.sh "target/bundled/Sputnik.clap"
scripts/screenshot.sh                       # retake docs/sputnik.png
scripts/release --dry-run                   # what the next release would be
```

Work on the editor in a browser against the mock: `cd ui && python3 -m http.server 8765`, then open http://127.0.0.1:8765 (add `?version=1.2.0` to show a version). To load the page live in the plugin, set `SPUTNIK_UI_URL` to that URL before starting the host.

## Layout

- `plugin/src/engine.rs`: the DSP, in f64, with no host types, so `cargo test` runs it offline.
- `plugin/src/params.rs`: the parameters. Their defaults come from `Settings::default()`, and `settings()` turns them into the engine's `Settings`.
- `plugin/src/lib.rs`: host plumbing, plugin IDs and `PAGE` (title, window size, version).
- `plugin/src/scope.rs`: captures the output on the audio thread for the oscilloscope, and the editor's feed sends each new frame.
- `plugin/src/editor.rs`, `bridge.rs`: the webview editor and its message bridge. Nothing in them is specific to Sputnik. Don't add another channel to `editor.rs`; use `WebEditor::with_feed`.
- `plugin/tests/`: engine tests, golden renders in `golden/` and real DI recordings in `di/`. `support/` has the WAV and golden helpers.
- `ui/index.html`: the whole editor, compiled into the plugin, with a hand-written mock bridge (`mockBridge()`) for running it in a browser.

A new parameter goes in `SputnikParams` in `params.rs`, the engine's `Settings` in `engine.rs`, and the mock's `specs` in `ui/index.html`.
- `xtask/`, `bundler.toml`: nice-plug's bundler.
- `scripts/`: install, clap-validator, screenshot and release, with `pre-release.sh` retaking the screenshot for each release.
- `.claude/skills/`: `design-kickoff` (settle the look before building the editor), `screenshot` and `bless-goldens`. Use them rather than working from memory.

## Invariants and footguns

What was learned the hard way. Add to this list whenever something breaks in a way the code doesn't make obvious.

- `CLAP_ID` and `VST3_CLASS_ID` in `lib.rs` never change after the first release, and parameter IDs never change without a major release. Hosts use them to find the plugin and restore saved projects.
- After a change to `plugin/src` or `ui/`, run `scripts/install.sh` so the user can try it. It installs as `X.Y.Z-dev+<sha>`, so the panel shows which build the host loaded. REAPER keeps a plugin's dylib loaded: tell the user to restart it.
- Don't bump the version by hand. It only changes at release.

### Sound

- Two modes: Sputter (a gated, choked square fuzz, with a stutter gate synced to host tempo and a fixed rate when there's none) and Synth (fuzz into a resonant filter swept by an envelope follower).
- A pitch-tracked square/saw oscillator is blended in with Synth. When tracking fails on chords, noise or slides, its glitches are kept on purpose. Don't smooth or duck them.
- A clean low-end crossover sits in front, so bass keeps its weight.
- The filter, envelope follower, sag and smoothers all decay into subnormals in silence: flush them (see below).

### Parameters and the engine

- The engine works in f64. Each `Settings` field is in the units the control shows. When a few knobs drive many engine values, add a `Controls` struct whose `settings()` does the mapping (Ghosted, Altitude), so the engine can still be tested on raw values.
- The engine glides to new settings itself, with one-pole smoothers built from `coef()`, so turning a knob doesn't click. `reset()` jumps to the current settings; the plugin calls it on activate and reset.
- Keep `stomp` on effects. Glide the wet amount toward 0 or 1, and `flush()` it: anything gliding toward 0 decays into subnormals otherwise.
- Random numbers come from a seeded RNG (Ghosted's xorshift64* `Rng`), so golden renders repeat exactly.
- Tempo sync: read the host transport in `process` and pass the engine plain values. Fall back to 120 BPM when the host gives no tempo.

### Things clap-validator catches

Run `scripts/clap-validator.sh` after any DSP change, not just in CI.

- Subnormal output. Recursive filters, envelopes and smoothers decay into subnormals in silence. Flush state and output below about 1e-30 to zero.
- nice-plug's `v2s_f32_hz_then_khz` already appends Hz/kHz, so adding `with_unit(" Hz")` doubles it. `v2s_f32_gain_to_db` and `v2s_f32_percentage` don't, so `with_unit` is right with those.
- NaN from filters whose corner frequency is above Nyquist at low sample rates. Clamp corners below Nyquist.

### Editor

- The window size is set in `PAGE.size` in `lib.rs` and by the page's outer element in `ui/index.html`. Change them together.
- The browser preview gets cached: add `?v=N` after each edit. Keep the mock's `specs` in `ui/index.html` in step with the plugin's parameters, including the skew of any skewed range. (Sputnik predates the kit's generated `ui/mock.json`.)
- To show the page something that isn't a parameter (an LED, a scope, a meter), use `WebEditor::with_feed`. Share the state from the audio thread through atomics, and have the closure return a message only when something changed, since it runs every frame.
- SVG reused through `<use>` doesn't get page CSS rules. Stamp it from a `<template>` instead, or put attributes on the elements themselves.
- Fonts can't load from the network: embed them as base64 `@font-face` data URLs, and keep the file and its license in `ui/fonts`.
- The macOS webview is the system WKWebView, so check a newer CSS feature is in Safari for the macOS versions you support. Windows uses WebView2.
- After a visible UI change, retake the screenshot with the `screenshot` skill, commit it with the change, and keep the README `alt` text current. Nothing on GitHub retakes it.

### Testing in REAPER

- REAPER only monitors input on armed tracks.
- Multi-output plugins: declare a second layout without the aux port so hosts without multi-out still load them, and keep a scratch buffer for the aux signal in that case (see Halfwit).

### nice-plug fork

Plugins pin the `patina` branch of [starzonmyarmz/nice-plug](https://github.com/starzonmyarmz/nice-plug/tree/patina): upstream plus one commit that adds `ActivateContext::gui_context()`. Keep the fork to that one commit.

- Upgrading: rebase `patina` onto upstream, push, and bump the `rev` in `plugin/` and `xtask/` `Cargo.toml` in every plugin. Then run the tests and clap-validator in each.
- Don't send the patch upstream. It was written with AI, and nice-plug's [AI policy](https://rust.audio/community/ai/) doesn't accept AI-written code, PR descriptions or comments.

## Conventions

- Conventional Commits with Title-Case scopes: `feat(Engine): …`. The body explains why.
- Tests ship in the same commit as the change. Pure logic is tested; the UI or host shell stays thin.
- A `feat`, `fix` or `perf` commit that someone using Sputnik would notice adds a line under `## [Unreleased]` in `CHANGELOG.md`, in the same commit, written for them, not as a code summary.
- This repo is public. Keep real names, logins, paths and work data out of code, fixtures, docs, the changelog and commits.
- Breaking means a renamed parameter ID or a preset that no longer loads.

## Releasing

Releases happen only when asked. Never bump, tag or publish on your own.

`scripts/release` works out the bump from commits since the last tag, finalizes the changelog, commits `chore(Release): X.Y.Z`, tags `vX.Y.Z` and pushes. The tag triggers `.github/workflows/release.yml`.
