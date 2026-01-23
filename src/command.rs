use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use redis_protocol::resp2::types::BytesFrame;
use std::collections::HashMap;

use crate::{db::Db, keyspace::Value};

#[derive(Debug)]
pub enum Command {
    Get {
        key: Bytes,
        keyspace: Bytes,
    },
    Set {
        key: Bytes,
        value: Bytes,
        keyspace: Bytes,
    },
    Del {
        key: Bytes,
        keyspace: Bytes,
    },
    KsCreate {
        name: Bytes,
        options: HashMap<Bytes, Bytes>,
    },
    KsDrop {
        name: Bytes,
    },
    KsAlter {
        name: Bytes,
        options: HashMap<Bytes, Bytes>,
    },
    KsList,
    KsExists {
        name: Bytes,
    },
}

impl TryFrom<BytesFrame> for Command {
    type Error = anyhow::Error;

    fn try_from(frame: BytesFrame) -> Result<Self, Self::Error> {
        let array = match frame {
            BytesFrame::Array(a) => a,
            _ => bail!("expected array frame"),
        };

        let mut iter = array.into_iter();

        let cmd_name = extract_bulk(&mut iter, "command name")?;

        match cmd_name.to_ascii_uppercase().as_slice() {
            b"GET" => {
                let keyspace = extract_bulk(&mut iter, "keyspace")?;
                let key = extract_bulk(&mut iter, "key")?;
                Ok(Command::Get { key, keyspace })
            }
            b"SET" => {
                let keyspace = extract_bulk(&mut iter, "keyspace")?;
                let key = extract_bulk(&mut iter, "key")?;
                let value = extract_bulk(&mut iter, "value")?;
                Ok(Command::Set {
                    key,
                    value,
                    keyspace,
                })
            }
            b"DEL" => {
                let keyspace = extract_bulk(&mut iter, "keyspace")?;
                let key = extract_bulk(&mut iter, "key")?;
                Ok(Command::Del { key, keyspace })
            }
            b"KS.CREATE" => {
                let name = extract_bulk(&mut iter, "keyspace")?;
                let options = extract_option_pairs(&mut iter)?;
                Ok(Command::KsCreate { name, options })
            }
            b"KS.DROP" => {
                let name = extract_bulk(&mut iter, "keyspace")?;
                ensure_no_args(&mut iter)?;
                Ok(Command::KsDrop { name })
            }
            b"KS.ALTER" => {
                let name = extract_bulk(&mut iter, "keyspace")?;
                let options = extract_option_pairs(&mut iter)?;
                Ok(Command::KsAlter { name, options })
            }
            b"KS.LIST" => {
                ensure_no_args(&mut iter)?;
                Ok(Command::KsList)
            }
            b"KS.EXISTS" => {
                let name = extract_bulk(&mut iter, "keyspace")?;
                ensure_no_args(&mut iter)?;
                Ok(Command::KsExists { name })
            }
            other => bail!("unknown command: {}", String::from_utf8_lossy(other)),
        }
    }
}

fn extract_bulk(iter: &mut impl Iterator<Item = BytesFrame>, name: &str) -> Result<Bytes> {
    match iter.next() {
        Some(BytesFrame::BulkString(b)) => Ok(b),
        Some(_) => bail!("expected bulk string for {}", name),
        None => bail!("missing argument: {}", name),
    }
}

fn ensure_no_args(iter: &mut impl Iterator<Item = BytesFrame>) -> Result<()> {
    if iter.next().is_some() {
        bail!("unexpected extra argument");
    }
    Ok(())
}

fn extract_option_pairs(
    iter: &mut impl Iterator<Item = BytesFrame>,
) -> Result<HashMap<Bytes, Bytes>> {
    let mut args = Vec::new();
    while let Some(frame) = iter.next() {
        match frame {
            BytesFrame::BulkString(b) => args.push(b),
            _ => bail!("expected bulk string for options"),
        }
    }

    if args.len() % 2 != 0 {
        bail!("missing value for option");
    }

    let mut options = HashMap::with_capacity(args.len() / 2);
    let mut iter = args.into_iter();
    while let Some(key) = iter.next() {
        let value = iter.next().expect("option value checked above");
        options.insert(key, value);
    }

    Ok(options)
}

impl Command {
    pub fn execute(self, db: &Db) -> Result<BytesFrame> {
        match self {
            Command::Set {
                key,
                keyspace,
                value,
            } => {
                db.with_keyspace_mut(&keyspace, |ks| {
                    ks.set(key, value);
                })
                .ok_or_else(|| anyhow!("ERR keyspace does not exist"))?;
                Ok(BytesFrame::SimpleString(Bytes::from("OK")))
            }
            Command::Get { key, keyspace } => {
                let result = db
                    .with_keyspace(&keyspace, |ks| {
                        ks.get(&key).map(|entry| match entry.value() {
                            Value::Bytes(bytes) => bytes.clone(),
                            Value::Integer(i) => Bytes::from(i.to_string()),
                            Value::Float(f) => Bytes::from(f.to_string()),
                        })
                    })
                    .ok_or_else(|| anyhow!("ERR keyspace does not exist"))?;

                match result {
                    Some(value) => Ok(BytesFrame::BulkString(value)),
                    None => Ok(BytesFrame::Null),
                }
            }
            Command::Del { key, keyspace } => {
                let deleted = db
                    .with_keyspace_mut(&keyspace, |ks| ks.del(&key))
                    .ok_or_else(|| anyhow!("ERR keyspace does not exist"))?;
                Ok(BytesFrame::BulkString(Bytes::from(if deleted {
                    "1"
                } else {
                    "0"
                })))
            }
            Command::KsCreate { name, options } => {
                db.create_keyspace(name, options)?;
                Ok(BytesFrame::SimpleString(Bytes::from("OK")))
            }
            Command::KsDrop { name } => {
                db.drop_keyspace(&name)?;
                Ok(BytesFrame::SimpleString(Bytes::from("OK")))
            }
            Command::KsAlter { name, options } => {
                db.alter_keyspace(&name, options)?;
                Ok(BytesFrame::SimpleString(Bytes::from("OK")))
            }
            Command::KsList => {
                let keyspaces = db.list_keyspaces();
                let items = keyspaces.into_iter().map(BytesFrame::BulkString).collect();
                Ok(BytesFrame::Array(items))
            }
            Command::KsExists { name } => Ok(BytesFrame::Integer(if db.keyspace_exists(&name) {
                1
            } else {
                0
            })),
        }
    }
}
