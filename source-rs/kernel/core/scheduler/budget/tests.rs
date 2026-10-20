use super::*;
use crate::scheduler::{Event, Scheduler};

#[test]
fn limits_validate_before_mutation_and_cannot_be_reconfigured_after_execution() {
    let mut budgets = Budgets::new(2);
    assert!(!budgets.configure(&[1]));
    assert!(!budgets.configure(&[0, PERIOD_US + 1]));
    assert!(budgets.configure(&[20_000, 0]));
    let mut shares = Shares::new(2);
    budgets.advance(0, None, &mut shares);
    assert!(!budgets.configure(&[PERIOD_US; 2]));
    assert_eq!(budgets.accounts[0].limit, 20_000);
}
#[test]
fn partial_execution_is_charged_with_fractional_refill_preserved() {
    let mut shares = Shares::new(1);
    let mut short = Budgets::new(1);
    let mut long = Budgets::new(1);
    for budgets in [&mut short, &mut long] {
        assert!(budgets.configure(&[12_345]));
        budgets.advance(0, None, &mut shares);
    }
    for us in 1..=10_000 {
        short.advance(us, Some(0), &mut shares);
    }
    long.advance(10_000, Some(0), &mut shares);
    assert_eq!(short.accounts[0].credit, long.accounts[0].credit);
    assert_eq!(short.report(0).direct_us, 10_000);
    assert_eq!(short.report(0).subtree_us, 10_000);
}
#[test]
fn overrun_debt_survives_restart_and_idle_refill_has_a_burst_ceiling() {
    let mut scheduler = Scheduler::new(1);
    assert!(scheduler.configure_cpu(&[20_000]));
    scheduler.account_cpu(0);
    scheduler.admit(0);
    assert_eq!(scheduler.advance(Event::Yield, 0, true), Some(0));
    scheduler.account_cpu(35_000); // 25 ms empties the bucket; another 10 ms is debt.
    assert_eq!(scheduler.advance(Event::Continue, 0, true), None);
    assert!(!scheduler.has_eligible_ready());
    scheduler.cancel(0);
    scheduler.reap(0);
    scheduler.admit(0);
    assert_eq!(scheduler.advance(Event::Yield, 0, true), None);
    scheduler.account_cpu(75_000); // Debt paid exactly; no spendable credit yet.
    assert!(!scheduler.has_eligible_ready());
    scheduler.account_cpu(75_001);
    assert!(scheduler.has_eligible_ready());
    assert_eq!(scheduler.cpu_report(0).direct_us, 35_000);
    assert_eq!(scheduler.cpu_report(0).throttles, 1);
    scheduler.account_cpu(1_000_000);
    scheduler.budgets.refill(0, scheduler.budgets.elapsed);
    assert_eq!(
        scheduler.budgets.accounts[0].credit,
        20_000 * i128::from(PERIOD_US)
    );
}
#[test]
fn every_ancestor_is_charged_and_either_ancestor_can_gate_a_descendant() {
    let mut shares = Shares::new(4);
    shares.set_parent(2, 0);
    shares.set_parent(3, 2);
    let mut budgets = Budgets::new(4);
    assert!(budgets.configure(&[20_000, 0, 5_000, 0]));
    budgets.advance(0, None, &mut shares);
    budgets.advance(6_000, Some(3), &mut shares);
    assert_eq!(
        (0..4).map(|slot| shares.eligible(slot)).collect::<Vec<_>>(),
        [true, true, false, false]
    );
    assert_eq!(budgets.report(0).subtree_us, 6_000);
    assert_eq!(budgets.report(2).subtree_us, 6_000);
    assert_eq!(budgets.report(3).direct_us, 6_000);
    assert_eq!(budgets.report(1).subtree_us, 0);
    budgets.advance(36_000, Some(0), &mut shares);
    assert_eq!(
        (0..4).map(|slot| shares.eligible(slot)).collect::<Vec<_>>(),
        [false, true, false, false]
    );
}
#[test]
fn yields_and_new_descendants_cannot_renew_an_exhausted_parent() {
    let mut scheduler = Scheduler::new(4);
    assert!(scheduler.configure_cpu(&[20_000, 0, 0, 0]));
    scheduler.set_parent(2, 0);
    scheduler.set_parent(3, 2);
    for slot in 0..3 {
        scheduler.admit(slot);
    }
    scheduler.account_cpu(0);
    for us in 0..80_000 {
        scheduler.advance(Event::Yield, 0, true);
        scheduler.account_cpu(us + 1);
    }
    let used = scheduler.cpu_report(0).subtree_us;
    assert!(used <= 20_000 + 80_000 / 5 + 1);
    assert!(scheduler.cpu_report(1).direct_us > used);
    scheduler.park(Event::Yield, 0);
    scheduler.admit(3);
    assert_eq!(scheduler.shares.eligible(3), scheduler.shares.eligible(0));
    assert_eq!(scheduler.cpu_report(0).subtree_us, used);
}
#[test]
fn boot_work_and_reparenting_keep_existing_debt_with_the_account() {
    let mut scheduler = Scheduler::new(3);
    assert!(scheduler.configure_cpu(&[0, 0, 10_000]));
    scheduler.set_parent(2, 0);
    scheduler.account_cpu(0);
    scheduler.charge_cpu(2, 20_000); // Deferred work can finish with no runnable task.
    assert_eq!(scheduler.cpu_report(0).subtree_us, 20_000);
    let debt = scheduler.budgets.accounts[2].credit;
    assert!(debt < 0);
    scheduler.set_parent(2, 1);
    scheduler.admit(2);
    assert!(!scheduler.has_eligible_ready());
    assert_eq!(scheduler.budgets.accounts[2].credit, debt);
    assert_eq!(scheduler.cpu_report(1).subtree_us, 0);
    scheduler.account_cpu(120_000);
    assert!(scheduler.has_eligible_ready());
    scheduler.advance(Event::Yield, 0, true);
    scheduler.account_cpu(120_500);
    assert_eq!(scheduler.cpu_report(1).subtree_us, 500);
    assert_eq!(scheduler.cpu_report(0).subtree_us, 20_000);
}

