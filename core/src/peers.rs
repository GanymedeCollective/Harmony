//! Generic indexed collection of cross-platform peer entities.

use std::collections::HashMap;

use crate::PlatformId;

/// Trait for core types (`CoreChannel`, `CoreUser`) that aggregate
/// platform-specific aliases.
pub trait Peered: Clone {
    type Platform: Clone;

    fn aliases(&self) -> &HashMap<PlatformId, Self::Platform>;
    fn aliases_mut(&mut self) -> &mut HashMap<PlatformId, Self::Platform>;

    /// The platform this item belongs to.
    fn platform_of(item: &Self::Platform) -> &PlatformId;

    /// The platform-specific id (e.g. channel id, user id).
    fn id_of(item: &Self::Platform) -> &str;

    /// Construct a core entity from a single platform alias with default
    /// fields.
    fn from_single_alias(platform: PlatformId, item: Self::Platform) -> Self;

    /// Key used to auto-correlate entities across platforms (e.g. normalized
    /// channel name, lowercased display name). Return `None` to skip.
    fn match_key(item: &Self::Platform) -> Option<String>;
}

/// Indexed collection of peer entities with O(1) lookup by (platform, id)
/// and O(1) auto-correlation by match key.
pub struct Peers<T: Peered> {
    items: Vec<Option<T>>,
    index: HashMap<(PlatformId, String), usize>,
    match_index: HashMap<String, usize>,
}

