use bytes::Bytes;
use std::collections::HashMap;

pub enum Value {
    Bytes(Bytes),
    Integer(isize),
    Float(f64),
}

pub struct Entry {
    value: Value,
    ttl: Option<u64>,
}

pub struct Keyspace {
    store: HashMap<Bytes, Entry>,
    mem_size: usize,
    options: HashMap<Bytes, Bytes>,
}

impl Keyspace {
    pub fn new() -> Self {
        Keyspace {
            store: HashMap::with_capacity(4096),
            mem_size: 0,
            options: HashMap::new(),
        }
    }

    pub fn with_options(options: HashMap<Bytes, Bytes>) -> Self {
        Keyspace {
            store: HashMap::with_capacity(4096),
            mem_size: 0,
            options,
        }
    }

    pub fn set(&mut self, key: Bytes, value: Bytes) {
        self.store.insert(
            key,
            Entry {
                value: Value::Bytes(value),
                ttl: None,
            },
        );
    }

    pub fn get(&self, key: &Bytes) -> Option<&Entry> {
        self.store.get(key)
    }

    pub fn del(&mut self, key: &Bytes) -> bool {
        self.store.remove(key).is_some()
    }

    pub fn apply_options(&mut self, options: HashMap<Bytes, Bytes>) {
        self.options.extend(options);
    }
}

impl Entry {
    pub fn value(&self) -> &Value {
        &self.value
    }
}
