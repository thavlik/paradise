use serde::{Serialize, Deserialize};

/// Defines a virtual audio device which can later be
/// used by the CLI to stream audio. 
#[derive(Serialize, Deserialize, Debug, Clone)]
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

    /// Case insensitive. Default value is the first item.
    /// Typically we'll deal with 32-bit.
    /// Example options: ["F32", "U16", "U32"]
    #[serde(rename = "supportedSampleFormats")]
    pub supported_sample_formats: Vec<String>,
}

fn reconcile(current: &Device, desired: &Device) -> Result<(), ()> {
    Err(())
}

#[cfg(test)]
mod test {
    use super::*;
    use difference::{Difference, Changeset};

    #[test]
    fn foo() {
        let a = serde_yaml::to_string(&Device{
            name: String::from("foo"),
            inputs: 2,
            outputs: 2,
            supported_sample_rates: vec![48000],
            supported_sample_formats: vec![String::from("F32")],
        }).unwrap();
        let b = serde_yaml::to_string(&Device{
            name: String::from("bar"),
            inputs: 1,
            outputs: 2,
            supported_sample_rates: vec![48000],
            supported_sample_formats: vec![String::from("F32")],
        }).unwrap();

        // Compare both texts, the third parameter defines the split level.
        let Changeset { diffs, .. } = Changeset::new(&a, &b, "\n");

        let mut t = term::stdout().unwrap();

        for i in 0..diffs.len() {
            match diffs[i] {
                Difference::Same(ref x) => {
                    t.reset().unwrap();
                    writeln!(t, " {}", x);
                }
                Difference::Add(ref x) => {
                    t.fg(term::color::GREEN).unwrap();
                    writeln!(t, "+{}", x);
                }
                Difference::Rem(ref x) => {
                    t.fg(term::color::RED).unwrap();
                    writeln!(t, "-{}", x);
                }
            }
        }
        t.reset().unwrap();
        t.flush().unwrap();
    }
}