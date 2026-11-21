# Sputnik

Read `README.md` first for what the plugin does and how to build and run
it. This file has the maintainer details it leaves out. The plan and its
reasoning live in the Keep Up idea "Sputnik: synth fuzz pedal"
(`keepup idea show 66`).

## Layout

- `plugin/src/engine.rs`: all DSP, with no plugin framework types, so tests
  drive it directly.
- `plugin/src/lib.rs`: host parameters and plumbing.
- `plugin/src/editor.rs` and `plugin/src/bridge.rs`: nothing specific to
  Sputnik. The page's size and HTML come from `PAGE` in `lib.rs`, and the
  bridge works with any `#[derive(Params)]` struct.
- `ui/index.html`: the whole editor, compiled into the plugin, with a mock
  bridge for running it in a browser.

A new parameter goes in `SputnikParams` in `lib.rs`, the engine's
setters in `engine.rs`, and the mock's `specs` in `ui/index.html`.

## Sound changes and golden renders

`plugin/tests/golden` holds renders the tests compare against. After an
intended change to the sound, rerun with `BLESS=1 cargo test` to rewrite
them. Never commit rewritten renders until the user has listened to
them, and say in the commit what changed in the sound.

## Things clap-validator catches

Run `scripts/clap-validator.sh target/bundled/Sputnik.clap` after any
DSP change, not just in CI.

- Subnormal output. The filter, envelope follower, sag and smoothers
  decay into subnormals in silence. Flush state and output below about
  1e-30 to zero.
- NaN from a filter corner above Nyquist at low sample rates. Clamp
  corners below Nyquist.
- `v2s_f32_hz_then_khz` already appends Hz/kHz, so don't add
  `with_unit(" Hz")`. `v2s_f32_percentage` doesn't append `%`, so
  `with_unit("%")` is right with it.

## Sound

- Two modes: Sputter (gated, choked square fuzz, with a stutter gate
  synced to host tempo and a fixed rate when there's none) and Synth (fuzz
  into a resonant filter swept by an envelope follower).
- A pitch-tracked square/saw oscillator is blended in with Synth. When
  tracking fails on chords, noise or slides, its glitches are kept on
  purpose. Don't smooth or duck them.
- A clean low-end crossover sits in front, so bass keeps its weight.

## Design

Settled with the user; keep to it.

- Object: a 1957 mission control console. Cream enamel panel, black
  bakelite knobs with engraved labels, and a green phosphor CRT
  oscilloscope.
- Controls: Fuzz, Gate, Sag, Synth, Move, Level, in plain words (themed
  names were turned down). A two-position rotary selector, SPUTTER /
  SYNTH, above two rows of small trims: Tone and Peak (Cutoff and
  Resonance), then Crush, Rate and Low (Crossover). Every parameter has
  its own knob: the user doesn't want anything a real pedal couldn't do,
  so no XY pad or dragging on the screen. A centred metal `stomp` switch
  at the bottom, on by default, with a red lamp beside it.
- Version: a small riveted, engraved ID plate in a bottom corner,
  `SPUTNIK-1  No. v1.0.0`.
- Palette: cream enamel panel #e9e0c9, black bakelite #2b2a27, green
  phosphor #7dff9a on CRT glass #0d1a12, red lamp #ff3b2f, warm grey
  engraving #5b5345. Green belongs to the CRT and red to the lamp only.
- Art: the CRT shows the real output as a triggered oscilloscope trace
  with phosphor persistence, flat in silence. `scope.rs` captures it on
  the audio thread and the editor's feed sends each new frame. Nothing
  else animates.
- Wordmark: Stardos Stencil Bold, cream on a riveted black bakelite
  badge. Only the SPUTNIK glyphs are embedded; re-subset from
  `ui/fonts/StardosStencil-Bold.ttf` if the wordmark's letters change.
  Labels are engraved small caps in Helvetica Neue.

## Editor

- The window size is set in `PAGE.size` in `lib.rs` and by the page's
  outer element in `ui/index.html`. Change them together.
- The local preview server gets cached by the browser. Add `?v=N` after
  each edit. Keep the mock's `specs` in step with the plugin's
  parameters, including the skew of any skewed range.
- Fonts can't load from the network: embed them as base64 `@font-face`
  data URLs, and keep the file and its license in `ui/fonts`.
- Check that newer CSS features are in the macOS WKWebView before
  relying on them.
- After a visible UI change, retake `docs/sputnik.jpg` with the
  `screenshot` skill.

## Testing in REAPER

- `scripts/install.sh` installs into `~/Library/Audio/Plug-Ins`. REAPER
  keeps a plugin's dylib loaded, so restart it after reinstalling.
- REAPER only monitors input on armed tracks.

## Releases

`release.yml` runs on `vX.Y.Z` tags, which must match `version` in the
root `Cargo.toml`. To release: bump `version`, commit, then
`git tag vX.Y.Z && git push origin main vX.Y.Z`.

## nice-plug fork

The plugin is pinned to the `patina` branch of a nice-plug fork, which
adds `ActivateContext::gui_context()`. To upgrade, rebase that branch onto
upstream, push it, and update the `rev` in both `plugin/Cargo.toml` and
`xtask/Cargo.toml`.
