use crate::db::Evictor;
use crate::frame::Frame;
use bytes::Bytes;
use std::iter::Peekable;
use std::ops::Add;
use std::str::{self, Utf8Error};
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use std::vec::IntoIter;
use thiserror::Error;

#[derive(Debug)]
struct Parser {
    tokens: Peekable<IntoIter<Frame>>,
}

#[derive(Debug)]
pub struct Create {
    keyspace: Bytes,
    evictor: Evictor,
    if_not_exists: bool,
}

#[derive(Debug)]
pub struct Set {
    keyspace: Bytes,
    key: Bytes,
    value: Bytes,
    expire_at: Option<u64>,
    if_not_exists: bool,
    if_exists: bool,
}

#[derive(Debug)]
pub struct Get {
    keyspace: Bytes,
    key: Bytes,
}

#[derive(Debug)]
pub struct Del {
    keyspace: Bytes,
    key: Bytes,
}

#[derive(Debug)]
pub struct Drop {
    keyspace: Bytes,
    if_exists: bool,
}

#[derive(Debug)]
pub struct Count {
    keyspace: Bytes,
}

#[derive(Debug)]
pub struct Ttl {
    keyspace: Bytes,
    key: Bytes,
}

#[derive(Debug)]
pub enum Command {
    Create(Create),
    Set(Set),
    Get(Get),
    Del(Del),
    Drop(Drop),
    Count(Count),
    Ttl(Ttl),
    Ping,
    Keyspaces,
}

#[derive(Debug, Error)]
pub enum ParseCommandError {
    #[error("invalid command format")]
    InvalidFormat,

    #[error("wrong number of arguments for '{0}' command")]
    WrongArgCount(String),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),

    #[error("invalid argument '{0}' for '{1}' command")]
    InvalidArg(String, String),

    #[error("invalid value '{0}' for argument '{1}' for '{2}' command")]
    InvalidArgValue(String, String, String),

    #[error(transparent)]
    SystemTimeError(#[from] SystemTimeError),

    #[error("unknown command '{0}'")]
    UnknownCommand(String),
}

impl Parser {
    pub fn new(frame: Frame) -> Result<Self, ParseCommandError> {
        match frame {
            Frame::Array(tokens) => Ok(Parser {
                tokens: tokens.into_iter().peekable(),
            }),
            _ => Err(ParseCommandError::InvalidFormat),
        }
    }

    pub fn next(&mut self) -> Option<Frame> {
        self.tokens.next()
    }

    pub fn has_remaining(&mut self) -> bool {
        self.tokens.peek().is_some()
    }

    pub fn next_as_string(&mut self) -> Result<Option<String>, ParseCommandError> {
        let frame = match self.next() {
            Some(frame) => frame,
            None => return Ok(None),
        };

        match frame {
            Frame::String(data) => Ok(Some(str::from_utf8(&data[..])?.to_string())),
            _ => Err(ParseCommandError::InvalidFormat),
        }
    }

    pub fn next_as_bytes(&mut self) -> Result<Option<Bytes>, ParseCommandError> {
        let frame = match self.next() {
            Some(frame) => frame,
            None => return Ok(None),
        };

        match frame {
            Frame::String(data) => Ok(Some(data)),
            _ => Err(ParseCommandError::InvalidFormat),
        }
    }
}

impl Create {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("create".to_string()))?;

        let mut command = Create {
            keyspace,
            evictor: Evictor::Nop,
            if_not_exists: false,
        };

        if !parser.has_remaining() {
            return Ok(command);
        };

        while parser.has_remaining() {
            let key = parser
                .next_as_string()?
                .ok_or_else(|| ParseCommandError::WrongArgCount("create".to_string()))?
                .to_lowercase();

            if matches!(key.as_str(), "evictor") {
                let value = parser
                    .next_as_string()?
                    .ok_or_else(|| ParseCommandError::WrongArgCount("create".to_string()))?
                    .to_lowercase();
                match value.as_str() {
                    "nop" => command.evictor = Evictor::Nop,
                    "random" => command.evictor = Evictor::Random,
                    "lru" => command.evictor = Evictor::Lru,
                    _ => {
                        return Err(ParseCommandError::InvalidArgValue(
                            value,
                            key,
                            "create".to_string(),
                        ))
                    }
                }
            } else if matches!(key.as_str(), "if_not_exists") {
                if !command.if_not_exists {
                    command.if_not_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "create".to_string()));
            }
        }

        Ok(command)
    }

    pub fn keyspace(&self) -> Bytes {
        self.keyspace.clone()
    }
    pub fn evictor(&self) -> Evictor {
        self.evictor
    }
    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
}

impl Set {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?;

        let key = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?;

        let value = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?;

        let mut command = Set {
            keyspace,
            key,
            value,
            expire_at: None,
            if_not_exists: false,
            if_exists: false,
        };

        if !parser.has_remaining() {
            return Ok(command);
        }

