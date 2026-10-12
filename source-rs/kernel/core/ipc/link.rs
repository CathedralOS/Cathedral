//! Boot-approved peer graph. Ports grant connection acceptance, never task control.
use crate::ipc::MAX_TASKS;
use cathedral_contracts::user as abi;

#[derive(Clone, Copy, Debug)]
pub struct LinkSpec {
    pub client: usize,
    pub service: usize,
}
pub struct Link {
    pub spec: LinkSpec,
    names: [u64; 2],
    alive: [bool; 2],
    accepted: [bool; 2],
}
impl Link {
    pub fn new(spec: LinkSpec, tasks: usize, initial: usize) -> Result<Self, u64> {
        if tasks > MAX_TASKS
            || initial > tasks
            || spec.client >= tasks
            || spec.service >= tasks
            || spec.client == spec.service
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        Ok(Self {
            spec,
            names: [0; 2],
            alive: [spec.client < initial, spec.service < initial],
            accepted: [false; 2],
        })
    }
    fn side(&self, caller: usize) -> Result<usize, u64> {
        if caller == self.spec.client {
            Ok(0)
        } else if caller == self.spec.service {
            Ok(1)
        } else {
            Err(abi::DENIED)
        }
    }
    pub fn port(&self, caller: usize) -> Result<u64, u64> {
        let side = self.side(caller)?;
        if !self.alive[side] {
            return Err(abi::DENIED);
        }
        if self.names[side] == 0 {
            return Err(abi::DENIED);
        }
        Ok(self.names[side])
    }
    pub fn admit(&mut self, slot: usize, name: u64) -> bool {
        let Ok(side) = self.side(slot) else {
            return false;
        };
        self.names[side] = name;
        self.alive[side] = true;
        self.accepted = [false; 2];
        self.alive == [true; 2]
    }
    pub fn close(&mut self, slot: usize) {
        if let Ok(side) = self.side(slot) {
            self.alive[side] = false;
            self.accepted = [false; 2];
        }
    }
    pub fn accept(&mut self, caller: usize, port: u64) -> Result<(), u64> {
        if self.port(caller) != Ok(port) {
            return Err(abi::BAD_HANDLE);
        }
        if self.alive != [true; 2] {
            return Err(abi::PEER_CLOSED);
        }
        let side = self.side(caller)?;
        if self.accepted[side] {
            return Err(abi::BUSY);
        }
        self.accepted[side] = true;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reserved_peers_accept_each_incarnation_without_acquiring_control() {
        let mut link = Link::new(
            LinkSpec {
                client: 3,
                service: 1,
            },
            4,
            1,
        )
        .unwrap();
        assert_eq!(link.port(0), Err(abi::DENIED));
        assert!(!link.admit(1, 1));
        let service = link.port(1).unwrap();
        assert_eq!(link.accept(1, service), Err(abi::PEER_CLOSED));
        assert!(link.admit(3, 3));
        let client = link.port(3).unwrap();
        assert_eq!(link.accept(3, service), Err(abi::BAD_HANDLE));
        link.accept(3, client).unwrap();
        assert_eq!(link.accept(3, client), Err(abi::BUSY));
        link.accept(1, service).unwrap();
        link.close(3);
        assert_eq!(link.accept(1, service), Err(abi::PEER_CLOSED));
        assert!(link.admit(3, 4));
        link.accept(1, service).unwrap();
        assert_eq!(link.accept(3, client), Err(abi::BAD_HANDLE));
        link.accept(3, link.port(3).unwrap()).unwrap();
        assert!(
            Link::new(
                LinkSpec {
                    client: 4,
                    service: 1
                },
                4,
                1
            )
            .is_err()
        );
    }
}
