# Sputnik

<img src="docs/sputnik.png" width="504" alt="Sputnik, a cream enamel mission-control console with a stencilled SPUTNIK badge, a green phosphor oscilloscope showing the output, a Sputter/Synth selector with tone, peak, crush, rate and low trims, a row of fuzz, gate, sag, synth, move and level bakelite knobs, a centred footswitch with a red lamp, and a riveted ID plate">

A digital synth fuzz for guitar and bass, as a VST3 and CLAP plugin for
macOS and Windows. A square-wave fuzz that chokes and sputters, with an
oscillator that follows your playing blended in, and a phosphor
oscilloscope that shows what's coming out.

It has two modes: Sputter, a gated fuzz that stutters in time with your
host, and Synth, a fuzz bass with a filter that opens as you dig in.

## Installing

Download the zip from
[Releases](https://github.com/starzonmyarmz/sputnik/releases) and follow
`INSTALL.txt` inside it.

## Building

```sh
cargo xtask bundle sputnik --release       # target/bundled/Sputnik.{vst3,clap}
scripts/bump.sh [patch|minor|major]        # bump the version (patch by default)
scripts/install.sh                         # bundle, sign ad hoc and install on macOS
scripts/clap-validator.sh target/bundled/Sputnik.clap
```

## Tests

`cargo test` runs the engine offline, in host-sized blocks, against
stored golden renders and synthetic checks. After an intended change to
the sound, rerun with `BLESS=1` to rewrite the renders, and listen to
them before committing.

## Working on the UI

The editor is a webview showing `ui/index.html`, which is compiled into
the plugin. Without the plugin, the page runs against a mock, so you can
work on it in a browser:

```sh
cd ui && python3 -m http.server 8765    # open http://127.0.0.1:8765
```

Add `?version=1.2.0` to show a version other than `0.0.0`. To load
the page live in the plugin, set `SPUTNIK_UI_URL` to that URL
before starting the host.

## CI and releases

- `check.yml` runs on pushes to main and on PRs: the tests and
  clap-validator on macOS, and clap-validator on the Windows build.
- `release.yml` runs on `vX.Y.Z` tags. The tag must match `version` in
  the root `Cargo.toml`. It builds a universal macOS bundle and a
  Windows bundle, validates both, and publishes `Sputnik-X.Y.Z.zip`
  with `INSTALL.txt`.
- `screenshot.yml` runs on pushes to main that touch `ui/`, the version
  or `plugin/src/lib.rs`. It retakes `docs/sputnik.png` with
  `scripts/screenshot.sh`, sets its width in the README, and commits both
  if the image changed.

## Built with

- [nice-plug](https://codeberg.org/RustAudio/nice-plug), pinned to
  [a fork](https://github.com/starzonmyarmz/nice-plug/tree/patina) that
  adds `ActivateContext::gui_context()`.
- Started from
  [plugin-template](https://github.com/starzonmyarmz/plugin-template).
