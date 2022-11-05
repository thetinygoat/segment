use crate::{
    command::{Command, Count, Create, Del, Drop, Get, Set, Ttl},
    connection::ConnectionError,
    frame::Frame,
};
use bytes::Bytes;
use crossbeam::sync::WaitGroup;
use libproc::libproc::pid_rusage::{pidrusage, RUsageInfoV2};
use parking_lot::{Mutex, RwLock};
use std::process;
use std::{
    collections::HashMap,
    str::{self, Utf8Error},
    time::Duration,
};
use std::{
    sync::Arc,
    time::{Instant, SystemTime, SystemTimeError, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::time;
use tracing::{debug, error};

static EXPIRING_EVICTOR_SAMPLE_SIZE: u8 = 5;
static MAX_MEMORY_EVICTOR_SAMPLE_SIZE: u8 = 3;

#[derive(Debug)]
pub struct Value {
    data: Bytes,
    last_accessed: Instant,
    expire_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evictor {
    Nop,
    Random,
    Lru,
}

#[derive(Debug)]
pub struct Keyspace {
    store: Arc<Mutex<HashMap<Bytes, Value>>>,
    expiring: Arc<Mutex<HashMap<Bytes, u64>>>,
    evictor: Evictor,
    wg: WaitGroup,
    done: broadcast::Receiver<()>,
    server_max_memory: u64,
}

#[derive(Debug)]
pub struct Db {
    keyspaces: RwLock<HashMap<Bytes, Keyspace>>,
    done: broadcast::Receiver<()>,
    wg: WaitGroup,
    server_max_memory: u64,
}

#[derive(Debug, Error)]
pub enum ExecuteCommandError {
    #[error(transparent)]
    ConnectionError(#[from] ConnectionError),

    #[error("keyspace '{0}' already exists")]
    KeyspaceExists(String),

    #[error("keyspace '{0}' does not exist")]
    KeyspaceDoesNotExist(String),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),

    #[error(transparent)]
    SystemTimeError(#[from] SystemTimeError),
}

impl Db {
    pub fn new(done: broadcast::Receiver<()>, wg: WaitGroup, server_max_memory: u64) -> Self {
        Db {
            keyspaces: RwLock::new(HashMap::new()),
            done,
            wg,
            server_max_memory,
        }
    }

    pub async fn execute(&self, command: Command) -> Result<Frame, ExecuteCommandError> {
        match command {
            Command::Create(cmd) => self.exec_create(&cmd).await,
            Command::Drop(cmd) => self.exec_drop(&cmd),
            Command::Keyspaces => self.exec_keyspaces(),
            Command::Set(cmd) => self.exec_set(&cmd),
            Command::Ping => Ok(Frame::String(Bytes::from_static(b"PONG"))),
            Command::Get(cmd) => self.exec_get(&cmd),
            Command::Del(cmd) => self.exec_del(&cmd),
            Command::Count(cmd) => self.exec_count(&cmd),
            Command::Ttl(cmd) => self.exec_ttl(&cmd),
        }
    }

    async fn exec_create(&self, cmd: &Create) -> Result<Frame, ExecuteCommandError> {
        let mut handle = self.keyspaces.write();
        if handle.contains_key(&cmd.keyspace()) {
            if cmd.if_not_exists() {
                return Ok(Frame::Boolean(false));
            } else {
                return Err(ExecuteCommandError::KeyspaceExists(
                    str::from_utf8(&cmd.keyspace()[..])?.to_string(),
                ));
            }
        }

        let ks = Keyspace::new(
            self.done.resubscribe(),
            self.wg.clone(),
            cmd.evictor(),
            self.server_max_memory,
        );
        ks.start_expiring_evictor();

        if self.server_max_memory > 0 {
            ks.start_max_memory_evictor();
        }

        handle.insert(cmd.keyspace(), ks);

        Ok(Frame::Boolean(true))
    }

    fn exec_drop(&self, cmd: &Drop) -> Result<Frame, ExecuteCommandError> {
        let mut handle = self.keyspaces.write();
        if !handle.contains_key(&cmd.keyspace()) {
            if cmd.if_exists() {
                return Ok(Frame::Boolean(false));
            } else {
                return Err(ExecuteCommandError::KeyspaceDoesNotExist(
                    str::from_utf8(&cmd.keyspace()[..])?.to_string(),
                ));
            }
        }
        handle.remove(&cmd.keyspace());
        Ok(Frame::Boolean(true))
    }

    fn exec_keyspaces(&self) -> Result<Frame, ExecuteCommandError> {
        let handle = self.keyspaces.read();
        let mut keyspaces = Vec::with_capacity(handle.keys().count());
        for key in handle.keys() {
            keyspaces.push(Frame::String(key.clone()))
        }
        Ok(Frame::Array(keyspaces))
    }

    fn exec_set(&self, cmd: &Set) -> Result<Frame, ExecuteCommandError> {
        let handle = self.keyspaces.read();
        let keyspace = handle.get(&cmd.keyspace());
        if let Some(ks) = keyspace {
            if cmd.if_exists() || cmd.if_not_exists() {
                if cmd.if_exists() {
                    return ks.set_if_exists(cmd.key(), cmd.value(), cmd.expire_at());
                } else {
                    return ks.set_if_not_exists(cmd.key(), cmd.value(), cmd.expire_at());
                }
            } else {
                return ks.set(cmd.key(), cmd.value(), cmd.expire_at());
            }
        }

        Err(ExecuteCommandError::KeyspaceDoesNotExist(
            str::from_utf8(&cmd.keyspace()[..])?.to_string(),
        ))
    }

    fn exec_get(&self, cmd: &Get) -> Result<Frame, ExecuteCommandError> {
        let handle = self.keyspaces.read();
        let keyspace = handle.get(&cmd.keyspace());
        if let Some(ks) = keyspace {
            return ks.get(cmd.key());
        }

        Err(ExecuteCommandError::KeyspaceDoesNotExist(
            str::from_utf8(&cmd.keyspace()[..])?.to_string(),
        ))
    }

    fn exec_del(&self, cmd: &Del) -> Result<Frame, ExecuteCommandError> {
        let handle = self.keyspaces.read();
        let keyspace = handle.get(&cmd.keyspace());
        if let Some(ks) = keyspace {
            return ks.del(cmd.key());
        }

        Err(ExecuteCommandError::KeyspaceDoesNotExist(
            str::from_utf8(&cmd.keyspace()[..])?.to_string(),
        ))
    }

    fn exec_count(&self, cmd: &Count) -> Result<Frame, ExecuteCommandError> {
        let handle = self.keyspaces.read();
        let keyspace = handle.get(&cmd.keyspace());
        if let Some(ks) = keyspace {
            return ks.count();
        }

        Err(ExecuteCommandError::KeyspaceDoesNotExist(
            str::from_utf8(&cmd.keyspace()[..])?.to_string(),
        ))
    }

    fn exec_ttl(&self, cmd: &Ttl) -> Result<Frame, ExecuteCommandError> {
        let handle = self.keyspaces.read();
        let keyspace = handle.get(&cmd.keyspace());
        if let Some(ks) = keyspace {
            return ks.ttl(cmd.key());
        }

        Err(ExecuteCommandError::KeyspaceDoesNotExist(
            str::from_utf8(&cmd.keyspace()[..])?.to_string(),
        ))
    }
}

impl Keyspace {
    pub fn new(
        done: broadcast::Receiver<()>,
        wg: WaitGroup,
        evictor: Evictor,
        server_max_memory: u64,
    ) -> Self {
        Keyspace {
            store: Arc::new(Mutex::new(HashMap::new())),
            expiring: Arc::new(Mutex::new(HashMap::new())),
            evictor,
            done,
            wg,
            server_max_memory,
        }
    }
    pub fn set_if_not_exists(
        &self,
        key: Bytes,
        value: Bytes,
        expire_at: Option<u64>,
    ) -> Result<Frame, ExecuteCommandError> {
        let handle = self.store.lock();
        let val = handle.get(&key);
        if val.is_some() {
            return Ok(Frame::Boolean(false));
        }
        drop(handle);
        self.set(key, value, expire_at)
    }

    pub fn set_if_exists(
        &self,
        key: Bytes,
        value: Bytes,
        expire_at: Option<u64>,
    ) -> Result<Frame, ExecuteCommandError> {
        let handle = self.store.lock();
        let val = handle.get(&key);
        if val.is_none() {
            return Ok(Frame::Boolean(false));
        }
        drop(handle);
        self.set(key, value, expire_at)
    }

    pub fn set(
        &self,
        key: Bytes,
        value: Bytes,
        expire_at: Option<u64>,
    ) -> Result<Frame, ExecuteCommandError> {
        let mut handle = self.store.lock();
        let value = Value::new(value, expire_at);
        handle.insert(key.clone(), value);
        if let Some(expiry) = expire_at {
            let mut expring_handle = self.expiring.lock();
            expring_handle.insert(key, expiry);
        }
        Ok(Frame::Boolean(true))
    }

    pub fn get(&self, key: Bytes) -> Result<Frame, ExecuteCommandError> {
        let mut handle = self.store.lock();
        if let Some(val) = handle.get_mut(&key) {
            val.touch();
            if let Some(expiry) = val.expire_at() {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                if expiry < current_time {
                    handle.remove(&key);
                    return Ok(Frame::Null);
                }
            }
            return Ok(Frame::String(val.data()));
        }
        Ok(Frame::Null)
    }

    pub fn del(&self, key: Bytes) -> Result<Frame, ExecuteCommandError> {
        let mut handle = self.store.lock();
        let result = handle.remove(&key);
        Ok(Frame::Boolean(result.is_some()))
    }

    pub fn count(&self) -> Result<Frame, ExecuteCommandError> {
        let handle = self.store.lock();
        let count = handle.iter().count();
        Ok(Frame::Integer(count as i64))
    }

    pub fn ttl(&self, key: Bytes) -> Result<Frame, ExecuteCommandError> {
        let mut handle = self.store.lock();
        if let Some(val) = handle.get_mut(&key) {
            val.touch();
            if let Some(expiry) = val.expire_at() {
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                if expiry <= current_time {
                    handle.remove(&key);
                    return Ok(Frame::Null);
                } else {
                    return Ok(Frame::Integer(((expiry - current_time) * 1000) as i64));
                }
            }
            return Ok(Frame::Null);
        }
        Ok(Frame::Null)
    }
    fn start_expiring_evictor(&self) {
        let mut done = self.done.resubscribe();
        let wg = self.wg.clone();
        let expiring = self.expiring.clone();
        let store = self.store.clone();
        tokio::spawn(async move {
            debug!("expiring evictor started");
            loop {
                tokio::select! {
                    _ = done.recv() => {
                        drop(wg);
                        debug!("shutting down expiring evictor");
                        break;
                    }
                    _ = time::sleep(Duration::from_millis(500)) => {
                        let mut expring_handle = expiring.lock();
                        let mut store_handle = store.lock();
                        let mut expired_keys = Vec::with_capacity(5);

                        for (idx, (key, expiry)) in expring_handle.iter().enumerate() {
                            if idx >= EXPIRING_EVICTOR_SAMPLE_SIZE as usize {
                                break;
                            }

                            let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
                                Ok(time) => time.as_secs(),
                                Err(e) => {
                                    error!("{}", e);
                                    break;
                                }
                            };
                            if *expiry <= current_time {
                                expired_keys.push(key.clone());
                                store_handle.remove(key);
                            }
                        }

                        for key in expired_keys {
                            expring_handle.remove(&key);
                        }
                    }
                }
            }
        });
    }

    fn start_max_memory_evictor(&self) {
        if self.evictor == Evictor::Nop {
            return;
        }
        let mut done = self.done.resubscribe();
        let wg = self.wg.clone();
        let store = self.store.clone();
        let evictor = self.evictor;
        let server_max_memory = self.server_max_memory;
        tokio::spawn(async move {
            debug!("max memory evictor started");
            loop {
                tokio::select! {
                    _ = done.recv() => {
                        drop(wg);
                        debug!("shutting down max memory evictor");
                        break;
                    }
                    _ = time::sleep(Duration::from_millis(1000)) => {
                        let memory_usage = match pidrusage::<RUsageInfoV2>(process::id() as i32) {
                            Ok(rusage) => rusage.ri_resident_size,
                            Err(e) => {
                                error!("{}", e);
                                break;
                            }
                        };

                        if memory_usage < server_max_memory {
                            continue;
                        }

                        match evictor {
                            Evictor::Lru => {
                                let mut handle = store.lock();
                                let mut lru = Instant::now();
                                let mut to_evict: Option<Bytes> = None;
                                for (idx, (key, value)) in handle.iter().enumerate() {
                                    if idx >= MAX_MEMORY_EVICTOR_SAMPLE_SIZE as usize {
                                        break;
                                    }

                                    let last_accessed = value.last_accessed();

                                    if last_accessed < lru {
                                        lru = last_accessed;
                                        to_evict = Some(key.clone());
                                    }
                                }

                                if let Some(key) = to_evict {
                                    debug!("key '{:?}' evicted using lru policy", key);
                                    handle.remove(&key);
                                }
                            },
                            Evictor::Random => {
                                let mut handle = store.lock();
                                let mut to_evict: Option<Bytes> = None;
                                for (idx, key) in handle.keys().enumerate() {
                                    if idx >= MAX_MEMORY_EVICTOR_SAMPLE_SIZE as usize {
                                        break;
                                    }
                                    to_evict = Some(key.clone());
                                }

                                if let Some(key) = to_evict {
                                    debug!("key '{:?}' evicted using random policy", key);
                                    handle.remove(&key);
                                }
                            },
                            _ => unreachable!(),
                        }
                    }
                }
            }
        });
    }
}

impl Value {
    pub fn new(data: Bytes, expire_at: Option<u64>) -> Self {
        Value {
            data,
            last_accessed: Instant::now(),
            expire_at,
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }

    pub fn data(&self) -> Bytes {
        self.data.clone()
    }

    pub fn expire_at(&self) -> Option<u64> {
        self.expire_at
    }

    pub fn last_accessed(&self) -> Instant {
        self.last_accessed
    }
}
