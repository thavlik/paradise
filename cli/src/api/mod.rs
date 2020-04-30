use serde::{Serialize, Deserialize};

/// Defines a virtual audio device which can later be
/// used by the CLI to stream audio.
#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    /// The name of the device, as it appears to the OS.
    pub name: String,

    /// Number of input channels
    pub inputs: usize,

    /// Number of output channels.
    pub outputs: usize,

    /// List of supported sample rates for the device.
    /// There's no reason this has to be device-wide,
    /// but I (Tom) can't image a situation where it
    /// wouldn't be. e.g. cpal lists sample rates for
    /// input and output devices separately, but this
    /// shouldn't be taken as indication that they'll
    /// ever differ.
    /// e.g. vec![48000, 96000, 192000]
    #[serde(rename = "supportedSampleRates")]
    pub supported_sample_rates: Vec<usize>,

    /// The default sample rate. This value must be
    /// present supported_sample_rates or an error
    /// will be raised. If zero, defaults to the
    /// first element of supported_sample_rates.
    #[serde(rename = "defaultSampleRate")]
    pub default_sample_rate: usize,

    /// Case insensitive. Default value is the first item.
    /// Typically we'll deal with 32-bit.
    /// Example options: ["F32", "U16", "U32"]
    #[serde(rename = "supportedSampleFormats")]
    pub supported_sample_formats: Vec<String>,
}

fn reconcile(current: &Device, desired: &Device) {

}

