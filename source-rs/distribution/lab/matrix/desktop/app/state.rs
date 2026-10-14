//! App schema over its matrix-local catalog capability. Persist before presenting;
//! revoked bindings cannot save, and a replacement reads only committed state.
use super::*;
use cathedral_contracts::storage as store;
pub(super) struct State {
    binding: u64,
    generation: u64,
    style: u64,
    pub alternate: bool,
}
impl State {
    pub fn open(parent: Pair, style: u64) -> Self {
        let info = call(parent, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]);
        let mut state = Self {
            binding: 0,
            generation: 0,
            style,
            alternate: false,
        };
        if info[0] == abi::DENIED {
            return state;
        } // Explicit diskless pressure fixture.
        assert_eq!(info[0], 0);
        state.binding = info[1];
        let record = match state.request(parent, store::READ, 2, 0, &[]) {
            Ok(record) => record,
            Err(store::NOT_FOUND) => {
                let generation = state
                    .request(parent, store::LIST, 0, 0, &[])
                    .unwrap()
                    .generation;
                state
                    .request(parent, store::CREATE, 2, generation, &state.payload(false))
                    .unwrap()
            }
            Err(error) => panic!("matrix app storage {error}"),
        };
        // CREATE replies with the durable generation; read validates the app schema.
        state.generation = record.generation;
        let record = state.request(parent, store::READ, 2, 0, &[]).unwrap();
        assert!(
            record.payload() == state.payload(false) || record.payload() == state.payload(true)
        );
        state.alternate = record.bytes[5] != 0;
        state
    }
    pub fn toggle(&mut self, parent: Pair) -> Result<(), u64> {
        let alternate = !self.alternate;
        if self.binding != 0 {
            let record = self.request(
                parent,
                store::REPLACE,
                2,
                self.generation,
                &self.payload(alternate),
            )?;
            self.generation = record.generation;
        }
        self.alternate = alternate;
        Ok(())
    }
    pub fn stage(&self, parent: Pair) {
        assert_ne!(self.binding, 0);
        self.request(parent, store::BEGIN, 0, self.generation, &[])
            .unwrap();
        self.request(
            parent,
            store::STAGE_REPLACE,
            2,
            0,
            &self.payload(!self.alternate),
        )
        .unwrap();
    }
    pub fn color(&self) -> u64 {
        if self.alternate {
            0xd18b35
        } else if self.style == 0 {
            LEFT
        } else {
            RIGHT
        }
    }
    fn payload(&self, alternate: bool) -> [u8; 6] {
        [
            b'M',
            b'C',
            b'0',
            b'1',
            self.style as u8,
            u8::from(alternate),
        ]
    }
    fn request(
        &self,
        parent: Pair,
        op: u64,
        object: u64,
        generation: u64,
        payload: &[u8],
    ) -> Result<store::Record, u64> {
        let opcode = (self.binding << wire::STORAGE_SCOPE_BITS) | (wire::STORAGE_BASE + op);
        let bytes = store::request(opcode, object, generation, payload).unwrap();
        store::Record::decode(&wire::encode(call(parent, wire::decode(&bytes).unwrap())))
    }
}
