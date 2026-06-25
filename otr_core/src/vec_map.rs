use std::ops::Deref;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct VecMap<K: PartialEq, V> {
    entries: Vec<(K, V)>
}

#[allow(unused)]
impl<KLookup, KOwn, V> VecMap<KOwn, V>
where
    KLookup: ?Sized + PartialEq,
    KOwn: PartialEq + Deref<Target = KLookup>
{
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new()
        }
    }

    pub(crate) fn insert(&mut self, key: KOwn, mut value: V) -> Option<V> {
        for (k, v) in self.entries.iter_mut() {
            if k == &key {
                std::mem::swap(v, &mut value);
                return Some(value)
            }
        }
        self.entries.push((key, value));
        None
    }

    pub(crate) fn remove(&mut self, key: impl AsRef<KLookup>) -> Option<V> {
        let mut index = None;

        for (i, (k, _)) in self.entries.iter_mut().enumerate() {
            let k = k as &KOwn;
            if k.deref() == key.as_ref() {
                index = Some(i);
            }
        }

        if let Some(index) = index {
            Some(self.entries.remove(index).1)
        } else {
            None
        }
    }

    pub fn get(&self, key: impl AsRef<KLookup>) -> Option<&V> {
        for (k, v) in self.entries.iter() {
            if k.deref() == key.as_ref() {
                return Some(v)
            }
        }

        None
    }

    pub(crate) fn get_mut(&mut self, key: impl AsRef<KLookup>) -> Option<&mut V> {
        for (k, v) in self.entries.iter_mut() {
            let k = k as &KOwn;
            if k.deref() == key.as_ref() {
                return Some(v)
            }
        }

        None
    }

    pub(crate) fn contains_key(&self, key: impl AsRef<KLookup>) -> bool {
        for (k, _) in self.entries.iter() {
            if k.deref() == key.as_ref() {
                return true
            }
        }

        false
    }

    pub(crate) fn iter(&self) -> core::slice::Iter<'_, (KOwn, V)> {
        self.entries.iter()
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}