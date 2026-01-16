use crate::keyspace::Keyspace;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type KeyspaceHandle = Arc<RwLock<Keyspace>>;

pub struct Db {
    keyspaces: RwLock<HashMap<Bytes, KeyspaceHandle>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            keyspaces: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_or_create_keyspace(&self, name: &Bytes) -> KeyspaceHandle {
        if let Some(handle) = self.keyspaces.read().unwrap().get(name).cloned() {
            return handle;
        }

        let mut keyspaces = self.keyspaces.write().unwrap();
        keyspaces
            .entry(name.clone())
            .or_insert_with(|| Arc::new(RwLock::new(Keyspace::new())))
            .clone()
    }

    pub fn with_keyspace<F, R>(&self, name: &Bytes, f: F) -> Option<R>
    where
        F: FnOnce(&Keyspace) -> R,
    {
        let handle = self.get_or_create_keyspace(name);
        let keyspace = handle.read().unwrap();
        Some(f(&keyspace))
    }

    pub fn with_keyspace_mut<F, R>(&self, name: &Bytes, f: F) -> Option<R>
    where
        F: FnOnce(&mut Keyspace) -> R,
    {
        let handle = self.get_or_create_keyspace(name);
        let mut keyspace = handle.write().unwrap();
        Some(f(&mut keyspace))
    }
}
