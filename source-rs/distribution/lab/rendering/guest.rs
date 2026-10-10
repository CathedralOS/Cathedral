//! Actual page-backed scalar rendering and a deliberate hardware boundary probe.
pub(crate) mod bounds;
use cathedral_rendering_lab::{
    pixels::{self, Source},
    scene,
};
use cathedral_user_runtime::{
    ipc::Handle,
    link::Link,
    memory::{Private, Sealed, Shared},
    task::{Launch, Outcome, Port},
};
pub fn supervise() -> u64 {
    let producer = Launch::at(0).unwrap().spawn(0).unwrap();
    let consumer = Launch::at(1).unwrap().spawn(0).unwrap();
    // Both peers must be admitted before a link can be connected or a page offered.
    for index in 0..2 {
        Port::at(index)
            .unwrap()
            .connect()
            .unwrap()
            .0
            .send(&[0; 8])
            .unwrap();
    }
    assert_eq!(producer.wait().unwrap(), Outcome::Returned(0));
    assert_eq!(consumer.wait().unwrap(), Outcome::Returned(0));
    let guard = Launch::at(2).unwrap().spawn(0).unwrap();
    let Outcome::Fault {
        vector,
        error,
        address,
        ..
    } = guard.wait().unwrap()
    else {
        panic!()
    };
    assert_eq!(vector, 14);
    assert_eq!(error & 31, 6);
    // All earlier allocations were released: the fixture reuses region slot zero.
    assert_eq!(address, cathedral_contracts::memory::address(0) + 4096);
    cathedral_user_runtime::write(b"Cathedral: rendering rows and shared leaves agree; same-page native escape observed; page boundary fault contained\n").unwrap();
    0
}
fn ready() {
    assert_eq!(
        Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap(),
        8
    );
}
pub fn produce() -> u64 {
    ready();
    let link = Link::at(0).unwrap();
    let (send, receive) = link.connect().unwrap();
    let (nodes, _) = scene::fixture(64, 16, 4).unwrap();
    let leaves: [Sealed; 4] = core::array::from_fn(|i| {
        let mut page = Private::allocate(1).unwrap();
        let rect = nodes[4 + i].rect;
        assert_eq!(
            pixels::leaf(page.bytes_mut(), rect, rect.width * 4),
            Ok(1024)
        );
        page.seal(link).unwrap()
    });
    let mut offer = [0; 32];
    for (leaf, bytes) in leaves.iter().zip(offer.chunks_exact_mut(8)) {
        bytes.copy_from_slice(&leaf.handle().to_le_bytes());
    }
    send.send(&offer).unwrap();
    let mut completion = [0; 64];
    assert_eq!(receive.receive(&mut completion).unwrap(), 8);
    assert_eq!(&completion[..8], &4096u64.to_le_bytes());
    for leaf in leaves {
        leaf.release().unwrap();
    }
    0
}
pub fn compose() -> u64 {
    ready();
    bounds::rows();
    let (receive, send) = Link::at(0).unwrap().connect().unwrap();
    let mut offer = [0; 64];
    assert_eq!(receive.receive(&mut offer).unwrap(), 32);
    let leaves: [Shared; 4] = core::array::from_fn(|i| {
        Shared::accept(u64::from_le_bytes(
            offer[i * 8..(i + 1) * 8].try_into().unwrap(),
        ))
        .unwrap()
    });
    let sources = core::array::from_fn(|i| Source {
        bytes: leaves[i].bytes(),
        stride: 128,
    });
    let (nodes, count) = scene::fixture(64, 16, 4).unwrap();
    let plan = scene::resolve(&nodes[..count], 64, 16).unwrap();
    let mut output = Private::allocate(1).unwrap();
    assert_eq!(
        pixels::compose(&plan, &sources, output.bytes_mut()),
        Ok(4096)
    );
    assert!(pixels::verify(output.bytes_mut(), 64, 16));
    drop(output);
    drop(leaves);
    // Completion is sent only after dropping every accepted reader.
    send.send(&4096u64.to_le_bytes()).unwrap();
    0
}
