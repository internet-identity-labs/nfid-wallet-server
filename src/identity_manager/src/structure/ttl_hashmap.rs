use std::collections::HashMap;

pub struct TtlHashMap<K, V> {
    map: HashMap<K, (V, u64)>,
    ttl_millis: u64,
}

impl<K, V> TtlHashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new(ttl_millis: u64) -> Self {
        TtlHashMap {
            map: HashMap::new(),
            ttl_millis,
        }
    }

    pub fn insert(&mut self, key: K, value: V, timestamp: u64) {
        self.map.insert(key, (value, timestamp));
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|(value, _)| value)
    }

    pub fn clean_expired_entries(&mut self, timestamp: u64) {
        let mut deletion_timestamp = timestamp - self.ttl_millis;
        self.map.retain(|_, (_, expiration)| expiration > &mut deletion_timestamp)
    }
}
