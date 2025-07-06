use nih_plug::prelude::*;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::Arc;

use crate::filterbank::FilterBank;

pub mod filterbank;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct ColourizerRs {
    params: Arc<ColourizerRsParams>,
    filterbank: FilterBank,
    filterbanks: Vec<FilterBank>,
    sample_rate: f32,
}

#[derive(Enum, Clone, Copy, Debug, PartialEq, Eq)]
enum ProcessingMode {
    #[id = "mono"]
    Mono,
    #[id = "multi"]
    Multi,
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
    /// Dry/wet mix between 0 (dry) and 1 (wet)
    #[id = "dry_wet"]
    pub dry_wet: FloatParam,
    /// Processing mode: mono or multi-channel
    #[id = "mode"]
    pub mode: EnumParam<ProcessingMode>,
}

impl Default for ColourizerRs {
    fn default() -> Self {
        let sample_rate = 44_100.0;
        Self {
            params: Arc::new(ColourizerRsParams::default()),
            filterbank: FilterBank::new(sample_rate),
            filterbanks: Vec::new(),
            sample_rate,
        }
    }
}

impl Default for ColourizerRsParams {
    fn default() -> Self {
        const MIYAKO_BUSHI: [f32; 12] =
            [1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0];
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
            c: FloatParam::new(
                "C",
                MIYAKO_BUSHI[0],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            c_sharp: FloatParam::new(
                "C#",
                MIYAKO_BUSHI[1],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            d: FloatParam::new(
                "D",
                MIYAKO_BUSHI[2],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            d_sharp: FloatParam::new(
                "D#",
                MIYAKO_BUSHI[3],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            e: FloatParam::new(
                "E",
                MIYAKO_BUSHI[4],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            f: FloatParam::new(
                "F",
                MIYAKO_BUSHI[5],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            f_sharp: FloatParam::new(
                "F#",
                MIYAKO_BUSHI[6],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            g: FloatParam::new(
                "G",
                MIYAKO_BUSHI[7],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            g_sharp: FloatParam::new(
                "G#",
                MIYAKO_BUSHI[8],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            a: FloatParam::new(
                "A",
                MIYAKO_BUSHI[9],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            a_sharp: FloatParam::new(
                "A#",
                MIYAKO_BUSHI[10],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            b: FloatParam::new(
                "B",
                MIYAKO_BUSHI[11],
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            dry_wet: FloatParam::new("Dry/Wet", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            mode: EnumParam::new("Processing Mode", ProcessingMode::Mono),
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
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(6),
            main_output_channels: NonZeroU32::new(6),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames {
                layout: Some("5.1"),
                ..PortNames::const_default()
            },
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames {
                layout: Some("Mono"),
                ..PortNames::const_default()
            },
        },
    ];

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
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.filterbank = FilterBank::new(self.sample_rate);
        self.filterbanks = (0..audio_io_layout
            .main_output_channels
            .map(NonZeroU32::get)
            .unwrap_or(0) as usize)
            .map(|_| FilterBank::new(self.sample_rate))
            .collect();
        let _ = ThreadPoolBuilder::new().build_global();
        true
    }

    fn reset(&mut self) {
        self.filterbank = FilterBank::new(self.sample_rate);
        for fb in &mut self.filterbanks {
            *fb = FilterBank::new(self.sample_rate);
        }
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
        match self.params.mode.value() {
            ProcessingMode::Mono => {
                self.filterbank.set_gains(note_gains);
                let mix = self.params.dry_wet.value();
                for mut samples in buffer.iter_samples() {
                    let gain = self.params.gain.smoothed.next();
                    let mut sum = 0.0;
                    for sample in samples.iter_mut() {
                        sum += *sample;
                    }
                    let input_sum = sum / samples.len() as f32;
                    let processed = self.filterbank.process_sample(input_sum) * gain;
                    for sample in samples.iter_mut() {
                        let dry = *sample;
                        *sample = dry * (1.0 - mix) + processed * mix;
                    }
                }
            }
            ProcessingMode::Multi => {
                let channels = buffer.as_slice();
                if self.filterbanks.len() != channels.len() {
                    self.filterbanks = (0..channels.len())
                        .map(|_| FilterBank::new(self.sample_rate))
                        .collect();
                }
                for fb in &mut self.filterbanks {
                    fb.set_gains(note_gains);
                }
                let gain = self.params.gain.smoothed.next();
                let mix = self.params.dry_wet.value();
                channels
                    .par_iter_mut()
                    .zip(self.filterbanks.par_iter_mut())
                    .for_each(|(ch, fb)| {
                        for sample in ch.iter_mut() {
                            let dry = *sample;
                            let wet = fb.process_sample(dry) * gain;
                            *sample = dry * (1.0 - mix) + wet * mix;
                        }
                    });
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
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Surround,
    ];
}

impl Vst3Plugin for ColourizerRs {
    const VST3_CLASS_ID: [u8; 16] = *b"ColourizerEffect";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Dynamics,
        Vst3SubCategory::Surround,
    ];
}

nih_export_clap!(ColourizerRs);
nih_export_vst3!(ColourizerRs);

#[cfg(test)]
mod tests {
    use super::*;
    use nih_plug::prelude::*;

    struct DummyContext;

    impl ProcessContext<ColourizerRs> for DummyContext {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Vst3
        }

        fn execute_background(&self, _task: ()) {}
        fn execute_gui(&self, _task: ()) {}
        fn transport(&self) -> &Transport {
            panic!("unused")
        }
        fn next_event(&mut self) -> Option<PluginNoteEvent<ColourizerRs>> {
            None
        }
        fn send_event(&mut self, _event: PluginNoteEvent<ColourizerRs>) {}
        fn set_latency_samples(&self, _samples: u32) {}
        fn set_current_voice_capacity(&self, _capacity: u32) {}
    }

    fn plugin_with_mix(mix: f32) -> ColourizerRs {
        let mut params = ColourizerRsParams::default();
        params.dry_wet = FloatParam::new("Dry/Wet", mix, FloatRange::Linear { min: 0.0, max: 1.0 });
        ColourizerRs {
            params: Arc::new(params),
            filterbank: FilterBank::new(44_100.0),
            filterbanks: Vec::new(),
            sample_rate: 44_100.0,
        }
    }

    fn run_once(mut p: ColourizerRs) -> Vec<f32> {
        let mut data = vec![1.0; 16];
        let mut buffer = Buffer::default();
        unsafe {
            buffer.set_slices(data.len(), |s| {
                *s = vec![data.as_mut_slice()];
            });
        }
        let mut aux = AuxiliaryBuffers {
            inputs: &mut [],
            outputs: &mut [],
        };
        let mut ctx = DummyContext;
        p.process(&mut buffer, &mut aux, &mut ctx);
        data
    }

    #[test]
    fn dry_only_passes_input() {
        let out = run_once(plugin_with_mix(0.0));
        for s in out {
            assert!((s - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn wet_only_blocks_without_filters() {
        let out = run_once(plugin_with_mix(1.0));
        for s in out {
            assert!(s.abs() < 1e-6);
        }
    }

    #[test]
    fn half_mix_interpolates() {
        let dry = run_once(plugin_with_mix(0.0));
        let wet = run_once(plugin_with_mix(1.0));
        let half = run_once(plugin_with_mix(0.5));
        for ((d, w), h) in dry.iter().zip(wet.iter()).zip(half.iter()) {
            let expected = 0.5 * (*d) + 0.5 * (*w);
            assert!((*h - expected).abs() < 1e-6);
        }
    }
}
