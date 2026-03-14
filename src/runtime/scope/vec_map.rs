
#[derive(Debug, Clone)]
pub(crate) struct VecMap<K: PartialEq, V> {
    entries: Vec<(K, V)>
}

impl<K: PartialEq, V> VecMap<K, V> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new()
        }
    }

    pub(crate) fn insert(&mut self, key: K, mut value: V) -> Option<V> {
        for (k, v) in self.entries.iter_mut() {
            if k == &key {
                std::mem::swap(v, &mut value);
                return Some(value)
            }
        }
        self.entries.push((key, value));
        None
    }

    pub(crate) fn remove(&mut self, key: &K) -> Option<V> {
        let mut index = None;

        for (i, (k, _)) in self.entries.iter_mut().enumerate() {
            if k == key {
                index = Some(i);
            }
        }

        if let Some(index) = index {
            Some(self.entries.remove(index).1)
        } else {
            None
        }
    }

    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        for (k, v) in self.entries.iter() {
            if k == key {
                return Some(v)
            }
        }

        None
    }

    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        for (k, v) in self.entries.iter_mut() {
            if k == key {
                return Some(v)
            }
        }

        None
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        for (k, _) in self.entries.iter() {
            if k == key {
                return true
            }
        }

        false
    }
}