//! Validate process capacity, channel reservations and CPU ceilings before generating grants.
use std::env;
pub(super) struct Budget {
    pub count: usize,
    pub dynamic: usize,
    pub limits: Vec<usize>,
    pub cpu_limits: Vec<u64>,
}
pub(super) fn read() -> Budget {
    let count: usize = env::var("CATHEDRAL_LAUNCH_COUNT")
        .unwrap_or_else(|_| "0".into())
        .parse()
        .unwrap();
    assert!(
        count <= cathedral_contracts::user::MAX_LAUNCHES,
        "launch arena exhausted"
    );
    let dynamic: usize = env::var("CATHEDRAL_DYNAMIC_TASKS")
        .unwrap_or_else(|_| "0".into())
        .parse()
        .unwrap();
    let accounts = count
        .checked_add(1)
        .and_then(|n| n.checked_add(dynamic))
        .expect("account overflow");
    assert!(
        accounts <= cathedral_contracts::user::MAX_TASKS,
        "process pool exhausted"
    );
    let limits: Vec<usize> = env::var("CATHEDRAL_CHANNEL_LIMITS")
        .map(|text| {
            text.split(',')
                .map(|n| n.parse().expect("invalid channel reservation"))
                .collect()
        })
        .unwrap_or_else(|_| {
            let mut limits = vec![cathedral_contracts::user::DEFAULT_CHANNELS; count + 1];
            limits.resize(accounts, 0);
            limits
        });
    assert!(
        limits.len() == accounts
            && limits
                .iter()
                .all(|&n| n <= cathedral_contracts::user::MAX_CREATED_CHANNELS),
        "invalid channel reservations"
    );
    assert!(
        limits.iter().sum::<usize>() <= cathedral_contracts::user::MAX_DYNAMIC_CHANNELS,
        "channel reservations exceed backing"
    );
    let cpu_limits: Vec<u64> = env::var("CATHEDRAL_CPU_LIMITS")
        .map(|text| {
            text.split(',')
                .map(|n| n.parse().expect("invalid CPU limit"))
                .collect()
        })
        .unwrap_or_else(|_| vec![0; accounts]);
    assert!(
        cpu_limits.len() == accounts
            && cpu_limits
                .iter()
                .all(|&n| n <= cathedral_contracts::user::CPU_REFILL_PERIOD_US),
        "invalid CPU burst/refill limit"
    );
    Budget {
        count,
        dynamic,
        limits,
        cpu_limits,
    }
}
