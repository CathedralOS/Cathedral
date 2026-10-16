//! Immutable fixed chunks with flat or tree manifests. Explicit retained versions are the
//! only liveness roots in this host model; hashes convey no application authority.
use crate::chunk_tree::Tree;
use std::collections::{BTreeMap, HashMap, HashSet};

pub(crate) type Digest = [u8; 32];

#[derive(Clone, Copy, Debug)]
pub enum Layout {
    Flat,
    Tree { fanout: usize },
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flat => write!(f, "flat"),
            Self::Tree { fanout } => write!(f, "tree_{fanout}"),
        }
    }
}

#[derive(Default)]
pub struct Reads {
    pub metadata_bytes: usize,
    pub metadata_records: usize,
    pub payload_bytes: usize,
    pub payload_records: usize,
    seen_metadata: HashSet<Digest>,
    seen_payload: HashSet<Digest>,
}

impl Reads {
    pub(crate) fn metadata(&mut self, id: Digest, bytes: usize) {
        if self.seen_metadata.insert(id) {
            self.metadata_bytes += bytes;
            self.metadata_records += 1;
        }
    }
    fn payload(&mut self, id: Digest, bytes: usize) {
        if self.seen_payload.insert(id) {
            self.payload_bytes += bytes;
            self.payload_records += 1;
        }
    }
}

