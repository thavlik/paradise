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
    let mut iface = Interface::new(8);

    const NUM_CHANNELS: usize = 128;
    const NUM_INTERCONNECT_CHANNELS: usize = 32;

    let mut patchbays: Vec<_> = (0..8)
        .map(|i| Patchbay::new(NUM_CHANNELS as u8))
        .collect();

    let mut ifaces: Vec<_> = (0..1)
        .map(|i| Interface::new(8))
        .collect();

    for i in 0..patchbays.len()-1 {
        for n in 0..NUM_INTERCONNECT_CHANNELS {
            *patchbays[i+0].outputs[n].other
                .lock()
                .unwrap() = Some(IO::PatchbayIO(patchbays[i+1].inputs[n].clone()));
            *patchbays[i+1].inputs[n].other
                .lock()
                .unwrap() = Some(IO::PatchbayIO(patchbays[i+0].outputs[n].clone()));
        }
    }

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

    //let result = astar(&Pos(1, 1), |p| p.successors(), |_| 0,
    //                   |p| *p == GOAL);
    //assert_eq!(result.expect("no path found").1, 4);
    println!("Hello, world!");
}
