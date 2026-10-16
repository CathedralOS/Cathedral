// Immutable positional chunk-reference tree. Level zero points to payload chunks;
// higher levels point to nodes. Only affected paths are rebuilt during replacement.
use crate::chunks::{Digest, Reads, Work, digest};
use std::collections::{BTreeMap, HashMap, HashSet};

struct Node {
    level: u8,
    children: Vec<Digest>,
    encoded: Vec<u8>,
}

pub struct Tree {
    fanout: usize,
    nodes: HashMap<Digest, Node>,
}

impl Tree {
    #[cfg(feature = "engines")]
    pub fn records(&self) -> Vec<(Digest, Vec<u8>)> {
        self.nodes
            .iter()
            .map(|(key, node)| (*key, node.encoded.clone()))
            .collect()
    }

    pub fn new(fanout: usize) -> Self {
        assert!(fanout >= 2 && u32::try_from(fanout).is_ok());
        Self {
            fanout,
            nodes: HashMap::new(),
        }
    }

    pub fn build(&mut self, chunks: &[Digest], work: &mut Work) -> Digest {
        assert!(!chunks.is_empty());
        let mut children = chunks.to_vec();
        let mut level = 0;
        loop {
            let parents: Vec<_> = children
                .chunks(self.fanout)
                .map(|group| self.intern(level, group.to_vec(), work))
                .collect();
            if parents.len() == 1 {
                return parents[0];
            }
            children = parents;
            level += 1;
        }
    }

    /// Update all changed references in one traversal. A shared ancestor is
    /// encoded/hashed once, even if several edits reach different children.
    pub fn replace(
        &mut self,
        root: Digest,
        changes: &BTreeMap<usize, Digest>,
        work: &mut Work,
    ) -> Digest {
        self.replace_at(root, 0, changes, work)
    }

    fn replace_at(
        &mut self,
        key: Digest,
        base: usize,
        changes: &BTreeMap<usize, Digest>,
        work: &mut Work,
    ) -> Digest {
        let node = &self.nodes[&key];
        let level = node.level;
        let span = self.fanout.checked_pow(level.into()).expect("bounded tree");
        let mut children = node.children.clone();
        for (index, child) in children.iter_mut().enumerate() {
            let start = base + index * span;
            if changes.range(start..start + span).next().is_none() {
                continue;
            }
            *child = if level == 0 {
                changes[&start]
            } else {
                self.replace_at(*child, start, changes, work)
            };
        }
        if children == self.nodes[&key].children {
            key
        } else {
            self.intern(level, children, work)
        }
    }

    /// Return only requested chunk references, accounting for unique whole
    /// encoded nodes needed to authenticate this read with an initially cold cache.
    pub fn range(&self, root: Digest, first: usize, end: usize, reads: &mut Reads) -> Vec<Digest> {
        let mut result = Vec::with_capacity(end - first);
        self.range_at(root, 0, first, end, reads, &mut result);
        assert_eq!(result.len(), end - first);
        result
    }

    fn range_at(
        &self,
        key: Digest,
        base: usize,
        first: usize,
        end: usize,
        reads: &mut Reads,
        result: &mut Vec<Digest>,
    ) {
        let node = &self.nodes[&key];
        reads.metadata(key, node.encoded.len());
        let span = self
            .fanout
            .checked_pow(node.level.into())
            .expect("bounded tree");
        for (index, child) in node.children.iter().enumerate() {
            let start = base + index * span;
            if start >= end || start + span <= first {
                continue;
            }
            if node.level == 0 {
                result.push(*child);
            } else {
                self.range_at(*child, start, first, end, reads, result);
            }
        }
    }

    /// Reachability follows all retained tree roots, deduplicating shared nodes
    /// and payload references. No production collection/lifetime policy is implied.
    pub fn retain(&mut self, roots: &[Digest], chunks: &mut HashSet<Digest>) -> (usize, usize) {
        let mut seen = HashSet::new();
        let mut pending = roots.to_vec();
        while let Some(key) = pending.pop() {
            if !seen.insert(key) {
                continue;
            }
            let node = &self.nodes[&key];
            if node.level == 0 {
                chunks.extend(node.children.iter().copied());
            } else {
                pending.extend(node.children.iter().copied());
            }
        }
        self.nodes.retain(|key, _| seen.contains(key));
        (
            self.nodes.values().map(|node| node.encoded.len()).sum(),
            self.nodes.len(),
        )
    }

    fn intern(&mut self, level: u8, children: Vec<Digest>, work: &mut Work) -> Digest {
        assert!(!children.is_empty() && children.len() <= self.fanout);
        let mut encoded = vec![level];
        encoded.extend_from_slice(&(children.len() as u32).to_le_bytes());
        for child in &children {
            encoded.extend_from_slice(child);
        }
        let key = digest(b"cathedral.layout-lab.chunk-tree.v1\0", &encoded);
        work.hashed_manifest += encoded.len();
        if let Some(existing) = self.nodes.get(&key) {
            assert_eq!(existing.encoded, encoded, "hash collision");
        } else {
            work.new_manifest += encoded.len();
            self.nodes.insert(
                key,
                Node {
                    level,
                    children,
                    encoded,
                },
            );
        }
        key
    }
}
