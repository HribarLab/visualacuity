extern crate lru;

use lru::LruCache;
use std::cell::RefCell;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;

pub(crate) struct LruCacher<K, V> {
    cache: RefCell<LruCache<K, Arc<V>>>,
}

impl<K: Clone + Hash + Eq, V: Clone> LruCacher<K, V> {
    pub(crate) fn new(cache_size: usize) -> Self {
        let cache_size = NonZeroUsize::new(cache_size).unwrap();
        let cache = RefCell::new(LruCache::new(cache_size));
        Self { cache }
    }

    pub(crate) fn get<F>(&self, key: &K, func: F) -> V
    where
        F: Fn() -> V,
    {
        let mut cache = self.cache.borrow_mut();
        match cache.get(key) {
            Some(cached_result) => V::clone(cached_result),
            None => {
                let computed = func();
                cache.put(key.clone(), Arc::new(computed.clone()));
                computed
            }
        }
    }
}