        while parser.has_remaining() {
            let key = parser
                .next_as_string()?
                .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?
                .to_lowercase();

            if matches!(key.as_str(), "expire_at") {
                let value = parser
                    .next_as_string()?
                    .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?;
                let timestamp = value.parse::<u64>().map_err(|_| {
                    ParseCommandError::InvalidArgValue(value, key, "set".to_string())
                })?;
                match command.expire_at {
                    Some(_) => return Err(ParseCommandError::InvalidFormat),
                    None => command.expire_at = Some(timestamp),
                }
            } else if matches!(key.as_str(), "expire_after") {
                let value = parser
                    .next_as_string()?
                    .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?;

                let millis = value.parse::<u64>().map_err(|_| {
                    ParseCommandError::InvalidArgValue(value, key, "set".to_string())
                })?;

                let timestamp = SystemTime::now()
                    .add(Duration::from_millis(millis))
                    .duration_since(UNIX_EPOCH)?
                    .as_secs();

                match command.expire_at {
                    Some(_) => return Err(ParseCommandError::InvalidFormat),
                    None => command.expire_at = Some(timestamp),
                }
            } else if matches!(key.as_str(), "if_not_exists") {
                if !command.if_not_exists && !command.if_exists {
                    command.if_not_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else if matches!(key.as_str(), "if_exists") {
                if !command.if_not_exists && !command.if_exists {
                    command.if_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "set".to_string()));
            }
        }

        Ok(command)
    }

    pub fn expire_at(&self) -> Option<u64> {
        self.expire_at
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn key(&self) -> Bytes {
        self.key.clone()
    }

    pub fn value(&self) -> Bytes {
        self.value.clone()
    }

    pub fn keyspace(&self) -> Bytes {
        self.keyspace.clone()
    }
}

impl Get {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("get".to_string()))?;

        let key = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("get".to_string()))?;

        let command = Get { keyspace, key };

        if parser.has_remaining() {
            return Err(ParseCommandError::WrongArgCount("get".to_string()));
        }

        Ok(command)
    }

    pub fn keyspace(&self) -> Bytes {
        self.keyspace.clone()
    }

    pub fn key(&self) -> Bytes {
        self.key.clone()
    }
}

impl Del {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("del".to_string()))?;

        let key = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("del".to_string()))?;

        let command = Del { keyspace, key };

        if parser.has_remaining() {
            return Err(ParseCommandError::WrongArgCount("del".to_string()));
        }

        Ok(command)
    }

    pub fn keyspace(&self) -> Bytes {
        self.keyspace.clone()
    }

    pub fn key(&self) -> Bytes {
        self.key.clone()
    }
}

impl Drop {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("drop".to_string()))?;

        let mut command = Drop {
            keyspace,
            if_exists: false,
        };

        if !parser.has_remaining() {
            return Ok(command);
        }

        while parser.has_remaining() {
            let key = parser
                .next_as_string()?
                .ok_or_else(|| ParseCommandError::WrongArgCount("drop".to_string()))?
                .to_lowercase();

            if matches!(key.as_str(), "if_exists") {
                if !command.if_exists {
                    command.if_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "drop".to_string()));
            }
        }

        Ok(command)
    }

    pub fn keyspace(&self) -> Bytes {
        self.keyspace.clone()
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }
}

impl Count {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("count".to_string()))?;

        let command = Count { keyspace };

        if parser.has_remaining() {
            return Err(ParseCommandError::WrongArgCount("count".to_string()));
        }

        Ok(command)
    }

    pub fn keyspace(&self) -> Bytes {
        self.keyspace.clone()
    }
}

impl Ttl {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("ttl".to_string()))?;

        let key = parser
            .next_as_bytes()?
            .ok_or_else(|| ParseCommandError::WrongArgCount("ttl".to_string()))?;

        let command = Ttl { keyspace, key };

        if parser.has_remaining() {
            return Err(ParseCommandError::WrongArgCount("ttl".to_string()));
        }

        Ok(command)
    }

    pub fn keyspace(&self) -> Bytes {
        self.keyspace.clone()
    }

    pub fn key(&self) -> Bytes {
        self.key.clone()
    }
}

pub fn parse(frame: Frame) -> Result<Command, ParseCommandError> {
    let mut parser = Parser::new(frame)?;
    let command = match parser.next().ok_or(ParseCommandError::InvalidFormat)? {
        Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
        _ => return Err(ParseCommandError::InvalidFormat),
    };

    match command.as_str() {
        "create" => Ok(Command::Create(Create::parse(&mut parser)?)),
        "set" => Ok(Command::Set(Set::parse(&mut parser)?)),
        "get" => Ok(Command::Get(Get::parse(&mut parser)?)),
        "del" => Ok(Command::Del(Del::parse(&mut parser)?)),
        "drop" => Ok(Command::Drop(Drop::parse(&mut parser)?)),
        "count" => Ok(Command::Count(Count::parse(&mut parser)?)),
        "ttl" => Ok(Command::Ttl(Ttl::parse(&mut parser)?)),
        "ping" => Ok(Command::Ping),
        "keyspaces" => Ok(Command::Keyspaces),
        _ => Err(ParseCommandError::UnknownCommand(command)),
    }
}
