//! Immutable fixed chunks and flat manifests. Explicit retained versions are the
//! only liveness roots in this host model; hashes convey no application authority.
use std::collections::{HashMap, HashSet};

type Digest = [u8; 32];

#[derive(Clone, Copy)]
pub struct Policy {
    pub chunk_bytes: usize,
    pub inline_limit: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    id: Digest,
    len: usize,
    contents: Contents,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Contents {
    Inline(Vec<u8>),
    Chunks(Vec<Digest>),
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Work {
    pub hashed_payload: usize,
    pub hashed_manifest: usize,
    pub new_payload: usize,
    pub new_manifest: usize,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Live {
    pub payload: usize,
    pub manifests: usize,
    pub chunks: usize,
    pub versions: usize,
}

pub struct Store {
    policy: Policy,
    chunks: HashMap<Digest, Vec<u8>>,
    manifests: HashMap<Digest, Vec<u8>>,
    pub work: Work,
}

impl Store {
    pub fn new(policy: Policy) -> Self {
        assert!(policy.chunk_bytes > 0 && u32::try_from(policy.chunk_bytes).is_ok());
        Self {
            policy,
            chunks: HashMap::new(),
            manifests: HashMap::new(),
            work: Work::default(),
        }
    }

    pub fn put(&mut self, bytes: &[u8]) -> Version {
        let contents = if bytes.len() <= self.policy.inline_limit {
            Contents::Inline(bytes.to_vec())
        } else {
            Contents::Chunks(
                bytes
                    .chunks(self.policy.chunk_bytes)
                    .map(|part| self.intern_chunk(part))
                    .collect(),
            )
        };
        self.publish(bytes.len(), contents)
    }

    /// Same-length range replacement reuses untouched chunk references. Insertions
    /// are deliberately a full put in the workload, exposing fixed-boundary costs.
    pub fn patch(
        &mut self,
        old: &Version,
        offset: usize,
        bytes: &[u8],
    ) -> Result<Version, &'static str> {
        let end = offset.checked_add(bytes.len()).ok_or("range overflow")?;
        if end > old.len {
            return Err("range outside object");
        }
        if bytes.is_empty() {
            return Ok(old.clone());
        }
        let contents = match &old.contents {
            Contents::Inline(previous) => {
                let mut next = previous.clone();
                next[offset..end].copy_from_slice(bytes);
                Contents::Inline(next)
            }
            Contents::Chunks(previous) => {
                let mut next = previous.clone();
                let first = offset / self.policy.chunk_bytes;
                let last = (end - 1) / self.policy.chunk_bytes;
                for index in first..=last {
                    let mut part = self.chunks[&previous[index]].clone();
                    let base = index * self.policy.chunk_bytes;
                    let from = offset.max(base);
                    let to = end.min(base + part.len());
                    part[from - base..to - base]
                        .copy_from_slice(&bytes[from - offset..to - offset]);
                    next[index] = self.intern_chunk(&part);
                }
                Contents::Chunks(next)
            }
        };
        Ok(self.publish(old.len, contents))
    }

    pub fn read(&self, version: &Version) -> Vec<u8> {
        assert!(
            self.manifests.contains_key(&version.id),
            "version was not retained"
        );
        match &version.contents {
            Contents::Inline(bytes) => bytes.clone(),
            Contents::Chunks(parts) => {
                let mut bytes = Vec::with_capacity(version.len);
                for part in parts {
                    bytes.extend_from_slice(&self.chunks[part]);
                }
                assert_eq!(bytes.len(), version.len);
                bytes
            }
        }
    }

    /// Exact reachability for this finite, single-threaded experiment. This is not
    /// a proposed production GC, reference-count protocol or snapshot API.
    pub fn retain(&mut self, roots: &[Version]) -> Live {
        let mut chunks = HashSet::new();
        let mut manifests = HashSet::new();
        for root in roots {
            assert!(self.manifests.contains_key(&root.id));
            manifests.insert(root.id);
            if let Contents::Chunks(parts) = &root.contents {
                chunks.extend(parts.iter().copied());
            }
        }
        self.chunks.retain(|key, _| chunks.contains(key));
        self.manifests.retain(|key, _| manifests.contains(key));
        Live {
            payload: self.chunks.values().map(Vec::len).sum(),
            manifests: self.manifests.values().map(Vec::len).sum(),
            chunks: self.chunks.len(),
            versions: self.manifests.len(),
        }
    }

    fn intern_chunk(&mut self, bytes: &[u8]) -> Digest {
        let key = digest(b"cathedral.layout-lab.chunk.v1\0", bytes);
        self.work.hashed_payload += bytes.len();
        if let Some(existing) = self.chunks.get(&key) {
            assert_eq!(existing, bytes, "hash collision");
        } else {
            self.work.new_payload += bytes.len();
            self.chunks.insert(key, bytes.to_vec());
        }
        key
    }

    fn publish(&mut self, len: usize, contents: Contents) -> Version {
        // Experimental exact encoding: mode:u8, length:u64, chunk size:u32,
        // count:u32, then either inline bytes or count 32-byte chunk digests.
        // No padding, compression, object IDs, index entries or device headers.
        let mut encoded = vec![match contents {
            Contents::Inline(_) => 0,
            Contents::Chunks(_) => 1,
        }];
        encoded.extend_from_slice(&(len as u64).to_le_bytes());
        encoded.extend_from_slice(&(self.policy.chunk_bytes as u32).to_le_bytes());
        match &contents {
            Contents::Inline(bytes) => {
                encoded.extend_from_slice(&0u32.to_le_bytes());
                encoded.extend_from_slice(bytes);
            }
            Contents::Chunks(parts) => {
                encoded.extend_from_slice(
                    &u32::try_from(parts.len())
                        .expect("bounded workload")
                        .to_le_bytes(),
                );
                for part in parts {
                    encoded.extend_from_slice(part);
                }
            }
        }
        let id = digest(b"cathedral.layout-lab.manifest.v1\0", &encoded);
        self.work.hashed_manifest += encoded.len();
        if let Some(existing) = self.manifests.get(&id) {
            assert_eq!(existing, &encoded, "hash collision");
        } else {
            self.work.new_manifest += encoded.len();
            self.manifests.insert(id, encoded);
        }
        Version { id, len, contents }
    }
}

fn digest(domain: &[u8], bytes: &[u8]) -> Digest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}
