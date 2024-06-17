use std::collections::BTreeSet;
use std::ops::Deref;
use std::sync::Arc;

use dashmap::DashMap;

use crate::resp::bulkstring::RespBulkString;
use crate::resp::frame::RespFrame;

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
    set: DashMap<String, BTreeSet<RespFrame>>,
}

impl Deref for Backend {
    type Target = BackendInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
            hmap: DashMap::new(),
            set: DashMap::new(),
        }
    }
}

impl Backend {
    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn set(&mut self, key: &str, val: RespFrame) {
        self.map.insert(key.to_string(), val);
    }

    pub fn hget(&self, key: &str, filed: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(filed).map(|v| v.value().clone()))
    }

    pub fn hset(&mut self, key: &str, field: &str, val: RespFrame) {
        self.hmap
            .entry(key.to_string())
            .or_default()
            .insert(field.to_string(), val);
    }

    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.value().clone())
    }

    pub fn hmget(&self, key: &str, fileds: &[String]) -> Vec<RespFrame> {
        let Some(inner) = self.hmap.get(key) else {
            return vec![RespBulkString::null().into(); fileds.len()];
        };

        let mut vec = Vec::with_capacity(fileds.len());
        for field in fileds.iter() {
            match inner.get(field).map(|v| v.value().clone()) {
                Some(frame) => vec.push(frame),
                None => vec.push(RespBulkString::null().into()),
            }
        }
        vec
    }

    pub fn sadd(&self, key: &str, member: RespFrame) -> i64 {
        if self.sismember(key, &member) == 1 {
            return 0;
        }
        self.set.entry(key.to_string()).or_default().insert(member);
        1
    }

    pub fn sismember(&self, key: &str, member: &RespFrame) -> i64 {
        let Some(inner) = self.set.get(key) else {
            return 0;
        };
        if inner.contains(member) {
            1
        } else {
            0
        }
    }
}
