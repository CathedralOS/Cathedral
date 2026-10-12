//! Persistent CPU accounts: measured execution consumes an ancestor chain.
//! Each configured account has a burst allowance and a continuous refill rate.
//! Overruns leave debt; neither admission, reaping nor loan movement clears it.
use super::shares::Shares;
use alloc::{vec, vec::Vec};
pub const PERIOD_US: u64 = cathedral_contracts::user::CPU_REFILL_PERIOD_US;

#[derive(Clone, Copy, Default, Debug)]
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
    charged: bool,
    report: CpuReport,
}
pub(super) struct Budgets {
    accounts: Vec<Account>,
    pub(super) eligible: Vec<bool>,
    last: Option<u64>,
}
impl Budgets {
    pub(super) fn new(count: usize) -> Self {
        Self {
            accounts: vec![Account::default(); count],
            eligible: vec![true; count],
            last: None,
        }
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
    pub(super) fn advance(&mut self, now: u64, running: Option<usize>, shares: &Shares) {
        let elapsed = self.last.replace(now).map_or(0, |last| {
            now.checked_sub(last)
                .expect("CPU accounting clock went backwards")
        });
        for account in &mut self.accounts {
            account.charged = false;
        }
        if let Some(slot) = running {
            let direct = &mut self.accounts[slot].report.direct_us;
            *direct = direct.saturating_add(elapsed);
        }
        let mut ancestor = running;
        while let Some(slot) = ancestor {
            self.accounts[slot].charged = true;
            ancestor = shares.parent(slot);
        }
        for account in &mut self.accounts {
            if account.charged {
                account.report.subtree_us = account.report.subtree_us.saturating_add(elapsed);
            }
            if account.limit != 0 {
                let old = account.credit;
                let rate = i128::from(account.limit)
                    - if account.charged {
                        i128::from(PERIOD_US)
                    } else {
                        0
                    };
                // Net refill/consumption avoids dropping refill while running at
                // the burst ceiling. Debt survives until enough real time elapses.
                account.credit = (old + i128::from(elapsed) * rate)
                    .min(i128::from(account.limit) * i128::from(PERIOD_US));
                if old > 0 && account.credit <= 0 {
                    account.report.throttles = account.report.throttles.saturating_add(1);
                }
            }
        }
        self.refresh(shares);
    }
    pub(super) fn refresh(&mut self, shares: &Shares) {
        for slot in 0..self.accounts.len() {
            let account = &self.accounts[slot];
            self.eligible[slot] = (account.limit == 0 || account.credit > 0)
                && shares
                    .parent(slot)
                    .is_none_or(|parent| self.eligible[parent]);
        }
    }
    pub(super) fn report(&self, slot: usize) -> CpuReport {
        self.accounts[slot].report
    }
}

#[cfg(test)]
mod tests;
