use pathfinding::prelude::{astar};
use std::ops::Deref;
use std::sync::{
    Arc,
    Mutex,
    RwLock,
};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
mod node;

use node::{
    Node,
    NodeHandle,
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
    fn advanced_routing() {
        const NUM_PATCHBAYS: usize = 3;
        const NUM_CHANNELS: usize = 64;
        const NUM_INTERCONNECT_CHANNELS: usize = 16;
        const NUM_UNITS: usize = 4;
        const NUM_CHANNEL_STRIPS: usize = 256;

        let mut patchbays: Vec<NodeHandle> = (0..NUM_PATCHBAYS)
            .map(|_| Arc::new(RwLock::new(Node::make(NodeKind::Patchbay, vec![], vec![]))))
            .collect();

        let mut ifaces: Vec<_> = (0..2)
            .map(|i| Node::new(NodeKind::Interface, 8))
            .collect();

        let mut patchbays = patchbays.into_iter()
            .map(|mut pb| {
                {
                    let mut l = pb.write().unwrap();
                    l.inputs = (0..NUM_CHANNELS)
                        .map(|j| IO::new(j as u8, false, None, Arc::downgrade(&pb)))
                        .collect::<Vec<_>>();
                    l.outputs = (0..NUM_CHANNELS)
                        .map(|j| IO::new(j as u8, true, None, Arc::downgrade(&pb)))
                        .collect::<Vec<_>>();
                }
                pb
            })
            .collect::<Vec<_>>();

        // Connect the first handful of channels to the next
        // patchbay. The first unit has unused input channels
        // and the last has as many unused outputs.
        for i in 0..patchbays.len() - 1 {
            let (a, b) = patchbays[i..i + 2].split_at_mut(1);
            a[0].write().unwrap().outputs[..NUM_INTERCONNECT_CHANNELS].iter_mut()
                .zip(b[0].write().unwrap().inputs[..NUM_INTERCONNECT_CHANNELS].iter_mut())
                .for_each(|(output, input)| output.write().unwrap().input = Some(input.clone()));
            //a[0].write().unwrap().inputs[..NUM_INTERCONNECT_CHANNELS].iter_mut()
            //    .zip(b[0].write().unwrap().outputs[..NUM_INTERCONNECT_CHANNELS].iter_mut())
            //    .for_each(|(input, output)| output.write().unwrap().input = Some(input.clone()));
        }

    /*
        ifaces.iter_mut()
            .zip(patchbays.iter_mut())
            .enumerate()
            .for_each(|(i, (iface, pb))| {
                let mut iface = iface.write().unwrap();
                let mut pb = pb.write().unwrap();
                iface.inputs.iter_mut()
                    .zip(pb.outputs[NUM_INTERCONNECT_CHANNELS..].iter_mut())
                    .for_each(|(output, input)| output.write().unwrap().input = Some(input.clone()));
                iface.outputs.iter_mut()
                    .zip(pb.inputs[NUM_INTERCONNECT_CHANNELS..].iter_mut())
                    .for_each(|(output, input)| output.write().unwrap().input = Some(input.clone()));
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
                    // TODO: assign IO on the patchbays
                    //preamp.inputs[0].borrow_mut()
                    //    .input
                    //    .replace(reserve_on_patchbay(&mut patchbays));
                });

         */

        assert!(astar(
            &patchbays[0].read().unwrap().outputs[0],
            |io| io.deref().read().unwrap().successors(),
            |io| 0,
            |io| *io == patchbays.last().unwrap().read().unwrap().inputs[0],
        ).is_some());
    }
}

fn reserve_on_patchbay(patchbays: &mut Vec<Node>) { //-> (Rc<RefCell<IO>>, Rc<RefCell<IO>>) {
}