pub struct Edit<'a> {
    pub offset: usize,
    pub bytes: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct Policy {
    pub chunk_bytes: usize,
    pub inline_limit: usize,
    pub layout: Layout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    id: Digest,
    len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Contents {
    Inline(Vec<u8>),
    Chunks(Vec<Digest>),
    Tree(Digest),
}

struct Manifest {
    contents: Contents,
    encoded: Vec<u8>,
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
    pub tree_nodes: usize,
}

pub struct Store {
    policy: Policy,
    chunks: HashMap<Digest, Vec<u8>>,
    manifests: HashMap<Digest, Manifest>,
    tree: Tree,
    pub work: Work,
}

impl Store {
    /// Export the exact experimental records for the host engine comparison.
    /// Distinct key prefixes separate payload, version and tree-node records.
    #[cfg(feature = "engines")]
    pub fn records(&self, version: &Version) -> BTreeMap<Vec<u8>, Vec<u8>> {
        let mut records = BTreeMap::new();
        for (kind, entries) in [
            (
                b'c',
                self.chunks
                    .iter()
                    .map(|(id, bytes)| (*id, bytes.clone()))
                    .collect::<Vec<_>>(),
            ),
            (
                b'm',
                self.manifests
                    .iter()
                    .map(|(id, m)| (*id, m.encoded.clone()))
                    .collect(),
            ),
            (b't', self.tree.records()),
        ] {
            for (id, bytes) in entries {
                let mut key = vec![kind];
                key.extend_from_slice(&id);
                records.insert(key, bytes);
            }
        }
        records.insert(b"head".to_vec(), version.id.to_vec());
        records
    }

    pub fn new(policy: Policy) -> Self {
        assert!(policy.chunk_bytes > 0 && u32::try_from(policy.chunk_bytes).is_ok());
        Self {
            tree: Tree::new(match policy.layout {
                Layout::Flat => 2,
                Layout::Tree { fanout } => fanout,
            }),
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
            let parts: Vec<_> = bytes
                .chunks(self.policy.chunk_bytes)
                .map(|part| self.intern_chunk(part))
                .collect();
            match self.policy.layout {
                Layout::Flat => Contents::Chunks(parts),
                Layout::Tree { .. } => Contents::Tree(self.tree.build(&parts, &mut self.work)),
            }
        };
        self.publish(bytes.len(), contents)
    }

    /// Same-length replacements are staged in input order, with one new payload
    /// per touched chunk and one rebuild per affected metadata node on publication.
    pub fn patch(
        &mut self,
        old: &Version,
        offset: usize,
        bytes: &[u8],
    ) -> Result<Version, &'static str> {
        self.patch_batch(old, &[Edit { offset, bytes }])
    }

    pub fn patch_batch(
        &mut self,
        old: &Version,
        edits: &[Edit<'_>],
    ) -> Result<Version, &'static str> {
        // Validate the entire batch before allocating, hashing or publishing.
        for edit in edits {
            let end = edit
                .offset
                .checked_add(edit.bytes.len())
                .ok_or("range overflow")?;
            if end > old.len {
                return Err("range outside object");
            }
        }
        if edits.iter().all(|edit| edit.bytes.is_empty()) {
            return Ok(old.clone());
        }
        let previous = self.manifests[&old.id].contents.clone();
        let next = match previous {
            Contents::Inline(mut bytes) => {
                for edit in edits {
                    bytes[edit.offset..edit.offset + edit.bytes.len()].copy_from_slice(edit.bytes);
                }
                Contents::Inline(bytes)
            }
            contents => {
                let mut staged = BTreeMap::new();
                for edit in edits {
                    if edit.bytes.is_empty() {
                        continue;
                    }
                    let end = edit.offset + edit.bytes.len();
                    for index in
                        edit.offset / self.policy.chunk_bytes..=(end - 1) / self.policy.chunk_bytes
                    {
                        let part = staged.entry(index).or_insert_with(|| {
                            let key = match &contents {
                                Contents::Chunks(parts) => parts[index],
                                Contents::Tree(root) => {
                                    self.tree
                                        .range(*root, index, index + 1, &mut Reads::default())[0]
                                }
                                Contents::Inline(_) => unreachable!(),
                            };
                            self.chunks[&key].clone()
                        });
                        let base = index * self.policy.chunk_bytes;
                        let from = edit.offset.max(base);
                        let to = end.min(base + part.len());
                        part[from - base..to - base]
                            .copy_from_slice(&edit.bytes[from - edit.offset..to - edit.offset]);
                    }
                }
                let changes: BTreeMap<_, _> = staged
                    .into_iter()
                    .map(|(index, bytes)| (index, self.intern_chunk(&bytes)))
                    .collect();
                match contents {
                    Contents::Chunks(mut parts) => {
                        for (index, key) in changes {
                            parts[index] = key;
                        }
                        Contents::Chunks(parts)
                    }
                    Contents::Tree(root) => {
                        Contents::Tree(self.tree.replace(root, &changes, &mut self.work))
                    }
                    Contents::Inline(_) => unreachable!(),
                }
            }
        };
        Ok(self.publish(old.len, next))
    }

    pub fn read(&self, version: &Version) -> Vec<u8> {
        self.read_range(version, 0, version.len).unwrap().0
    }

    /// Count unique packed records required by a cold authenticated range read.
    /// This accesses resident host maps, not a device or a persistent cache.
    pub fn read_range(
        &self,
        version: &Version,
        offset: usize,
        length: usize,
    ) -> Result<(Vec<u8>, Reads), &'static str> {
        let end = offset.checked_add(length).ok_or("range overflow")?;
        if end > version.len {
            return Err("range outside object");
        }
        let mut reads = Reads::default();
        if length == 0 {
            return Ok((Vec::new(), reads));
        }
        let manifest = &self.manifests[&version.id];
        reads.metadata(version.id, manifest.encoded.len());
        let bytes = match &manifest.contents {
            Contents::Inline(bytes) => bytes[offset..end].to_vec(),
            contents => {
                let first = offset / self.policy.chunk_bytes;
                let last = end.div_ceil(self.policy.chunk_bytes);
                let parts = match contents {
                    Contents::Chunks(parts) => parts[first..last].to_vec(),
                    Contents::Tree(root) => self.tree.range(*root, first, last, &mut reads),
                    Contents::Inline(_) => unreachable!(),
                };
                let mut bytes = Vec::with_capacity(length);
                for (index, key) in parts.iter().enumerate() {
                    let part = &self.chunks[key];
                    reads.payload(*key, part.len());
                    let base = (first + index) * self.policy.chunk_bytes;
                    bytes.extend_from_slice(
                        &part[offset.max(base) - base..end.min(base + part.len()) - base],
                    );
                }
                bytes
            }
        };
        assert_eq!(bytes.len(), length);
        Ok((bytes, reads))
    }

    /// Exact reachability for this finite, single-threaded experiment. This is not
    /// a proposed production GC, reference-count protocol or snapshot API.
    pub fn retain(&mut self, roots: &[Version]) -> Live {
        let mut chunks = HashSet::new();
        let mut manifests = HashSet::new();
        let mut tree_roots = Vec::new();
        for root in roots {
            assert!(self.manifests.contains_key(&root.id));
            manifests.insert(root.id);
            match &self.manifests[&root.id].contents {
                Contents::Chunks(parts) => chunks.extend(parts.iter().copied()),
                Contents::Tree(key) => tree_roots.push(*key),
                Contents::Inline(_) => {}
            }
        }
        let (tree_bytes, tree_nodes) = self.tree.retain(&tree_roots, &mut chunks);
        self.chunks.retain(|key, _| chunks.contains(key));
        self.manifests.retain(|key, _| manifests.contains(key));
        Live {
            payload: self.chunks.values().map(Vec::len).sum(),
            manifests: self
                .manifests
                .values()
                .map(|manifest| manifest.encoded.len())
                .sum::<usize>()
                + tree_bytes,
            chunks: self.chunks.len(),
            versions: self.manifests.len(),
            tree_nodes,
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
        // count:u32, then inline bytes, count 32-byte chunk digests, or a
        // tree descriptor (fanout:u32, root digest:32). Tree nodes own their level.
        // No padding, compression, object IDs, index entries or device headers.
        let mut encoded = vec![match contents {
            Contents::Inline(_) => 0,
            Contents::Chunks(_) => 1,
            Contents::Tree(_) => 2,
        }];
        encoded.extend_from_slice(&(len as u64).to_le_bytes());
        encoded.extend_from_slice(&(self.policy.chunk_bytes as u32).to_le_bytes());
        match &contents {
            Contents::Tree(root) => {
                encoded.extend_from_slice(
                    &u32::try_from(len.div_ceil(self.policy.chunk_bytes))
                        .expect("bounded workload")
                        .to_le_bytes(),
                );
                let Layout::Tree { fanout } = self.policy.layout else {
                    unreachable!()
                };
                encoded.extend_from_slice(&(fanout as u32).to_le_bytes());
                encoded.extend_from_slice(root);
            }
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
            assert_eq!(existing.encoded, encoded, "hash collision");
        } else {
            self.work.new_manifest += encoded.len();
            self.manifests.insert(id, Manifest { contents, encoded });
        }
        Version { id, len }
    }
}

pub(crate) fn digest(domain: &[u8], bytes: &[u8]) -> Digest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}
