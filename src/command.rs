use anyhow::{bail, Ok, Result};
use bytes::Bytes;
use redis_protocol::resp2::types::BytesFrame;

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
            other => bail!("unknown command: {}", String::from_utf8_lossy(other)),
        }
    }
}

fn extract_bulk(iter: &mut impl Iterator<Item = BytesFrame>, name: &str) -> anyhow::Result<Bytes> {
    match iter.next() {
        Some(BytesFrame::BulkString(b)) => Ok(b),
        Some(_) => bail!("expected bulk string for {}", name),
        None => bail!("missing argument: {}", name),
    }
}

impl Command {
    pub fn execute(self, db: &Db) -> Result<Option<Bytes>> {
        match self {
            Command::Set {
                key,
                keyspace,
                value,
            } => {
                db.with_keyspace_mut(&keyspace, |ks| {
                    ks.set(key, value);
                });
                Ok(Some(Bytes::from("OK")))
            }
            Command::Get { key, keyspace } => {
                let result = db.with_keyspace(&keyspace, |ks| {
                    ks.get(&key).map(|entry| match entry.value() {
                        Value::Bytes(bytes) => bytes.clone(),
                        Value::Integer(i) => Bytes::from(i.to_string()),
                        Value::Float(f) => Bytes::from(f.to_string()),
                    })
                });
                Ok(result.flatten())
            }
            Command::Del { key, keyspace } => {
                let deleted = db.with_keyspace_mut(&keyspace, |ks| ks.del(&key));
                if let Some(deleted) = deleted {
                    return Ok(Some(Bytes::from(if deleted { "1" } else { "0" })));
                }

                Ok(Some(Bytes::from("0")))
            }
        }
    }
}
