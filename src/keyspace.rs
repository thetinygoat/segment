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
}

impl Keyspace {
    pub fn new() -> Self {
        Keyspace {
            store: HashMap::with_capacity(4096),
            mem_size: 0,
        }
    }
}
