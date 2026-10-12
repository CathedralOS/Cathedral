use super::*;
use crate::scheduler::{Event, Scheduler};

#[test]
fn limits_validate_before_mutation_and_cannot_be_reconfigured_after_execution() {
    let mut budgets = Budgets::new(2);
    assert!(!budgets.configure(&[1]));
    assert!(!budgets.configure(&[0, PERIOD_US + 1]));
    assert!(budgets.configure(&[20_000, 0]));
    let shares = Shares::new(2);
    budgets.advance(0, None, &shares);
    assert!(!budgets.configure(&[PERIOD_US; 2]));
    assert_eq!(budgets.accounts[0].limit, 20_000);
}
#[test]
fn partial_execution_is_charged_with_fractional_refill_preserved() {
    let shares = Shares::new(1);
    let mut short = Budgets::new(1);
    let mut long = Budgets::new(1);
    for budgets in [&mut short, &mut long] {
        assert!(budgets.configure(&[12_345]));
        budgets.advance(0, None, &shares);
    }
    for us in 1..=10_000 {
        short.advance(us, Some(0), &shares);
    }
    long.advance(10_000, Some(0), &shares);
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
    budgets.advance(0, None, &shares);
    budgets.advance(6_000, Some(3), &shares);
    assert_eq!(budgets.eligible, [true, true, false, false]);
    assert_eq!(budgets.report(0).subtree_us, 6_000);
    assert_eq!(budgets.report(2).subtree_us, 6_000);
    assert_eq!(budgets.report(3).direct_us, 6_000);
    assert_eq!(budgets.report(1).subtree_us, 0);
    budgets.advance(36_000, Some(0), &shares);
    assert_eq!(budgets.eligible, [false, true, false, false]);
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
    assert_eq!(scheduler.budgets.eligible[3], scheduler.budgets.eligible[0]);
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
    assert!(!scheduler.budgets.eligible[1]);
    assert!(!scheduler.assign_cpu(1, 100_001));
    scheduler.admit(1);
    assert!(!scheduler.assign_cpu(1, 20_000));
    assert_eq!(scheduler.cpu_report(0).subtree_us, 40_000);
}
