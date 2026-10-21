// Persistent CPU accounts: charge the running ancestor chain, refill other
// accounts lazily, and reopen exhausted gates at indexed repayment deadlines.
// Admission, reaping and loan movement never clear credit, debt or reports.
use super::{deadlines::Deadlines, shares::Shares};
use alloc::{collections::TryReserveError, vec::Vec};
pub const PERIOD_US: u64 = cathedral_kernel_contracts::user::CPU_REFILL_PERIOD_US;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct CpuReport {
    pub direct_us: u64,
    pub subtree_us: u64,
    pub throttles: u64,
}
#[derive(Clone, Copy, Default)]
struct Account {
    limit: u64,
    // Microseconds scaled by PERIOD_US, retaining fractional refill credit.
    credit: i128,
    updated: u64,
    report: CpuReport,
}
pub(super) struct Budgets {
    accounts: Vec<Account>,
    repayments: Deadlines,
    last: Option<u64>,
    elapsed: u64,
    #[cfg(test)]
    pub(super) visits: usize,
}
impl Budgets {
    #[cfg(test)]
    pub(super) fn new(count: usize) -> Self {
        Self::try_new(count).unwrap()
    }
    pub(super) fn try_new(count: usize) -> Result<Self, TryReserveError> {
        let mut accounts = Vec::new();
        accounts.try_reserve_exact(count)?;
        accounts.resize(count, Account::default());
        Ok(Self {
            accounts,
            repayments: Deadlines::try_new(count)?,
            last: None,
            elapsed: 0,
            #[cfg(test)]
            visits: 0,
        })
    }
    pub(super) fn configure(&mut self, limits: &[u64]) -> bool {
        if self.last.is_some()
            || (!limits.is_empty() && limits.len() != self.accounts.len())
            || limits.iter().any(|&n| n > PERIOD_US)
        {
            return false;
        }
        for (account, &limit) in self.accounts.iter_mut().zip(limits) {
            account.limit = limit;
            account.credit = i128::from(limit) * i128::from(PERIOD_US);
        }
        true
    }
    pub(super) fn configured_gates(&mut self, shares: &mut Shares) {
        for slot in 0..self.accounts.len() {
            self.gate(slot, shares);
        }
    }
    fn refill(&mut self, slot: usize, now: u64) {
        #[cfg(test)]
        {
            self.visits += 1;
        }
        let account = &mut self.accounts[slot];
        if account.limit != 0 {
            account.credit = (account.credit
                + i128::from(now - account.updated) * i128::from(account.limit))
            .min(i128::from(account.limit) * i128::from(PERIOD_US));
        }
        account.updated = now;
    }
    fn gate(&mut self, slot: usize, shares: &mut Shares) {
        let account = &self.accounts[slot];
        let allowed = account.limit == 0 || account.credit > 0;
        shares.allow(slot, allowed);
        if allowed {
            self.repayments.remove(slot);
        } else {
            // Strictly positive credit is required, even at exact repayment.
            let delay = (-account.credit) as u128 / u128::from(account.limit) + 1;
            self.repayments.set(slot, u128::from(self.elapsed) + delay);
        }
    }
    pub(super) fn advance(&mut self, now: u64, running: Option<usize>, shares: &mut Shares) {
        let elapsed = self.last.replace(now).map_or(0, |last| {
            now.checked_sub(last)
                .expect("CPU accounting clock went backwards")
        });
        let previous = self.elapsed;
        self.elapsed += elapsed;
        if let Some(slot) = running {
            let direct = &mut self.accounts[slot].report.direct_us;
            *direct = direct.saturating_add(elapsed);
        }
        let mut ancestor = running;
        while let Some(slot) = ancestor {
            // First settle the preceding idle interval, then apply net running
            // refill/consumption. Capping a gross refill first loses valid credit.
            self.refill(slot, previous);
            let account = &mut self.accounts[slot];
            account.report.subtree_us = account.report.subtree_us.saturating_add(elapsed);
            if account.limit != 0 {
                let old = account.credit;
                let rate = i128::from(account.limit) - i128::from(PERIOD_US);
                account.credit = (old + i128::from(elapsed) * rate)
                    .min(i128::from(account.limit) * i128::from(PERIOD_US));
                if old > 0 && account.credit <= 0 {
                    account.report.throttles = account.report.throttles.saturating_add(1);
                }
            }
            account.updated = self.elapsed;
            self.gate(slot, shares);
            ancestor = shares.parent(slot);
        }
        while let Some(slot) = self.repayments.pop_due(u128::from(self.elapsed)) {
            self.refill(slot, self.elapsed);
            self.gate(slot, shares);
        }
    }
    pub(super) fn assign(&mut self, slot: usize, limit: u64, shares: &mut Shares) -> bool {
        if limit > PERIOD_US {
            return false;
        }
        self.refill(slot, self.elapsed);
        let account = &mut self.accounts[slot];
        account.limit = limit;
        account.credit = account
            .credit
            .min(i128::from(limit) * i128::from(PERIOD_US));
        self.gate(slot, shares);
        true
    }
    pub(super) fn report(&self, slot: usize) -> CpuReport {
        self.accounts[slot].report
    }
}
