use atoi::atoi;
use bytes::Buf;
use bytes::Bytes;
use std::io::Cursor;
use std::num::ParseFloatError;
use std::str;
use thiserror::Error;

pub const STRING_IDENT: u8 = b'$';
pub const INTEGER_IDENT: u8 = b'%';
pub const ARRAY_IDENT: u8 = b'*';
pub const BOOLEAN_IDENT: u8 = b'^';
pub const NULL_IDENT: u8 = b'-';
pub const MAP_IDENT: u8 = b'#';
pub const DOUBLE_IDENT: u8 = b'.';
pub const ERROR_IDENT: u8 = b'!';

#[derive(Debug, PartialEq)]
pub enum Frame {
    String(Bytes),
    Integer(i64),
    Array(Vec<Frame>),
    Boolean(bool),
    Null,
    Map(Vec<Frame>),
    Double(f64),
    Error(Bytes),
}

#[derive(Debug, Error)]
pub enum ParseFrameError {
    #[error("incomplete frame")]
    Incomplete,

    #[error("invalid frame format")]
    InvalidFormat,

    #[error(transparent)]
    Utf8Error(#[from] str::Utf8Error),

    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame, ParseFrameError> {
    let line = get_line(buf)?;
    if line.is_empty() {
        return Err(ParseFrameError::InvalidFormat);
    }
    let frame_type = line[0];
    let line = &line[1..];
    match frame_type {
        STRING_IDENT => parse_string(buf, line),
        INTEGER_IDENT => parse_integer(line),
        ARRAY_IDENT => parse_array(buf, line),
        BOOLEAN_IDENT => parse_boolean(line),
        NULL_IDENT => parse_null(line),
        MAP_IDENT => parse_map(buf, line),
        DOUBLE_IDENT => parse_double(line),
        ERROR_IDENT => parse_error(buf, line),
        _ => Err(ParseFrameError::InvalidFormat),
    }
}

fn get_line<'a>(buf: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], ParseFrameError> {
    if !buf.has_remaining() {
        return Err(ParseFrameError::Incomplete);
    }

    let start = buf.position() as usize;
    let end = buf.get_ref().len() - 1;

    for i in start..end {
        if buf.get_ref()[i] == b'\r' && buf.get_ref()[i + 1] == b'\n' {
            buf.set_position((i + 2) as u64);
            return Ok(&buf.get_ref()[start..i]);
        }
    }

    Err(ParseFrameError::Incomplete)
}

fn skip(buf: &mut Cursor<&[u8]>, n: usize) -> Result<(), ParseFrameError> {
    if buf.remaining() < n {
        return Err(ParseFrameError::Incomplete);
    }
    buf.advance(n);
    Ok(())
}

fn parse_string(buf: &mut Cursor<&[u8]>, line: &[u8]) -> Result<Frame, ParseFrameError> {
    let len = atoi::<usize>(line).ok_or(ParseFrameError::InvalidFormat)?;
    let n = len + 2;

    if buf.remaining() < n {
        return Err(ParseFrameError::Incomplete);
    }

    let data = Bytes::copy_from_slice(&buf.chunk()[..len]);

    skip(buf, n)?;

    Ok(Frame::String(data))
}

fn parse_integer(line: &[u8]) -> Result<Frame, ParseFrameError> {
    let int = atoi::<i64>(line).ok_or(ParseFrameError::InvalidFormat)?;
    Ok(Frame::Integer(int))
}

fn parse_array(buf: &mut Cursor<&[u8]>, line: &[u8]) -> Result<Frame, ParseFrameError> {
    let len = atoi::<usize>(line).ok_or(ParseFrameError::InvalidFormat)?;
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(parse(buf)?);
    }

    Ok(Frame::Array(vec))
}

fn parse_boolean(line: &[u8]) -> Result<Frame, ParseFrameError> {
    if line.len() > 1 {
        return Err(ParseFrameError::InvalidFormat);
    }

    let val = line[0];

    match val {
        b'0' => Ok(Frame::Boolean(false)),
        b'1' => Ok(Frame::Boolean(true)),
        _ => Err(ParseFrameError::InvalidFormat),
    }
}

fn parse_null(line: &[u8]) -> Result<Frame, ParseFrameError> {
    if !line.is_empty() {
        return Err(ParseFrameError::InvalidFormat);
    }
    Ok(Frame::Null)
}

fn parse_map(buf: &mut Cursor<&[u8]>, line: &[u8]) -> Result<Frame, ParseFrameError> {
    let len = atoi::<usize>(line).ok_or(ParseFrameError::InvalidFormat)?;
    let mut map = Vec::with_capacity(2 * len);
    for _ in 0..len {
        let key = parse(buf)?;
        let value = parse(buf)?;
        map.push(key);
        map.push(value);
    }

    Ok(Frame::Map(map))
}

fn parse_double(line: &[u8]) -> Result<Frame, ParseFrameError> {
    let double = str::from_utf8(line)?.parse::<f64>()?;
    Ok(Frame::Double(double))
}

fn parse_error(buf: &mut Cursor<&[u8]>, line: &[u8]) -> Result<Frame, ParseFrameError> {
    let len = atoi::<usize>(line).ok_or(ParseFrameError::InvalidFormat)?;
    let n = len + 2;

    if buf.remaining() < n {
        return Err(ParseFrameError::Incomplete);
    }

    let data = Bytes::copy_from_slice(&buf.chunk()[..len]);

    skip(buf, n)?;

    Ok(Frame::Error(data))
}
