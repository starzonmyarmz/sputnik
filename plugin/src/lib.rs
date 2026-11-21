mod bridge;
mod editor;
pub mod engine;
pub mod params;

use engine::{Engine, Transport};
use nice_plug::prelude::*;
use params::SputnikParams;
use std::sync::Arc;

static PAGE: editor::Page = editor::Page {
    title: "Sputnik",
    size: (600, 460),
    html: include_str!("../../ui/index.html"),
    version: env!("CARGO_PKG_VERSION"),
    background: (233, 224, 201, 255),
    dev_url_var: "SPUTNIK_UI_URL",
};

pub struct Sputnik {
    params: Arc<SputnikParams>,
    engine: Engine,
}

impl Default for Sputnik {
    fn default() -> Self {
        Self { params: Arc::new(SputnikParams::default()), engine: Engine::new(48_000.0) }
    }
}

impl Plugin for Sputnik {
    const NAME: &'static str = "Sputnik";
    const VENDOR: &'static str = "Dan Marino";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type Editor = editor::WebEditor;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        Some(editor::WebEditor::new(&PAGE, self.params.clone()))
    }

    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        if self.engine.sample_rate() != buffer_config.sample_rate {
            self.engine = Engine::new(buffer_config.sample_rate);
        }
        self.engine.set(self.params.settings());
        self.engine.reset();
        true
    }

    fn reset(&mut self) {
        self.engine.set(self.params.settings());
        self.engine.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        self.engine.set_transport(Transport {
            tempo: transport.tempo,
            beats: transport.pos_beats(),
            playing: transport.playing,
        });
        self.engine.set(self.params.settings());
        if let [left, right] = buffer.as_slice() {
            self.engine.process(left, right);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for Sputnik {
    const CLAP_ID: &'static str = "com.danmarino.sputnik";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A synth fuzz");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Distortion, ClapFeature::Stereo];
}

impl Vst3Plugin for Sputnik {
    const VST3_CLASS_ID: [u8; 16] = *b"SputnikDanMarino";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nice_export_clap!(Sputnik);
nice_export_vst3!(Sputnik);
