# Design

How Sputnik looks and feels. When this file and your instincts disagree, this file wins. Change it on purpose, in the same commit as the code.

## The look in one line

A 1957 mission control console: cream enamel, black bakelite and a green phosphor oscilloscope.

## Principles

Every plugin gets its own look, but they share an approach.

- **Skeuomorphic.** The editor is a physical object (Patina an OP-1-style synth, Halfwit a stompbox, Ghosted a vintage radio), with real materials like powder coat, bakelite, walnut, brass and grille cloth, built in CSS gradients and shadows rather than images.
- **One theme.** Pick the object from the plugin's vibe and name, and let it run through everything: the wordmark, the art and the control labels. Names should be playful and capture the vibe.
- **Restrained.** One knob style and colour family per plugin, and few controls. Fold many settings into a handful of knobs with evocative names rather than exposing every one.
- **Colour and art mean something.** If two waves or knobs differ in colour, it's because they differ in function; otherwise make them the same. Art can follow parameters (Ghosted's ghosts fade with Haunt, the dial needle follows Distance), but don't fake audio metering.
- **Plain labels.** Words a guitarist understands, not DSP terms or fieldset jargon. Numbers go in a tooltip readout.
- **Type with character.** Embed a display face for the wordmark (Futura-like Jost, pillowy Modak), never a system or "commercial" font, and centre it optically on its badge.
- **Drawn, not vector-perfect.** Vary repeated elements (size, tilt, expression), and give outlines weight that swells on the shadow side, like inked comic art.
- **A footswitch on effects.** Pedals and other effects get an on/off toggle (`stomp`, on by default). Off fades the effect out and passes the dry signal; a lamp or LED shows the state, and the art reacts (Ghosted's dial dims and its ghosts leave).
- **The version on the panel.** Print it small and quiet as part of the object, like a model number on a pedal's base plate or a serial plate on a radio's back (`v1.2.0`), so anyone can tell which build they're running. Local installs show `X.Y.Z-dev+<sha>`.
- **Physical details behave physically.** Shadows stay put while a knob turns, pressed keys sink, lit things glow.

## This plugin

Settled with the user; keep to it.

- **Object:** a 1957 mission control console. Cream enamel panel, black bakelite knobs with engraved labels, and a green phosphor CRT oscilloscope.
- **Palette:** see Tokens. Green belongs to the CRT and red to the lamp only.
- **Wordmark font:** Stardos Stencil Bold, cream on a riveted black bakelite badge. Only the SPUTNIK glyphs are embedded; re-subset from `ui/fonts/StardosStencil-Bold.ttf` if the wordmark's letters change. Labels are engraved small caps in Helvetica Neue.
- **Controls:** Fuzz, Gate, Sag, Synth, Move, Level, in plain words (themed names were turned down). A two-position rotary selector, SPUTTER / SYNTH, above two rows of small trims: Tone and Peak (Cutoff and Resonance), then Crush, Rate and Low (Crossover). Every parameter has its own knob: nothing a real pedal couldn't do, so no XY pad or dragging on the screen. A centred metal `stomp` switch at the bottom, on by default, with a red lamp beside it.
- **Version placement:** a small riveted, engraved ID plate in a bottom corner, `SPUTNIK-1  No. v1.0.0`.
- **Art:** the CRT shows the real output as a triggered oscilloscope trace with phosphor persistence, flat in silence. Nothing else animates.

## Tokens

Every color, size and space comes from a token. If nothing fits, add a token here before using a new value.

| Token | Value | Used for |
|---|---|---|
| enamel | `#e9e0c9` | the panel |
| bakelite | `#2b2a27` | knobs and the wordmark badge |
| phosphor | `#7dff9a` | the CRT trace only |
| crt-glass | `#0d1a12` | the CRT screen |
| lamp | `#ff3b2f` | the stomp lamp only |
| engraving | `#5b5345` | engraved labels |

## Components

## Patterns

## Adding something new

1. Look for an existing component or pattern that already does it.
2. Build it from tokens only.
3. Check Reduce Motion if it moves.
4. Retake the screenshot if it shows in one.
5. Record it above.

## Known drift

Existing debt to clean up, not to copy.
