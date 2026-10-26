//! Full arenas, recipient-local escrow and restart must survive scoped withdrawal.
use super::*;
use alloc::vec::Vec;

const VICTIM: usize = abi::MAX_TASKS - 1;

fn graph() -> [EndpointSpec; MAX_ENDPOINTS] {
    core::array::from_fn(|index| {
        let peer = (index / 2) % VICTIM;
        EndpointSpec {
            sender: if index % 2 == 0 { peer } else { VICTIM },
            receiver: if index % 2 == 0 { VICTIM } else { peer },
            revoker: Some((peer + 1) % VICTIM),
        }
    })
}

fn local_grants(ipc: &Ipc, task: usize) -> Vec<(u64, usize, Right, bool)> {
    ipc.grants.entries[Grants::dynamic_slots(task)]
        .iter()
        .flatten()
        .map(|grant| {
            assert_eq!(grant.owner, task);
            (grant.ticket, grant.endpoint, grant.right, grant.visible)
        })
        .collect()
}

fn full(ipc: &Ipc) {
    assert_eq!(
        ipc.endpoints[MAX_ENDPOINTS..].iter().flatten().count(),
        abi::MAX_DYNAMIC_CHANNELS
    );
    for (slot, grant) in ipc
        .grants
        .entries
        .iter()
        .enumerate()
        .filter_map(|(slot, grant)| grant.map(|grant| (slot, grant)))
    {
        assert!(slot < MAX_ENDPOINTS * 3 || Grants::dynamic_slots(grant.owner).contains(&slot));
    }
}

#[test]
fn full_channel_arena_closes_last_task_preserves_peers_and_restarts_with_fresh_names() {
    let mut limits = [0; abi::MAX_TASKS];
    limits[VICTIM] = abi::MAX_DYNAMIC_HANDLES / 2;
    let mut remaining = abi::MAX_DYNAMIC_CHANNELS - limits[VICTIM];
    for limit in &mut limits[..VICTIM] {
        *limit = remaining.min(9);
        remaining -= *limit;
    }
    assert_eq!(remaining, 0);
    let mut ipc = Ipc::with_reservations(1, abi::MAX_TASKS, &graph(), &limits).unwrap();
    let mut peers = Vec::new();
    let mut old = Vec::new();
    for (task, &limit) in limits.iter().enumerate() {
        for index in 0..limit {
            let [send, receive] = ipc.create(task).unwrap();
            if task == VICTIM {
                ipc.send_memory(task, send, b"discard", index);
                old.extend([send, receive]);
            } else {
                ipc.send(task, send, b"preserve").unwrap();
                peers.push((task, send, receive));
            }
        }
    }
    full(&ipc);
    assert_eq!(
        ipc.budget(VICTIM).unwrap()[3],
        abi::MAX_DYNAMIC_HANDLES as u64
    );
    let before: Vec<_> = (0..VICTIM).map(|task| local_grants(&ipc, task)).collect();
    let old_revoker = ipc.grants.entries[2].unwrap();
    ipc.close_task(VICTIM);
    assert!(local_grants(&ipc, VICTIM).is_empty());
    assert_eq!(ipc.charged_channels(VICTIM), 0);
    let discarded = ipc.take_retired_memory();
    assert_eq!(discarded[0], (1u64 << limits[VICTIM]) - 1);
    assert!(discarded[1..].iter().all(|&word| word == 0));
    for (task, expected) in before.iter().enumerate() {
        assert_eq!(&local_grants(&ipc, task), expected);
    }
    for &(task, send, receive) in &peers {
        ipc.check(task, send, Right::Send).unwrap();
        let message = ipc.receive(task, receive, 64).unwrap().unwrap();
        assert_eq!(&message.bytes[..message.len], b"preserve");
    }
    ipc.prepare_child(0, 0, VICTIM, 2);
    ipc.accept_child(0, 0);
    assert_eq!(
        ipc.revoke(old_revoker.owner, old_revoker.ticket),
        Err(abi::BAD_HANDLE)
    );
    let [send, receive] = ipc.create(VICTIM).unwrap();
    for ticket in old {
        assert_eq!(
            ipc.grants.lookup(VICTIM, ticket).map(|_| ()),
            Err(abi::BAD_HANDLE)
        );
    }
    ipc.send(VICTIM, send, b"new incarnation").unwrap();
    assert_eq!(ipc.receive(VICTIM, receive, 64).unwrap().unwrap().len, 15);
    assert!(ipc.initial(VICTIM).unwrap().is_empty());
    for task in 0..abi::MAX_TASKS {
        ipc.close_task(task);
    }
    assert!(ipc.endpoints[MAX_ENDPOINTS..].iter().all(Option::is_none));
}

