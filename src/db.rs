use anyhow::{bail, Result};
use bytes::Bytes;
use std::collections::HashMap;

use crate::keyspace::Keyspace;

pub struct Db {
    keyspaces: HashMap<Bytes, Keyspace>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            keyspaces: HashMap::new(),
        }
    }

    pub fn new_keyspace(&mut self, name: Bytes) -> Result<()> {
        if self.keyspaces.contains_key(&name) {
            bail!("keyspace already exists")
        }

        self.keyspaces.insert(name, Keyspace::new());

        Ok(())
    }
}
