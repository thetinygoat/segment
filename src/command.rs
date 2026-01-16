use anyhow::bail;
use bytes::Bytes;
use redis_protocol::resp2::types::BytesFrame;

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
