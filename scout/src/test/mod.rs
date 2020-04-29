#[cfg(test)]
mod lua;

use super::*;
#[test]
fn basic() {
    const NUM_PATCHBAYS: usize = 3;
    const NUM_CHANNELS: usize = 32;
    const NUM_INTERCONNECT_CHANNELS: usize = 4;
    let mut patchbays: Vec<Box<Node>> = (0..NUM_PATCHBAYS)
        .map(|_| Node::make(Uuid::new_v4(), NodeKind::Patchbay, vec![], vec![]))
        .collect();
    let mut ifaces: Vec<Box<Node>> = (0..2)
        .map(|i| Node::new(NodeKind::Interface, 8))
        .collect();
    let mut patchbays = patchbays.into_iter()
        .map(|mut pb| {
            pb.inputs = (0..NUM_CHANNELS)
                .map(|j| IO::new(Uuid::new_v4(), j as u8, false, None, &*pb as _, Default::default()))
                .collect::<Vec<_>>();
            pb.outputs = (0..NUM_CHANNELS)
                .map(|j| IO::new(Uuid::new_v4(), j as u8, true, None, &*pb as _, Default::default()))
                .collect::<Vec<_>>();
            pb
        })
        .collect::<Vec<_>>();
    // Connect the first handful of channels to the next
    // patchbay. The first unit has unused input channels
    // and the last has as many unused outputs.
    for i in 0..patchbays.len() - 1 {
        let (a, b) = patchbays[i..i + 2].split_at_mut(1);
        a[0].outputs[..NUM_INTERCONNECT_CHANNELS].iter_mut()
            .zip(b[0].inputs[..NUM_INTERCONNECT_CHANNELS].iter_mut())
            .for_each(|(output, input)| output.input = Some(&**input as _));
        a[0].inputs[..NUM_INTERCONNECT_CHANNELS].iter_mut()
            .zip(b[0].outputs[..NUM_INTERCONNECT_CHANNELS].iter_mut())
            .for_each(|(input, output)| output.input = Some(&**input as _));
    }
    assert!(patchbays[0].inputs[0] == patchbays[0].inputs[0]);
    assert!(patchbays[0].inputs[0] != patchbays[0].inputs[1]);
    assert!(astar(
        &(&*patchbays[0].inputs[0] as *const IO),
        |io| unsafe { (**io).successors() },
        |io| 0,
        |io| unsafe { (**io).uid == patchbays.last().unwrap().inputs[0].uid },
    ).is_some());
}

#[test]
fn advanced_routing() {
    const NUM_PATCHBAYS: usize = 3;
    const NUM_CHANNELS: usize = 32;
    const NUM_INTERCONNECT_CHANNELS: usize = 4;
    //const NUM_UNITS: usize = 4;
    //const NUM_CHANNEL_STRIPS: usize = 256;

    let mut patchbays: Vec<Box<Node>> = (0..NUM_PATCHBAYS)
        .map(|_| Node::make(Uuid::new_v4(), NodeKind::Patchbay, vec![], vec![]))
        .collect();

    let mut ifaces: Vec<Box<Node>> = (0..2)
        .map(|i| Node::new(NodeKind::Interface, 8))
        .collect();

    let mut patchbays = patchbays.into_iter()
        .map(|mut pb| {
            pb.inputs = (0..NUM_CHANNELS)
                .map(|j| IO::new(Uuid::new_v4(), j as u8, false, None, &*pb as _, Default::default()))
                .collect::<Vec<_>>();
            pb.outputs = (0..NUM_CHANNELS)
                .map(|j| IO::new(Uuid::new_v4(), j as u8, true, None, &*pb as _, Default::default()))
                .collect::<Vec<_>>();
            pb
        })
        .collect::<Vec<_>>();

    // Connect the first handful of channels to the next
    // patchbay. The first unit has unused input channels
    // and the last has as many unused outputs.
    for i in 0..patchbays.len() - 1 {
        let (a, b) = patchbays[i..i + 2].split_at_mut(1);
        a[0].outputs[..NUM_INTERCONNECT_CHANNELS].iter_mut()
            .zip(b[0].inputs[..NUM_INTERCONNECT_CHANNELS].iter_mut())
            .for_each(|(output, input)| output.input = Some(&**input as _));
        a[0].inputs[..NUM_INTERCONNECT_CHANNELS].iter_mut()
            .zip(b[0].outputs[..NUM_INTERCONNECT_CHANNELS].iter_mut())
            .for_each(|(input, output)| output.input = Some(&**input as _));
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

    assert!(patchbays[0].inputs[0] == patchbays[0].inputs[0]);
    assert!(patchbays[0].inputs[0] != patchbays[0].inputs[1]);
    assert!(astar(
        &(&*patchbays[0].inputs[0] as *const IO),
        |io| unsafe { (**io).successors() },
        |io| 0,
        |io| unsafe { (**io).uid == patchbays.last().unwrap().inputs[0].uid },
    ).is_some());
}