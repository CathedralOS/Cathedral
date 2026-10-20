// Subtree delegation: the owner selects a source and a scope object, the
// child sees only a local binding. A whole source belongs to one child at a
// time; a subtree may be held beside anything, since the provider confines
// it and the owner holds all of it anyway. The scope is what the matrix
// stamps on the child's requests; zero passes on everything the matrix holds.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{matrix as wire, storage as protocol};
/// A local binding reply, or a stamped request still needing upstream transport.
pub enum Prepared {
    Reply(wire::Packet),
    Forward {
        source: usize,
        words: wire::Packet,
    },
    /// Send the read-only check upstream first. Only its successful reply
    /// permits forwarding `words`, after rechecking the caller's revision.
    Validate {
        source: usize,
        check: wire::Packet,
        words: wire::Packet,
        proof: Proof,
    },
}
/// Opaque bookkeeping for one upstream containment check. Only confirm it
/// after that exact check succeeds on the binding's live upstream connection.
#[derive(Clone, Copy)]
pub struct Proof {
    slot: usize,
    incarnation: u64,
    revision: u64,
    scopes: [u64; 2],
}
const PROOFS: usize = 4;
#[derive(Clone, Copy)]
struct Binding {
    incarnation: u64,
    source: usize,
    scope: u64,
    proofs: [Option<[u64; 2]>; PROOFS],
    next_proof: usize,
}
pub struct Scopes<const N: usize = 2> {
    bindings: [Option<Binding>; N],
    revisions: [u64; N],
}
impl<const N: usize> Scopes<N> {
    pub const fn empty() -> Self {
        Self {
            bindings: [None; N],
            revisions: [0; N],
        }
    }
    pub fn grant(
        &mut self,
        slot: usize,
        incarnation: u64,
        source: usize,
        scope: u64,
        available: usize,
    ) -> Result<(), u64> {
        if slot >= N || incarnation == 0 || source >= available {
            return Err(abi::DENIED);
        }
        if self.bindings[slot].is_some()
            || self
                .bindings
                .iter()
                .flatten()
                .any(|b| b.source == source && b.scope == 0 && scope == 0)
        {
            return Err(abi::BUSY);
        }
        self.revisions[slot] = self.revisions[slot]
            .checked_add(1)
            .filter(|n| *n <= u64::MAX >> cathedral_platform_contracts::matrix::STORAGE_SCOPE_BITS)
            .ok_or(abi::NO_MEMORY)?;
        self.bindings[slot] = Some(Binding {
            incarnation,
            source,
            scope,
            proofs: [None; PROOFS],
            next_proof: 0,
        });
        Ok(())
    }
    /// The source and scope bound to this connection: source and scope.
    pub fn binding(&self, slot: usize, incarnation: u64) -> Result<(usize, u64), u64> {
        self.bindings
            .get(slot)
            .and_then(|b| *b)
            .filter(|b| b.incarnation == incarnation)
            .map(|b| (b.source, b.scope))
            .ok_or(abi::DENIED)
    }
    pub fn revision(&self, slot: usize, incarnation: u64) -> Result<u64, u64> {
        self.binding(slot, incarnation)?;
        Ok(self.revisions[slot])
    }
    pub fn authorize(
        &self,
        slot: usize,
        incarnation: u64,
        revision: u64,
    ) -> Result<(usize, u64), u64> {
        if self.revision(slot, incarnation)? != revision {
            return Err(abi::DENIED);
        }
        self.binding(slot, incarnation)
    }
    /// Authenticate the connection and revision before appending this owner's
    /// scope. Child-provided stamps only narrow authority; folding a full packet
    /// requires a confirmed containment proof under this exact binding.
    pub fn prepare(
        &self,
        slot: usize,
        incarnation: u64,
        mut words: wire::Packet,
        realm: u64,
    ) -> Result<Prepared, u64> {
        let (source, scope) = self.binding(slot, incarnation)?;
        if words[..7] == [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0] {
            return Ok(Prepared::Reply([
                0,
                self.revision(slot, incarnation)?,
                if scope == 0 { realm } else { scope },
                0,
                0,
                0,
                0,
                0,
            ]));
        }
        self.authorize(slot, incarnation, words[0] >> wire::STORAGE_SCOPE_BITS)?;
        if !(wire::STORAGE_BASE + protocol::HEAD..=wire::STORAGE_BASE + protocol::LAST)
            .contains(&(words[0] & 0xffff))
        {
            return Err(abi::DENIED);
        }
        if protocol::stamp(&mut words, scope).is_err() {
            // Object parents never move and IDs are never reused. Proving that
            // the innermost scope is inside both remaining restrictions makes
            // it their intersection for this request. Deletion can only make
            // the subsequent request fail; it cannot broaden authority.
            let proof = Proof {
                slot,
                incarnation,
                revision: self.revisions[slot],
                scopes: [words[5], words[6]],
            };
            if self.bindings[slot]
                .unwrap()
                .proofs
                .contains(&Some(proof.scopes))
            {
                words[6] = 0;
                return Ok(Prepared::Forward { source, words });
            }
            let mut check = protocol::Request::new(protocol::HEAD, words[5], 0, 0, 0).words();
            check[0] = (words[0] & !0xffff) | (wire::STORAGE_BASE + protocol::HEAD);
            check[5] = words[6];
            check[6] = scope;
            check[7] = words[7];
            words[6] = 0;
            return Ok(Prepared::Validate {
                source,
                check,
                words,
                proof,
            });
        }
        Ok(Prepared::Forward { source, words })
    }
    /// Memoize only successful checks. The request itself still reaches the
    /// provider with its innermost scope. Immutable parents/non-reused IDs make
    /// this proof reusable, not the operation's result or existence of an object.
    /// Revoke/regrant, upstream revision changes and provider retirement discard
    /// the binding and its bounded FIFO of proofs together.
    pub fn confirm(&mut self, proof: Proof) -> Result<(), u64> {
        self.authorize(proof.slot, proof.incarnation, proof.revision)?;
        let binding = self.bindings[proof.slot].as_mut().unwrap();
        if !binding.proofs.contains(&Some(proof.scopes)) {
            binding.proofs[binding.next_proof] = Some(proof.scopes);
            binding.next_proof = (binding.next_proof + 1) % PROOFS;
        }
        Ok(())
    }
    /// A replaced/retired provider invalidates every grant naming that source.
    /// Other providers' bindings are independent and remain usable.
    pub fn revoke_source(&mut self, source: usize) {
        for binding in &mut self.bindings {
            if binding.is_some_and(|b| b.source == source) {
                *binding = None;
            }
        }
    }
    pub fn revoke(&mut self, slot: usize) -> Option<usize> {
        self.bindings.get_mut(slot)?.take().map(|b| b.source)
    }
    pub fn clear(&mut self) {
        self.bindings = [None; N];
    }
}

impl<const N: usize> Default for Scopes<N> {
    fn default() -> Self {
        Self::empty()
    }
}