#[test]
fn dormant_rebudgeting_preserves_debt_and_cannot_expand_an_ancestor() {
    let mut scheduler = Scheduler::new(2);
    scheduler.set_parent(1, 0);
    assert!(scheduler.configure_cpu(&[20_000, 10_000]));
    scheduler.account_cpu(0);
    scheduler.charge_cpu(1, 40_000);
    let debt = scheduler.budgets.accounts[1].credit;
    assert!(debt < 0);
    assert!(scheduler.assign_cpu(1, 0));
    assert_eq!(scheduler.budgets.accounts[1].credit, debt);
    assert!(scheduler.assign_cpu(1, 100_000));
    assert_eq!(scheduler.budgets.accounts[1].credit, debt);
    assert!(!scheduler.shares.eligible(1));
    assert!(!scheduler.assign_cpu(1, 100_001));
    scheduler.admit(1);
    assert!(!scheduler.assign_cpu(1, 20_000));
    assert_eq!(scheduler.cpu_report(0).subtree_us, 40_000);
}

#[test]
fn lazy_accounting_matches_eager_reference_through_rebinding_and_rate_changes() {
    // Independent eager arithmetic: settle every account at every sample, as in
    // the previous policy. Do not force the lazy implementation to materialize
    // idle credit while checking it, or the test would hide missing lazy refill.
    let mut shares = Shares::new(8);
    let mut budgets = Budgets::new(8);
    let mut parents = [
        None,
        None,
        Some(0),
        Some(0),
        Some(2),
        Some(1),
        Some(3),
        Some(4),
    ];
    let mut limits = [20_000, 0, 5_000, 100_000, 1, 0, 37_101, 10_000];
    let mut credit = limits.map(|limit| i128::from(limit) * i128::from(PERIOD_US));
    let mut reports = [CpuReport::default(); 8];
    for (slot, parent) in parents.iter().enumerate() {
        if let Some(parent) = parent {
            shares.set_parent(slot, *parent);
        }
    }
    budgets.configure(&limits);
    budgets.advance(1234, None, &mut shares);
    let mut now = 1234;
    let mut random = 17u64;
    for step in 0..10_000 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let elapsed = (random >> 32) % 30_000;
        let running = ((random >> 16) % 10 < 8).then_some((random >> 16) as usize % 8);
        now += elapsed;
        let mut charged = [false; 8];
        let mut ancestor = running;
        while let Some(slot) = ancestor {
            charged[slot] = true;
            ancestor = parents[slot];
        }
        for slot in 0..8 {
            if running == Some(slot) {
                reports[slot].direct_us += elapsed;
            }
            if charged[slot] {
                reports[slot].subtree_us += elapsed;
            }
            if limits[slot] != 0 {
                let old = credit[slot];
                let rate = i128::from(limits[slot])
                    - if charged[slot] {
                        i128::from(PERIOD_US)
                    } else {
                        0
                    };
                credit[slot] = (old + i128::from(elapsed) * rate)
                    .min(i128::from(limits[slot]) * i128::from(PERIOD_US));
                if old > 0 && credit[slot] <= 0 {
                    reports[slot].throttles += 1;
                }
            }
        }
        budgets.advance(now, running, &mut shares);
        if step % 13 == 0 {
            let slot = random as usize % 8;
            limits[slot] = [0, 1, 5_000, 100_000][(random >> 8) as usize % 4];
            credit[slot] = credit[slot].min(i128::from(limits[slot]) * i128::from(PERIOD_US));
            assert!(budgets.assign(slot, limits[slot], &mut shares));
        }
        if step % 29 == 0 {
            parents[7] = Some((random >> 24) as usize % 7);
            shares.set_parent(7, parents[7].unwrap());
        }
        for slot in 0..8 {
            let account = budgets.accounts[slot];
            let actual = if account.limit == 0 {
                account.credit
            } else {
                (account.credit
                    + i128::from(budgets.elapsed - account.updated) * i128::from(account.limit))
                .min(i128::from(account.limit) * i128::from(PERIOD_US))
            };
            assert_eq!(actual, credit[slot], "credit step {step}, slot {slot}");
            assert_eq!(
                budgets.report(slot),
                reports[slot],
                "reports step {step}, slot {slot}"
            );
            let mut allowed = true;
            let mut ancestor = Some(slot);
            while let Some(slot) = ancestor {
                allowed &= limits[slot] == 0 || credit[slot] > 0;
                ancestor = parents[slot];
            }
            assert_eq!(
                shares.eligible(slot),
                allowed,
                "gate step {step}, slot {slot}"
            );
        }
    }
}

#[test]
fn idle_refill_is_materialized_before_changing_a_dormant_rate() {
    let mut scheduler = Scheduler::new(1);
    scheduler.configure_cpu(&[20_000]);
    scheduler.account_cpu(0);
    scheduler.charge_cpu(0, 30_000);
    scheduler.account_cpu(200_000);
    assert!(scheduler.assign_cpu(0, 40_000));
    assert_eq!(
        scheduler.budgets.accounts[0].credit,
        20_000 * i128::from(PERIOD_US)
    );
}
