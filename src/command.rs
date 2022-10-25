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
}

impl Create {
    fn parse(parser: &mut Parser) -> Result<Self, ParseCommandError> {
        let keyspace = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("create".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        let mut command = Create {
            keyspace,
            evictor: Evictor::Nop,
            if_not_exists: false,
        };

        if !parser.has_remaining() {
            return Ok(command);
        };

        let evictor = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("create".to_string()))?
        {
            Frame::Map(map) => Self::parse_args(map)?,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        command.evictor = evictor;

        if !parser.has_remaining() {
            return Ok(command);
        }

        let if_not_exists = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("create".to_string()))?
        {
            Frame::Array(array) => Self::parse_flags(array)?,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        command.if_not_exists = if_not_exists;

        if parser.has_remaining() {
            return Err(ParseCommandError::WrongArgCount("create".to_string()));
        }

        Ok(command)
    }

    fn parse_args(args: Vec<Frame>) -> Result<Evictor, ParseCommandError> {
        let mut idx = 0;
        let mut evictor = Evictor::Nop;
        while idx < args.len() {
            let key = match &args[idx] {
                Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
                _ => return Err(ParseCommandError::InvalidFormat),
            };
            idx += 1;
            let value = match &args[idx] {
                Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
                _ => return Err(ParseCommandError::InvalidFormat),
            };
            idx += 1;

            if matches!(key.as_str(), "evictor") {
                if matches!(value.as_str(), "nop" | "random" | "lru") {
                    match value.as_str() {
                        "nop" => evictor = Evictor::Nop,
                        "random" => evictor = Evictor::Random,
                        "lru" => evictor = Evictor::Lru,
                        _ => unreachable!(),
                    }
                } else {
                    return Err(ParseCommandError::InvalidArgValue(
                        value,
                        key,
                        "create".to_string(),
                    ));
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "create".to_string()));
            }
        }

        Ok(evictor)
    }

    fn parse_flags(flags: Vec<Frame>) -> Result<bool, ParseCommandError> {
        let mut if_not_exists = false;

        for flag in flags {
            let key = match &flag {
                Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
                _ => return Err(ParseCommandError::InvalidFormat),
            };
            if matches!(key.as_str(), "if_not_exists") {
                if !if_not_exists {
                    if_not_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "create".to_string()));
            }
        }

        Ok(if_not_exists)
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
        let keyspace = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        let key = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        let value = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

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

        let (expire_at, expire_after) = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?
        {
            Frame::Map(map) => Self::parse_args(map)?,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        if expire_at.is_some() && expire_after.is_some() {
            return Err(ParseCommandError::InvalidFormat);
        }

        if expire_at.is_some() {
            command.expire_at = expire_at;
        } else if let Some(duration) = expire_after {
            command.expire_at = Some(
                SystemTime::now()
                    .add(Duration::from_millis(duration))
                    .duration_since(UNIX_EPOCH)?
                    .as_secs(),
            );
        }

        if !parser.has_remaining() {
            return Ok(command);
        }

        let (if_exists, if_not_exists) = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("set".to_string()))?
        {
            Frame::Array(array) => Self::parse_flags(array)?,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        if if_exists && if_not_exists {
            return Err(ParseCommandError::InvalidFormat);
        }

        if if_exists {
            command.if_exists = if_exists
        } else if if_not_exists {
            command.if_not_exists = if_not_exists
        }

        if parser.has_remaining() {
            return Err(ParseCommandError::WrongArgCount("set".to_string()));
        }

        Ok(command)
    }

    fn parse_args(args: Vec<Frame>) -> Result<(Option<u64>, Option<u64>), ParseCommandError> {
        let mut idx = 0;
        let mut expire_at: Option<u64> = None;
        let mut expire_after: Option<u64> = None;
        while idx < args.len() {
            let key = match &args[idx] {
                Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
                _ => return Err(ParseCommandError::InvalidFormat),
            };
            idx += 1;
            let value = match &args[idx] {
                Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
                _ => return Err(ParseCommandError::InvalidFormat),
            };
            idx += 1;

            if matches!(key.as_str(), "expire_at") {
                if expire_at.is_none() {
                    expire_at = Some(value.parse::<u64>().map_err(|_| {
                        ParseCommandError::InvalidArgValue(value, key, "set".to_string())
                    })?);
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else if matches!(key.as_str(), "expire_after") {
                if expire_after.is_none() {
                    expire_after = Some(value.parse::<u64>().map_err(|_| {
                        ParseCommandError::InvalidArgValue(value, key, "set".to_string())
                    })?);
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "set".to_string()));
            }
        }

        Ok((expire_at, expire_after))
    }

    fn parse_flags(flags: Vec<Frame>) -> Result<(bool, bool), ParseCommandError> {
        let mut if_exists = false;
        let mut if_not_exists = false;

        for flag in flags {
            let key = match &flag {
                Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
                _ => return Err(ParseCommandError::InvalidFormat),
            };

            if matches!(key.as_str(), "if_exists") {
                if !if_exists {
                    if_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else if matches!(key.as_str(), "if_not_exists") {
                if !if_not_exists {
                    if_not_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "set".to_string()));
            }
        }

        Ok((if_exists, if_not_exists))
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
        let keyspace = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("get".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        let key = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("get".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };
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
        let keyspace = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("del".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        let key = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("del".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };
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
        let keyspace = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("drop".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        let mut command = Drop {
            keyspace,
            if_exists: false,
        };

        if !parser.has_remaining() {
            return Ok(command);
        }

        let if_exists = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("drop".to_string()))?
        {
            Frame::Array(array) => Self::parse_flags(array)?,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        command.if_exists = if_exists;

        if parser.has_remaining() {
            return Err(ParseCommandError::WrongArgCount("drop".to_string()));
        }

        Ok(command)
    }

    fn parse_flags(flags: Vec<Frame>) -> Result<bool, ParseCommandError> {
        let mut if_exists = false;

        for flag in flags {
            let key = match &flag {
                Frame::String(data) => str::from_utf8(&data[..])?.to_lowercase(),
                _ => return Err(ParseCommandError::InvalidFormat),
            };
            if matches!(key.as_str(), "if_exists") {
                if !if_exists {
                    if_exists = true
                } else {
                    return Err(ParseCommandError::InvalidFormat);
                }
            } else {
                return Err(ParseCommandError::InvalidArg(key, "create".to_string()));
            }
        }

        Ok(if_exists)
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
        let keyspace = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("count".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

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
        let keyspace = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("ttl".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };

        let key = match parser
            .next()
            .ok_or_else(|| ParseCommandError::WrongArgCount("ttl".to_string()))?
        {
            Frame::String(data) => data,
            _ => return Err(ParseCommandError::InvalidFormat),
        };
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
