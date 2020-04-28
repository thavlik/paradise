
pub struct InterfaceIO {
    interface: std::sync::Weak<Interface>,
    channel: usize,
    other: Option<super::IO>,
    is_output: bool,
}

pub struct Interface {
    inputs: Vec<std::sync::Arc<InterfaceIO>>,
    outputs: Vec<std::sync::Arc<InterfaceIO>>,
}

impl super::NodeTrait for Interface {
    fn inputs(&self) -> Vec<super::IO> {
        self.inputs.iter()
            .map(|input| super::IO::InterfaceIO(input.clone()))
            .collect()
    }

    fn outputs(&self) -> Vec<super::IO> {
        self.outputs.iter()
            .map(|output| super::IO::InterfaceIO(output.clone()))
            .collect()
    }
}