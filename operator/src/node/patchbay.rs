pub struct PatchbayIO {
    interface: std::sync::Weak<Patchbay>,
    other: Option<super::IO>,
    is_output: bool,
}

pub struct Patchbay {
    inputs: Vec<std::sync::Arc<PatchbayIO>>,
    outputs: Vec<std::sync::Arc<PatchbayIO>>,
}

impl super::NodeTrait for Patchbay {
    fn inputs(&self) -> Vec<super::IO> {
        self.inputs.iter()
            .map(|input| super::IO::PatchbayIO(input.clone()))
            .collect()
    }

    fn outputs(&self) -> Vec<super::IO> {
        self.outputs.iter()
            .map(|output| super::IO::PatchbayIO(output.clone()))
            .collect()
    }
}