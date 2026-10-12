// Conserved frame funding. Actual frames are held by the session's frame bank.
// Account identity outlives tasks; retained objects keep their original charge.
use cathedral_contracts::user as abi;

pub struct Funding {
    limits: [usize; abi::MAX_TASKS],
    used: [usize; abi::MAX_TASKS],
}
impl Funding {
    pub fn new(limits: [usize; abi::MAX_TASKS]) -> Self {
        Self {
            limits,
            used: [0; abi::MAX_TASKS],
        }
    }
    pub fn capacity(&self) -> usize {
        self.limits.iter().sum()
    }
    pub fn charged(&self) -> usize {
        self.used.iter().sum()
    }
    pub fn snapshot(&self, account: usize) -> [usize; 2] {
        [self.limits[account], self.used[account]]
    }
    pub fn available(&self, account: usize) -> usize {
        self.limits[account] - self.used[account]
    }
    pub fn charge(&mut self, account: usize, pages: usize) -> Result<(), u64> {
        if pages > self.available(account) {
            return Err(abi::NO_MEMORY);
        }
        self.used[account] += pages;
        Ok(())
    }
    pub fn refund(&mut self, account: usize, pages: usize) {
        self.used[account] = self.used[account]
            .checked_sub(pages)
            .expect("unowned frame refund");
    }
    pub fn check_assign(&self, owner: usize, child: usize, requested: usize) -> Result<usize, u64> {
        if owner >= abi::MAX_TASKS || child >= abi::MAX_TASKS || owner == child {
            return Err(abi::INVALID_ARGUMENT);
        }
        let remaining = self.limits[owner]
            .checked_add(self.limits[child])
            .and_then(|sum| sum.checked_sub(requested))
            .ok_or(abi::NO_MEMORY)?;
        if remaining < self.used[owner] || requested < self.used[child] {
            return Err(abi::BUSY);
        }
        Ok(remaining)
    }
    pub fn assign(&mut self, owner: usize, child: usize, requested: usize) -> Result<(), u64> {
        let remaining = self.check_assign(owner, child, requested)?;
        self.limits[owner] = remaining;
        self.limits[child] = requested;
        Ok(())
    }
}
