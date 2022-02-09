use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration};
use ic_cdk::api::time;

struct ValueWrapper<V> {
    timeout: u64,
    val: V,
}

pub struct TtlHashMap<K, V>
    where
        K: Eq + Hash
{
    map: HashMap<K, ValueWrapper<V>>,
    pub ttl: Duration,
    oldest: Option<u64>,
    pub max_nodes: Option<usize>,
}


impl<K, V> TtlHashMap<K, V>
    where
        K: Eq + Hash
{
    pub fn new(ttl: Duration) -> Self {
        TtlHashMap {
            map: HashMap::new(),
            ttl,
            oldest: None,
            max_nodes: None,
        }
    }

    pub fn split_by_num_bound(&mut self, max: usize) -> HashMap<K, V> {
        let mut stale = HashMap::new();

        if self.map.len() > max {
            let mut v = self.to_sorted_vec();
            while v.len() > max {
                if let Some((k, v)) = v.pop() {
                    stale.insert(k, v.val);
                } else {
                    break;
                }
            }

            if let Some((_, v)) = v.last() {
                self.oldest = Some(v.timeout);
            } else {
                self.oldest = None;
            }

            for (k, v) in v.drain(..) {
                self.map.insert(k, v);
            }
        }
        stale
    }

    pub fn cleanup(&mut self) {
        if let Some(oldest) = self.oldest {
            let now = time();

            if now > oldest {
                let mut new_oldest: Option<u64> = None;
                self.map.retain(|_, v| {
                    let keep = v.timeout > now;


                    if keep {
                        if let Some(no) = new_oldest {
                            if v.timeout < no {
                                new_oldest = Some(v.timeout);
                            }
                        } else {
                            new_oldest = Some(v.timeout);
                        }
                    }
                    keep
                });

                self.oldest = new_oldest;
            }
        }

        if let Some(max) = self.max_nodes {
            self.split_by_num_bound(max);
        }
    }

    fn update_oldest(&mut self, croaktime: u64) {
        if let Some(oldest) = self.oldest {
            if croaktime < oldest {
                self.oldest = Some(croaktime)
            }
        } else {
            self.oldest = Some(croaktime)
        }
    }

    pub fn touch<Q: ?Sized>(&mut self, key: &Q)
        where
            K: Borrow<Q>,
            Q: Hash + Eq
    {
        if let Some(v) = self.map.get_mut(key) {
            let croaktime = time() + self.ttl.as_nanos() as u64;

            v.timeout = croaktime;

            self.update_oldest(croaktime);
        }
    }

    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&V>
        where
            K: Borrow<Q>,
            Q: Hash + Eq
    {
        self.cleanup();
        self.touch(key);
        self.get_raw(key)
    }


    pub fn get_raw<Q: ?Sized>(&self, key: &Q) -> Option<&V>
        where
            K: Borrow<Q>,
            Q: Hash + Eq
    {
        match self.map.get(key) {
            Some(v) => Some(&(v.val)),
            None => None
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.cleanup();

        let croaktime = time() + self.ttl.as_nanos() as u64;
        let ret = self.map.insert(
            key,
            ValueWrapper {
                timeout: croaktime,
                val: value,
            },
        );
        self.update_oldest(croaktime);

        match ret {
            Some(v) => Some(v.val),
            None => None
        }
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
        where
            K: Borrow<Q>,
            Q: Hash + Eq
    {
        self.cleanup();
        match self.map.remove(key) {
            Some(v) => Some(v.val),
            None => None
        }
    }

    fn to_sorted_vec(&mut self) -> Vec<(K, ValueWrapper<V>)> {
        let it = self.map.drain();
        let mut v: Vec<(K, ValueWrapper<V>)> = it.collect();
        v.sort_unstable_by(|a, b| b.1.timeout.partial_cmp(&a.1.timeout).unwrap());
        v
    }
}
