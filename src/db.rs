use crate::keyspace::Keyspace;
use anyhow::{bail, Result};
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

    pub fn get_keyspace(&self, name: &Bytes) -> Option<KeyspaceHandle> {
        self.keyspaces.read().unwrap().get(name).cloned()
    }

    pub fn with_keyspace<F, R>(&self, name: &Bytes, f: F) -> Option<R>
    where
        F: FnOnce(&Keyspace) -> R,
    {
        let handle = self.get_keyspace(name)?;
        let keyspace = handle.read().unwrap();
        Some(f(&keyspace))
    }

    pub fn with_keyspace_mut<F, R>(&self, name: &Bytes, f: F) -> Option<R>
    where
        F: FnOnce(&mut Keyspace) -> R,
    {
        let handle = self.get_keyspace(name)?;
        let mut keyspace = handle.write().unwrap();
        Some(f(&mut keyspace))
    }

    pub fn create_keyspace(
        &self,
        name: Bytes,
        options: std::collections::HashMap<Bytes, Bytes>,
    ) -> Result<()> {
        let mut keyspaces = self.keyspaces.write().unwrap();
        if keyspaces.contains_key(&name) {
            bail!("ERR keyspace already exists");
        }

        keyspaces.insert(
            name,
            Arc::new(RwLock::new(Keyspace::with_options(options))),
        );

        Ok(())
    }

    pub fn drop_keyspace(&self, name: &Bytes) -> Result<()> {
        let mut keyspaces = self.keyspaces.write().unwrap();
        if keyspaces.remove(name).is_none() {
            bail!("ERR keyspace does not exist");
        }

        Ok(())
    }

    pub fn alter_keyspace(
        &self,
        name: &Bytes,
        options: std::collections::HashMap<Bytes, Bytes>,
    ) -> Result<()> {
        let keyspaces = self.keyspaces.write().unwrap();
        let handle = match keyspaces.get(name) {
            Some(handle) => handle.clone(),
            None => bail!("ERR keyspace does not exist"),
        };

        let mut keyspace = handle.write().unwrap();
        keyspace.apply_options(options);
        Ok(())
    }

    pub fn keyspace_exists(&self, name: &Bytes) -> bool {
        self.keyspaces.read().unwrap().contains_key(name)
    }

    pub fn list_keyspaces(&self) -> Vec<Bytes> {
        let mut names: Vec<Bytes> = self.keyspaces.read().unwrap().keys().cloned().collect();
        names.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
        names
    }
}
