use atomic_float::AtomicF32;
use nih_plug_vizia::ViziaState;
use nih_plug::prelude::*;
use std::sync::Arc;

mod editor;
const PEAK_METER_DECAY_MS: f64 = 150.0;


// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

pub struct Distortion {
    params: Arc<DistortionParams>,

    peak_meter_decay_weight: f32,

    peak_meter: Arc<AtomicF32>,
}

#[derive(Params)]
struct DistortionParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,

    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

}

impl Default for Distortion {
    fn default() -> Self {
        Self {
            params: Arc::new(DistortionParams::default()),
            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}

impl Default for DistortionParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            threshold: FloatParam::new(
                "Threshold",
                0.5,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 1.0,
                    factor: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            mix: FloatParam::new(
                "Mix",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1.0,
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: 1.0,
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // dec ibels, we need logarithmic smoothing
,
        }
    }
}


impl Plugin for Distortion {
    const NAME: &'static str = "distortion_plugin";
    const VENDOR: &'static str = "Marley Wallace";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "reyaw@protonmail.com";

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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.peak_meter_decay_weight = 0.25f64
        .powf((_buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
        as f32;
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    // CURRENT PROBLEMS:
    // 1: positive side of the threshold slider seemingly does nothing
    // 2: Mix slider should be 0 to 1 instead of -1 to 1
    // 4: Threshold should be in decibels for sure

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves

            let mut amplitude = 0.0;
            let num_samples = channel_samples.len();

            let threshold = self.params.threshold.smoothed.next();
            let mix = self.params.mix.smoothed.next();

            for sample in channel_samples {
                let mut output = sample.clone();
                let clean_out = sample.clone();
                //Split these up for positive and negative input values?????

                if output > threshold {
                   output = threshold;
                  // input = threshold + (1.0/(input-threshold));
                }
                else if output < -threshold {
                    output = -threshold;
                  // input = -threshold - (1.0/(input-threshold));
                }
                // Wet/dry basically
                // Combine distorted signal with original based on mix
                *sample = ((1.0-mix) * clean_out) + (mix * output);
            }
            // To save resources, a plugin can (and probably should!) only perform expensive
            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                amplitude = (amplitude / num_samples as f32).abs();
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed)
            }


        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Distortion {
    const CLAP_ID: &'static str = "com.your-domain.distortion";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Crunchy distortion plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Distortion {
    const VST3_CLASS_ID: [u8; 16] = *b"marleydistortion";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Distortion);
nih_export_vst3!(Distortion);
