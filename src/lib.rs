use nih_plug::prelude::*;
use std::sync::Arc;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Distortion {
    params: Arc<DistortionParams>,
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

}

impl Default for Distortion {
    fn default() -> Self {
        Self {
            params: Arc::new(DistortionParams::default()),
        }
    }
}

impl Default for DistortionParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                0.0,
                FloatRange::Skewed {
                    min: -0.125,
                    max: 0.125,
                    factor: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB"),
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

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    // CURRENT PROBLEMS:
    // 1: positive side of the threshold slider seemingly does nothing
    // 2: Mix slider should be 0 to 1 instead of -1 to 1
    // 3: Getting weird, unpleasant clipping noises
    // 4: Threshold should be in decibels for sure

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves


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
