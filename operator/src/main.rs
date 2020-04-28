use pathfinding::prelude::{absdiff, astar};
use std::sync::{
    Arc,
    Mutex,
};

mod node;

use node::{
    Node,
    IO,
    interface::{
        Interface,
        InterfaceIO,
    },
    patchbay::{
        Patchbay,
        PatchbayIO,
    },
};

fn main() {
    //let result = astar(&Pos(1, 1), |p| p.successors(), |_| 0,
    //                   |p| *p == GOAL);
    //assert_eq!(result.expect("no path found").1, 4);
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn foo() {
        let mut iface = Interface::new(8);

        const NUM_CHANNELS: usize = 128;
        const NUM_INTERCONNECT_CHANNELS: usize = 32;
        const NUM_UNITS: usize = 4;

        // Create some patchbays
        let mut patchbays: Vec<_> = (0..8)
            .map(|i| Patchbay::new(NUM_CHANNELS as u8))
            .collect();

        // Create some audio interfaces
        let mut ifaces: Vec<_> = (0..1)
            .map(|i| Interface::new(8))
            .collect();

        // Connect the first handful of channels to the next
        // patchbay. The first unit has unused input channels
        // and the last has as many unused outputs.
        for i in 0..patchbays.len()-1 {
            let (a, b) = patchbays[i..i+2].split_at_mut(1);
            a[0].outputs[..NUM_INTERCONNECT_CHANNELS].iter_mut()
                .zip(b[0].inputs[..NUM_INTERCONNECT_CHANNELS].iter_mut())
                .for_each(|(output, input)| {
                    input.set_other(Some(IO::PatchbayIO(output.clone())));
                    output.set_other(Some(IO::PatchbayIO(input.clone())));
                });
        }

        // Connect all of the channels of the audio interfaces
        // to the next available range of channels on each PB.
        ifaces.iter_mut()
            .zip(patchbays.iter_mut())
            .enumerate()
            .for_each(|(i, (iface, pb))| {
                iface.inputs.iter_mut()
                    .zip(pb.outputs[NUM_INTERCONNECT_CHANNELS..].iter_mut())
                    .for_each(|(input, output)| {
                        input.set_other(Some(IO::PatchbayIO(output.clone())));
                        output.set_other(Some(IO::InterfaceIO(input.clone())));
                    });
                iface.outputs.iter_mut()
                    .zip(pb.inputs[NUM_INTERCONNECT_CHANNELS..].iter_mut())
                    .for_each(|(output, input)| {
                        input.set_other(Some(IO::InterfaceIO(output.clone())));
                        output.set_other(Some(IO::PatchbayIO(input.clone())));
                    });
            });

        // Add some audio units to the last patchbay
        let units: Vec<_> = (0..NUM_UNITS).map(|i| {
            0
        }).collect();
    }
}