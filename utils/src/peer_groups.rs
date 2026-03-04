//! A group-based collection where items are organized into peer groups.

use std::collections::HashMap;
use std::hash::Hash;

// This is the best way I found to have good complexity on relevant
// operations... There might be easier or more streamlined approaches but I
// couldn't think of them
pub struct PeerGroups<T: Eq + Hash + Clone, M = ()> {
    /// Maps each item to its internal Id
    id_of: HashMap<T, usize>,
    /// All inserted items, indexed by Id
    items: Vec<T>,
    /// For each Id, which `GroupId` does it belong to
    group_of: Vec<usize>,
    /// For each `GroupId`, the Ids of its members
    members: Vec<Vec<usize>>,
    /// Per-group metadata, indexed by `GroupId`
    group_metadata: Vec<Option<M>>,
}

impl<T: Eq + Hash + Clone, M> Default for PeerGroups<T, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone, M> PeerGroups<T, M> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id_of: HashMap::new(),
            items: Vec::new(),
            group_of: Vec::new(),
            members: Vec::new(),
            group_metadata: Vec::new(),
        }
    }

    /// Place all `items` into the same group, merging any existing groups
    /// they already belong to
    pub fn link(&mut self, items: &[T]) {
        if items.is_empty() {
            return;
        }

        let ids: Vec<usize> = items.iter().map(|item| self.get_or_emplace(item)).collect();

        let mut groups: Vec<usize> = ids.iter().map(|&id| self.group_of[id]).collect();
        groups.sort_unstable();
        groups.dedup();

        let survivor = groups[0];

        for &gid in &groups[1..] {
            let moving: Vec<usize> = std::mem::take(&mut self.members[gid]);
            for &mid in &moving {
                self.group_of[mid] = survivor;
            }
            self.members[survivor].extend(moving);
            self.group_metadata[gid] = None;
        }
    }

    /// All peers of `item` (same group, excluding `item` itself)
    pub fn peers(&self, item: &T) -> Option<Vec<&T>> {
        let &id = self.id_of.get(item)?;
        let gid = self.group_of[id];
        Some(
            self.members[gid]
                .iter()
                .filter(|&&mid| mid != id)
                .map(|&mid| &self.items[mid])
                .collect(),
        )
    }

    /// Metadata for the group that `item` belongs to, if any
    pub fn metadata(&self, item: &T) -> Option<&M> {
        let &id = self.id_of.get(item)?;
        self.group_metadata[self.group_of[id]].as_ref()
    }

    /// Set metadata on the group that `item` belongs to.
    /// Returns `false` if the item is unknown
    pub fn set_metadata(&mut self, item: &T, data: M) -> bool {
        let Some(&id) = self.id_of.get(item) else {
            return false;
        };
        self.group_metadata[self.group_of[id]] = Some(data);
        true
    }

    /// Number of non-empty groups
    #[must_use]
    pub fn group_count(&self) -> usize {
        self.members.iter().filter(|m| !m.is_empty()).count()
    }

    /// Remove tombstones left by merges: reassign `GroupIds`, shrink `members`
    /// and `group_metadata`, update `group_of`
    pub fn compact(&mut self) {
        let mut remap: Vec<usize> = vec![0; self.members.len()];
        let mut compacted_members: Vec<Vec<usize>> = Vec::new();
        let mut compacted_metadata: Vec<Option<M>> = Vec::new();

        for (old_gid, group) in self.members.iter().enumerate() {
            if !group.is_empty() {
                remap[old_gid] = compacted_members.len();
                compacted_members.push(group.clone());
            }
        }

        for (old_gid, meta) in self.group_metadata.drain(..).enumerate() {
            if !self.members[old_gid].is_empty() {
                compacted_metadata.push(meta);
            }
        }

        for gid in &mut self.group_of {
            *gid = remap[*gid];
        }

        self.members = compacted_members;
        self.group_metadata = compacted_metadata;
    }

    /// Get or create an internal Id for `item`. New items start in a singleton
    /// group
    fn get_or_emplace(&mut self, item: &T) -> usize {
        if let Some(&id) = self.id_of.get(item) {
            return id;
        }
        let id = self.items.len();
        self.items.push(item.clone());
        self.id_of.insert(item.clone(), id);
        let gid = self.members.len();
        self.group_of.push(gid);
        self.members.push(vec![id]);
        self.group_metadata.push(None);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_group() {
        let mut pg: PeerGroups<&str> = PeerGroups::new();
        pg.link(&["a", "b", "c"]);
        assert_eq!(pg.group_count(), 1);

        let mut peers: Vec<&&str> = pg.peers(&"a").unwrap();
        peers.sort();
        assert_eq!(peers, vec![&"b", &"c"]);
    }

    #[test]
    fn separate_groups() {
        let mut pg: PeerGroups<&str> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.link(&["x", "y"]);
        assert_eq!(pg.group_count(), 2);
        assert_eq!(pg.peers(&"a").unwrap(), vec![&"b"]);
        assert_eq!(pg.peers(&"x").unwrap(), vec![&"y"]);
    }

    #[test]
    fn merge_via_shared_item() {
        let mut pg: PeerGroups<&str> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.link(&["b", "c"]);
        assert_eq!(pg.group_count(), 1);

        let mut peers: Vec<&&str> = pg.peers(&"a").unwrap();
        peers.sort();
        assert_eq!(peers, vec![&"b", &"c"]);
    }

    #[test]
    fn merge_disjoint_groups() {
        let mut pg: PeerGroups<&str> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.link(&["c", "d"]);
        pg.link(&["b", "c"]);
        assert_eq!(pg.group_count(), 1);

        let mut peers: Vec<&&str> = pg.peers(&"a").unwrap();
        peers.sort();
        assert_eq!(peers, vec![&"b", &"c", &"d"]);
    }

    #[test]
    fn unknown_item_returns_none() {
        let pg: PeerGroups<&str> = PeerGroups::new();
        assert!(pg.peers(&"z").is_none());
        assert!(pg.metadata(&"z").is_none());
    }

    #[test]
    fn no_metadata_by_default() {
        let mut pg: PeerGroups<&str> = PeerGroups::new();
        pg.link(&["a", "b"]);
        assert!(pg.metadata(&"a").is_none());
    }

    #[test]
    fn compact_removes_tombstones() {
        let mut pg: PeerGroups<&str> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.link(&["c", "d"]);
        pg.link(&["b", "c"]);
        assert_eq!(pg.group_count(), 1);
        assert!(pg.members.len() > 1);

        pg.compact();
        assert_eq!(pg.members.len(), 1);
        assert_eq!(pg.group_count(), 1);

        let mut peers: Vec<&&str> = pg.peers(&"a").unwrap();
        peers.sort();
        assert_eq!(peers, vec![&"b", &"c", &"d"]);
    }

    #[test]
    fn compact_preserves_separate_groups() {
        let mut pg: PeerGroups<&str> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.link(&["x", "y"]);
        pg.compact();
        assert_eq!(pg.group_count(), 2);
        assert_eq!(pg.peers(&"a").unwrap(), vec![&"b"]);
        assert_eq!(pg.peers(&"x").unwrap(), vec![&"y"]);
    }

    #[test]
    fn metadata_set_and_get() {
        let mut pg: PeerGroups<&str, String> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.set_metadata(&"a", "hello".into());

        assert_eq!(pg.metadata(&"a"), Some(&"hello".into()));
        assert_eq!(pg.metadata(&"b"), Some(&"hello".into()));
    }

    #[test]
    fn metadata_survives_merge() {
        let mut pg: PeerGroups<&str, String> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.set_metadata(&"a", "from_config".into());
        pg.link(&["c", "d"]);

        pg.link(&["b", "c"]);

        assert_eq!(pg.metadata(&"a"), Some(&"from_config".into()));
        assert_eq!(pg.metadata(&"d"), Some(&"from_config".into()));
    }

    #[test]
    fn metadata_survives_compact() {
        let mut pg: PeerGroups<&str, i32> = PeerGroups::new();
        pg.link(&["a", "b"]);
        pg.set_metadata(&"a", 42);
        pg.link(&["x", "y"]);
        pg.set_metadata(&"x", 99);

        pg.compact();

        assert_eq!(pg.metadata(&"a"), Some(&42));
        assert_eq!(pg.metadata(&"x"), Some(&99));
    }

    #[test]
    fn set_metadata_unknown_item() {
        let mut pg: PeerGroups<&str, String> = PeerGroups::new();
        assert!(!pg.set_metadata(&"z", "nope".into()));
    }
}
