use nih_plug::prelude::*;
use std::sync::Arc;

use crate::filterbank::FilterBank;

pub mod filterbank;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct ColourizerRs {
    params: Arc<ColourizerRsParams>,
    filterbank: FilterBank,
    sample_rate: f32,
}

#[derive(Params)]
struct ColourizerRsParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "c"]
    pub c: FloatParam,
    #[id = "c_sharp"]
    pub c_sharp: FloatParam,
    #[id = "d"]
    pub d: FloatParam,
    #[id = "d_sharp"]
    pub d_sharp: FloatParam,
    #[id = "e"]
    pub e: FloatParam,
    #[id = "f"]
    pub f: FloatParam,
    #[id = "f_sharp"]
    pub f_sharp: FloatParam,
    #[id = "g"]
    pub g: FloatParam,
    #[id = "g_sharp"]
    pub g_sharp: FloatParam,
    #[id = "a"]
    pub a: FloatParam,
    #[id = "a_sharp"]
    pub a_sharp: FloatParam,
    #[id = "b"]
    pub b: FloatParam,
}

impl Default for ColourizerRs {
    fn default() -> Self {
        let sample_rate = 44_100.0;
        Self {
            params: Arc::new(ColourizerRsParams::default()),
            filterbank: FilterBank::new(sample_rate),
            sample_rate,
        }
    }
}

impl Default for ColourizerRsParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            c: FloatParam::new("C", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            c_sharp: FloatParam::new("C#", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            d: FloatParam::new("D", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            d_sharp: FloatParam::new("D#", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            e: FloatParam::new("E", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            f: FloatParam::new("F", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            f_sharp: FloatParam::new("F#", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            g: FloatParam::new("G", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            g_sharp: FloatParam::new("G#", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            a: FloatParam::new("A", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            a_sharp: FloatParam::new("A#", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            b: FloatParam::new("B", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

impl Plugin for ColourizerRs {
    const NAME: &'static str = "Colourizer Rs";
    const VENDOR: &'static str = "Daishi Suzuki";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "zukky.rikugame@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.filterbank = FilterBank::new(self.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.filterbank = FilterBank::new(self.sample_rate);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let note_gains = [
            self.params.c.value(),
            self.params.c_sharp.value(),
            self.params.d.value(),
            self.params.d_sharp.value(),
            self.params.e.value(),
            self.params.f.value(),
            self.params.f_sharp.value(),
            self.params.g.value(),
            self.params.g_sharp.value(),
            self.params.a.value(),
            self.params.a_sharp.value(),
            self.params.b.value(),
        ];
        self.filterbank.set_gains(note_gains);

        for mut samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            let mut sum = 0.0;
            for sample in samples.iter_mut() {
                sum += *sample;
            }
            let input_sum = sum / samples.len() as f32;

            let processed = self.filterbank.process_sample(input_sum) * gain;

            for sample in samples.iter_mut() {
                *sample = processed;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for ColourizerRs {
    const CLAP_ID: &'static str = "com.zukky.colourizer-rs";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("colourizer effect");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for ColourizerRs {
    const VST3_CLASS_ID: [u8; 16] = *b"ColourizerEffect";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(ColourizerRs);
nih_export_vst3!(ColourizerRs);
