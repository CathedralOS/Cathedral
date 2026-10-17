// The desktop owns worlds: top-level objects of the notes root, each the home
// of one workspace. The owner lists, creates and clones them on the root
// matrix's own authority, then grants the workspace one world as its scope,
// so an app inside sees a home and never the worlds beside it. Cloning a
// world is the store's subtree copy: every note in it is copied by reference,
// and a world of any size costs a few small records.
use super::*;
use cathedral_platform_contracts::storage as store;
pub(crate) const MAX_WORLDS: usize = 8;
/// The notes root is the first provider link the root matrix holds.
const SOURCE: u64 = 0;
pub(crate) struct Worlds {
    ids: [u64; MAX_WORLDS],
    count: usize,
    current: usize,
}
/// One object request on the root's own authority, answered in the
/// provider's words. Nothing is attached, so a publication only creates.
fn request(
    matrix: Pair,
    op: u64,
    object: u64,
    expected: u64,
    argument: u64,
) -> Result<wire::Packet, u64> {
    let reply = call(
        matrix,
        [
            wire::STORAGE_REQUEST,
            SOURCE,
            op,
            object,
            expected,
            argument,
            0,
            0,
        ],
    );
    if reply[0] != 0 {
        return Err(reply[0]);
    }
    Ok(reply)
}
impl Worlds {
    /// List the worlds; a fresh root gets its first one.
    pub fn open(matrix: Pair) -> Result<Self, u64> {
        let mut worlds = Self {
            ids: [0; MAX_WORLDS],
            count: 0,
            current: 0,
        };
        worlds.list(matrix)?;
        if worlds.count == 0 {
            let created = request(matrix, store::PUBLISH, 0, 0, 0)?;
            worlds.ids[0] = created[1];
            worlds.count = 1;
        }
        Ok(worlds)
    }
    fn list(&mut self, matrix: Pair) -> Result<(), u64> {
        self.count = 0;
        let mut after = 0;
        loop {
            let reply = request(matrix, store::LIST, 0, 0, after)?;
            let found = reply[1] as usize;
            for &id in &reply[2..2 + found.min(store::LIST_IDS)] {
                if self.count == MAX_WORLDS {
                    return Ok(());
                }
                self.ids[self.count] = id;
                self.count += 1;
                after = id;
            }
            if found < store::LIST_IDS {
                return Ok(());
            }
        }
    }
    pub fn current(&self) -> u64 {
        self.ids[self.current]
    }
    /// One-based position of the current world and how many there are.
    pub fn position(&self) -> (u64, u64) {
        (self.current as u64 + 1, self.count as u64)
    }
    /// Copy the current world and everything in it, then make the copy current.
    pub fn clone_current(&mut self, matrix: Pair) -> Result<(), u64> {
        if self.count == MAX_WORLDS {
            return Err(abi::NO_MEMORY);
        }
        let world = self.current();
        let head = request(matrix, store::HEAD, world, 0, 0)?;
        let copy = request(matrix, store::COPY_TREE, world, head[2], 0)?;
        self.list(matrix)?;
        self.current = self.ids[..self.count]
            .iter()
            .position(|id| *id == copy[1])
            .ok_or(abi::IO_ERROR)?;
        Ok(())
    }
    pub fn next(&mut self) {
        self.current = (self.current + 1) % self.count;
    }
}