impl<T: Peered> Default for Peers<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Peered> Peers<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }

    /// Build a collection from discovered platform items, auto-correlating
    /// across platforms by [`Peered::match_key`].
    #[must_use]
    pub fn build(discovered: &[(PlatformId, Vec<T::Platform>)]) -> Self {
        let mut peers = Self::new();
        peers.auto_correlate(discovered);
        peers
    }

    pub fn insert(&mut self, item: T) {
        let idx = self.items.len();
        for (platform, p) in item.aliases() {
            self.index
                .insert((platform.clone(), T::id_of(p).to_owned()), idx);
            if let Some(key) = T::match_key(p) {
                self.match_index.entry(key).or_insert(idx);
            }
        }
        self.items.push(Some(item));
    }

    #[must_use]
    pub fn find(&self, platform: &PlatformId, id: &str) -> Option<&T> {
        self.index
            .get(&(platform.clone(), id.to_owned()))
            .and_then(|&idx| self.items[idx].as_ref())
    }

    /// Insert, update, or auto-correlate a platform item.
    ///
    /// If a core entity already exists for this (platform, id), the alias is
    /// updated in place. Otherwise, if an entity on a *different* platform
    /// shares the same [`Peered::match_key`], the item is merged into that
    /// entity. As a last resort a new standalone entity is created.
    pub fn upsert(&mut self, item: T::Platform) {
        let platform = T::platform_of(&item).clone();
        let key = (platform.clone(), T::id_of(&item).to_owned());

        if let Some(&idx) = self.index.get(&key)
            && let Some(core) = &mut self.items[idx]
        {
            core.aliases_mut().insert(platform, item);
            return;
        }

        if let Some(match_key) = T::match_key(&item)
            && let Some(&existing_idx) = self.match_index.get(&match_key)
            && let Some(core) = &self.items[existing_idx]
            && !core.aliases().contains_key(&platform)
        {
            self.merge_into(existing_idx, item);
            return;
        }

        self.insert(T::from_single_alias(platform, item));
    }

    /// Detach a single platform alias from its core entity.
    ///
    /// The entity stays alive as long as it retains at least one alias.
    /// Only destroyed when the last alias is detached.
    pub fn detach(&mut self, platform: &PlatformId, id: &str) {
        let key = (platform.clone(), id.to_owned());
        let Some(&idx) = self.index.get(&key) else {
            return;
        };
        self.index.remove(&key);

        if let Some(core) = &mut self.items[idx] {
            if let Some(removed) = core.aliases_mut().remove(platform)
                && let Some(mk) = T::match_key(&removed)
                && self.match_index.get(&mk) == Some(&idx)
            {
                self.match_index.remove(&mk);
            }
            if core.aliases().is_empty() {
                self.items[idx] = None;
            }
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.iter().filter(|i| i.is_some()).count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn find_index(&self, platform: &PlatformId, id: &str) -> Option<usize> {
        self.index.get(&(platform.clone(), id.to_owned())).copied()
    }

    pub(crate) fn item_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.items[idx].as_mut()
    }

    pub(crate) fn reindex(
        &mut self,
        old_key: &(PlatformId, String),
        new_key: (PlatformId, String),
    ) {
        if let Some(&idx) = self.index.get(old_key) {
            self.index.remove(old_key);
            self.index.insert(new_key, idx);
        }
    }

    pub(crate) fn update_match_key(&mut self, old_key: Option<&str>, new_key: Option<String>, idx: usize) {
        if let Some(old) = old_key
            && self.match_index.get(old) == Some(&idx)
        {
            self.match_index.remove(old);
        }
        if let Some(new) = new_key {
            self.match_index.entry(new).or_insert(idx);
        }
    }

    /// Merge a platform item into an existing core entity at `idx`.
    fn merge_into(&mut self, idx: usize, item: T::Platform) {
        let platform = T::platform_of(&item).clone();
        self.index
            .insert((platform.clone(), T::id_of(&item).to_owned()), idx);
        if let Some(key) = T::match_key(&item) {
            self.match_index.entry(key).or_insert(idx);
        }
        if let Some(core) = &mut self.items[idx] {
            core.aliases_mut().insert(platform, item);
        }
    }

    /// Auto-correlate discovered items across platforms by matching
    /// [`Peered::match_key`].
    fn auto_correlate(&mut self, discovered: &[(PlatformId, Vec<T::Platform>)]) {
        for i in 0..discovered.len() {
            for j in (i + 1)..discovered.len() {
                let (p1, items1) = &discovered[i];
                let (p2, items2) = &discovered[j];

                let mut by_key: HashMap<String, &T::Platform> = HashMap::new();
                for item in items2 {
                    if let Some(key) = T::match_key(item) {
                        by_key.insert(key, item);
                    }
                }

                for item1 in items1 {
                    let Some(key1) = T::match_key(item1) else {
                        continue;
                    };
                    let Some(item2) = by_key.get(&key1) else {
                        continue;
                    };

                    let idx1 = self.find_index(p1, T::id_of(item1));
                    let idx2 = self.find_index(p2, T::id_of(item2));

                    match (idx1, idx2) {
                        (None, None) => {
                            let mut core = T::from_single_alias(p1.clone(), item1.clone());
                            core.aliases_mut().insert(p2.clone(), (*item2).clone());
                            self.insert(core);
                        }
                        (Some(idx), None) => self.merge_into(idx, (*item2).clone()),
                        (None, Some(idx)) => self.merge_into(idx, item1.clone()),
                        (Some(_), Some(_)) => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestItem {
        platform: PlatformId,
        id: String,
        name: String,
    }

    #[derive(Debug, Clone)]
    struct TestPeer {
        alias: HashMap<PlatformId, TestItem>,
    }

    impl Peered for TestPeer {
        type Platform = TestItem;

        fn aliases(&self) -> &HashMap<PlatformId, TestItem> {
            &self.alias
        }
        fn aliases_mut(&mut self) -> &mut HashMap<PlatformId, TestItem> {
            &mut self.alias
        }
        fn platform_of(item: &TestItem) -> &PlatformId {
            &item.platform
        }
        fn id_of(item: &TestItem) -> &str {
            &item.id
        }
        fn from_single_alias(platform: PlatformId, item: TestItem) -> Self {
            let mut alias = HashMap::new();
            alias.insert(platform, item);
            Self { alias }
        }
        fn match_key(item: &TestItem) -> Option<String> {
            Some(item.name.to_lowercase())
        }
    }

    fn item(platform: &str, id: &str, name: &str) -> TestItem {
        TestItem {
            platform: PlatformId::new(platform),
            id: id.to_owned(),
            name: name.to_owned(),
        }
    }

    fn pid(name: &str) -> PlatformId {
        PlatformId::new(name)
    }

    #[test]
    fn detach_one_alias_keeps_entity() {
        let mut peers: Peers<TestPeer> = Peers::new();
        let mut core = TestPeer::from_single_alias(pid("a"), item("a", "1", "alice"));
        core.aliases_mut()
            .insert(pid("b"), item("b", "2", "alice"));
        peers.insert(core);

        peers.detach(&pid("a"), "1");

        assert!(peers.find(&pid("a"), "1").is_none());
        let found = peers.find(&pid("b"), "2").expect("should still exist");
        assert_eq!(found.alias.len(), 1);
        assert!(found.alias.contains_key(&pid("b")));
        assert_eq!(peers.len(), 1);
    }

    #[test]
    fn detach_last_alias_removes_entity() {
        let mut peers: Peers<TestPeer> = Peers::new();
        peers.insert(TestPeer::from_single_alias(
            pid("a"),
            item("a", "1", "alice"),
        ));
        assert_eq!(peers.len(), 1);

        peers.detach(&pid("a"), "1");

        assert!(peers.find(&pid("a"), "1").is_none());
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn detach_no_stale_index() {
        let mut peers: Peers<TestPeer> = Peers::new();
        let mut core = TestPeer::from_single_alias(pid("a"), item("a", "1", "alice"));
        core.aliases_mut()
            .insert(pid("b"), item("b", "2", "alice"));
        peers.insert(core);

        peers.detach(&pid("a"), "1");

        assert!(peers.find(&pid("a"), "1").is_none());
        assert!(peers.find(&pid("b"), "2").is_some());
        assert_eq!(peers.len(), 1);
    }

    #[test]
    fn upsert_auto_correlates_by_match_key() {
        let mut peers: Peers<TestPeer> = Peers::new();
        peers.upsert(item("a", "1", "alice"));
        peers.upsert(item("b", "2", "alice"));

        assert_eq!(peers.len(), 1);
        let found = peers.find(&pid("a"), "1").expect("should exist");
        assert_eq!(found.alias.len(), 2);
        assert!(found.alias.contains_key(&pid("a")));
        assert!(found.alias.contains_key(&pid("b")));

        let same = peers.find(&pid("b"), "2").expect("should exist");
        assert_eq!(same.alias.len(), 2);
    }

    #[test]
    fn upsert_same_platform_no_merge() {
        let mut peers: Peers<TestPeer> = Peers::new();
        peers.upsert(item("a", "1", "alice"));
        peers.upsert(item("a", "2", "alice"));

        assert_eq!(peers.len(), 2);
        assert!(peers.find(&pid("a"), "1").is_some());
        assert!(peers.find(&pid("a"), "2").is_some());
    }

    #[test]
    fn upsert_updates_existing() {
        let mut peers: Peers<TestPeer> = Peers::new();
        peers.upsert(item("a", "1", "alice"));
        peers.upsert(item("a", "1", "ALICE_UPDATED"));

        assert_eq!(peers.len(), 1);
        let found = peers.find(&pid("a"), "1").expect("should exist");
        assert_eq!(found.alias[&pid("a")].name, "ALICE_UPDATED");
    }
}
