use std::ops::Deref;
use std::sync::Arc;

use dashmap::DashMap;

use crate::resp::frame::RespFrame;

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
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

    pub fn hget(&self, field: &str, key: &str) -> Option<RespFrame> {
        self.hmap
            .get(field)
            .and_then(|v| v.get(key).map(|v| v.value().clone()))
    }

    pub fn hset(&mut self, field: &str, key: &str, val: RespFrame) {
        self.hmap
            .entry(field.to_string())
            .or_default()
            .insert(key.to_string(), val);
    }

    pub fn hgetall(&self, field: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(field).map(|v| v.value().clone())
    }
}