#[test]
fn full_hidden_receive_chain_closes_without_consuming_unrelated_channels() {
    let mut limits = [0; abi::MAX_TASKS];
    limits[0] = abi::MAX_DYNAMIC_HANDLES;
    let mut remaining = abi::MAX_DYNAMIC_CHANNELS - limits[0];
    for limit in &mut limits[1..VICTIM] {
        *limit = remaining.min(9);
        remaining -= *limit;
    }
    assert_eq!(remaining, 0);
    let mut ipc = Ipc::with_reservations(1, abi::MAX_TASKS, &graph(), &limits).unwrap();
    let transport = ipc.handle(0, 0).unwrap();
    let mut previous = None;
    for _ in 0..abi::MAX_DYNAMIC_HANDLES {
        let [send, receive] = ipc.create(0).unwrap();
        ipc.send_cap(
            0,
            previous.unwrap_or(transport),
            b"queued receive",
            Some(receive),
        )
        .unwrap();
        if let Some(previous) = previous {
            ipc.close(0, previous).unwrap();
        }
        previous = Some(send);
    }
    let last = previous.unwrap();
    let escrow = cathedral_kernel_contracts::memory::MAX_READ_HANDLES - 1;
    ipc.send_memory(0, last, b"tail", escrow);
    let mut peers = Vec::new();
    for (task, &limit) in limits.iter().enumerate().skip(1).take(VICTIM - 1) {
        for _ in 0..limit {
            let [send, receive] = ipc.create(task).unwrap();
            ipc.send(task, send, b"preserve").unwrap();
            peers.push((task, send, receive));
        }
    }
    full(&ipc);
    let hidden = local_grants(&ipc, VICTIM);
    assert_eq!(hidden.len(), abi::MAX_DYNAMIC_HANDLES);
    assert!(
        hidden
            .iter()
            .all(|&(_, _, right, visible)| right == Right::Receive && !visible)
    );
    assert_eq!(ipc.budget(VICTIM).unwrap()[3], hidden.len() as u64);
    ipc.close_task(VICTIM);
    assert!(local_grants(&ipc, VICTIM).is_empty());
    assert_eq!(ipc.charged_channels(0), 1); // Only the retained tail send survives.
    assert_eq!(ipc.send(0, last, b"closed"), Err(abi::PEER_CLOSED));
    let retired = ipc.take_retired_memory();
    for (word, &bits) in retired.iter().enumerate() {
        assert_eq!(
            bits,
            if word == escrow / 64 {
                1 << (escrow % 64)
            } else {
                0
            }
        );
    }
    for (task, send, receive) in peers {
        ipc.check(task, send, Right::Send).unwrap();
        let message = ipc.receive(task, receive, 64).unwrap().unwrap();
        assert_eq!(&message.bytes[..message.len], b"preserve");
    }
    for task in 0..abi::MAX_TASKS {
        ipc.close_task(task);
    }
    assert!(ipc.endpoints[MAX_ENDPOINTS..].iter().all(Option::is_none));
    assert!(!ipc.memory_retirement_pending());
}
