use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct BiMap<L, R> {
    left_to_right: HashMap<L, R>,
    right_to_left: HashMap<R, L>,
}

impl<L, R> BiMap<L, R>
where
    L: Eq + Hash + Clone,
    R: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            left_to_right: HashMap::new(),
            right_to_left: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            left_to_right: HashMap::with_capacity(capacity),
            right_to_left: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, left: L, right: R) -> Option<(L, R)> {
        let evicted_by_left = self.remove_by_left(&left);
        let evicted_by_right = self.remove_by_right(&right);

        self.left_to_right.insert(left.clone(), right.clone());
        self.right_to_left.insert(right, left);

        evicted_by_left.or(evicted_by_right)
    }

    pub fn get_by_left<Q>(&self, left: &Q) -> Option<&R>
    where
        L: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.left_to_right.get(left)
    }

    pub fn get_by_right<Q>(&self, right: &Q) -> Option<&L>
    where
        R: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.right_to_left.get(right)
    }

    pub fn remove_by_left<Q>(&mut self, left: &Q) -> Option<(L, R)>
    where
        L: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        if let Some(right) = self.left_to_right.remove(left) {
            let left = self.right_to_left.remove(&right).unwrap();
            Some((left, right))
        } else {
            None
        }
    }

    pub fn remove_by_right<Q>(&mut self, right: &Q) -> Option<(L, R)>
    where
        R: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        if let Some(left) = self.right_to_left.remove(right) {
            let right = self.left_to_right.remove(&left).unwrap();
            Some((left, right))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.left_to_right.len()
    }

    pub fn is_empty(&self) -> bool {
        self.left_to_right.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&L, &R)> {
        self.left_to_right.iter()
    }
}

impl<L, R> Default for BiMap<L, R>
where
    L: Eq + Hash + Clone,
    R: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup() {
        let mut map = BiMap::new();
        map.insert("a", 1);
        assert_eq!(map.get_by_left(&"a"), Some(&1));
        assert_eq!(map.get_by_right(&1), Some(&"a"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn overwrite_left_evicts_old_pair() {
        let mut map = BiMap::new();
        map.insert("a", 1);
        let evicted = map.insert("a", 2);
        assert_eq!(evicted, Some(("a", 1)));
        assert_eq!(map.get_by_left(&"a"), Some(&2));
        assert_eq!(map.get_by_right(&1), None);
        assert_eq!(map.get_by_right(&2), Some(&"a"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn overwrite_right_evicts_old_pair() {
        let mut map = BiMap::new();
        map.insert("a", 1);
        let evicted = map.insert("b", 1);
        assert_eq!(evicted, Some(("a", 1)));
        assert_eq!(map.get_by_left(&"a"), None);
        assert_eq!(map.get_by_left(&"b"), Some(&1));
        assert_eq!(map.get_by_right(&1), Some(&"b"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_by_left() {
        let mut map = BiMap::new();
        map.insert("a", 1);
        let removed = map.remove_by_left(&"a");
        assert_eq!(removed, Some(("a", 1)));
        assert!(map.is_empty());
    }

    #[test]
    fn remove_by_right() {
        let mut map = BiMap::new();
        map.insert("a", 1);
        let removed = map.remove_by_right(&1);
        assert_eq!(removed, Some(("a", 1)));
        assert!(map.is_empty());
    }

    #[test]
    fn multiple_pairs() {
        let mut map = BiMap::with_capacity(3);
        map.insert("#rust", "111");
        map.insert("#general", "222");
        map.insert("#help", "333");
        assert_eq!(map.len(), 3);
        assert_eq!(map.get_by_left(&"#general"), Some(&"222"));
        assert_eq!(map.get_by_right(&"333"), Some(&"#help"));
    }
}
