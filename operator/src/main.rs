use pathfinding::prelude::{astar};
use std::ops::Deref;
use std::sync::{
    Arc,
    Mutex,
};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
mod node;

use node::{
    Node,
    NodeKind,
    IO,
    IOHandle,
    AudioUnit,
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
        const NUM_PATCHBAYS: usize = 6;
        const NUM_CHANNELS: usize = 128;
        const NUM_INTERCONNECT_CHANNELS: usize = 32;
        const NUM_UNITS: usize = 4;
        const NUM_CHANNEL_STRIPS: usize = 256;

        let mut patchbay_io = (0..NUM_PATCHBAYS)
            .map(|i| (
                (0..NUM_CHANNELS)
                    .map(|j| IOHandle::new(IO::new(j as u8, false, None)))
                    .collect::<Vec<_>>(),
                (0..NUM_CHANNELS)
                    .map(|j| IOHandle::new(IO::new(j as u8, true, None)))
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>();

        // Connect the first handful of channels to the next
        // patchbay. The first unit has unused input channels
        // and the last has as many unused outputs.
        for i in 0..patchbay_io.len() - 1 {
            let (a, b) = patchbay_io[i..i + 2].split_at_mut(1);
            a[0].1[..NUM_INTERCONNECT_CHANNELS].iter_mut()
                .zip(b[0].0[..NUM_INTERCONNECT_CHANNELS].iter_mut())
                .for_each(|(output, input)| {
                    //output.borrow_mut()
                    //    .input
                    //    .replace(input.clone());
                });
            a[0].0[..NUM_INTERCONNECT_CHANNELS].iter_mut()
                .zip(b[0].1[..NUM_INTERCONNECT_CHANNELS].iter_mut())
                .for_each(|(input, output)| {
                    //output.borrow_mut()
                    //    .input
                    //    .replace(input.clone());
                });
        }

        // Create some patchbays from the inputs/outputs
        let mut patchbays: Vec<_> = patchbay_io
            .into_iter()
            .map(|(inputs, outputs)| Node::make(NodeKind::Patchbay, inputs, outputs))
            .collect();

        let mut ifaces: Vec<_> = (0..2)
            .map(|i| Node::new(NodeKind::Interface, 8))
            .collect();

        ifaces.iter_mut()
            .zip(patchbays.iter_mut())
            .enumerate()
            .for_each(|(i, (iface, pb))| {
                iface.inputs.iter_mut()
                    .zip(pb.outputs[NUM_INTERCONNECT_CHANNELS..].iter_mut())
                    .for_each(|(input, output)| {
                        //output.borrow_mut()
                        //    .input
                        //    .replace(input.clone());
                    });
                iface.outputs.iter_mut()
                    .zip(pb.inputs[NUM_INTERCONNECT_CHANNELS..].iter_mut())
                    .for_each(|(output, input)| {
                        //output.borrow_mut()
                        //    .input
                        //    .replace(input.clone());
                    });
            });

        // Build the channel strips, patch them into the last
        // channel of each
        let mut channel_strips = (0..NUM_CHANNEL_STRIPS)
            .map(|_| ["neve511", "dbx560a", "ssl611eq"]
                .iter()
                .map(|name| Node::new(NodeKind::Unit(AudioUnit::new(String::from(*name))), 1))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>();
        channel_strips.iter_mut()
            .for_each(|v| {
                //preamp.inputs[0].borrow_mut()
                //    .input
                //    .replace(reserve_on_patchbay(&mut patchbays));
            });
        astar(
            &ifaces[0].outputs[0],
            |io| io.deref().borrow().successors(),
            |io| 0,
            |io| {
                io == *channel_strips[0][0].inputs[0]
            },
        );
    }
}

fn reserve_on_patchbay(patchbays: &mut Vec<Node>) { //-> (Rc<RefCell<IO>>, Rc<RefCell<IO>>) {
}