use atoi::atoi;
use bytes::Buf;
use bytes::Bytes;
use std::io::Cursor;
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

#[derive(Debug, Error, PartialEq)]
pub enum ParseFrameError {
    #[error("failed to parse frame, more data is required")]
    Incomplete,

    #[error("failed to parse frame, invalid frame format")]
    InvalidFormat,
}

pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame, ParseFrameError> {
    // since our frames are CRLF delimited, we read our frames line by line.
    // A line here represents a CRLF delimited section of frame. This is binary
    // safe because when reading bytes which might contain binary data, we
    // don't look for a delimiter, instead of depend on the prefixed length.
    // This is safe to do for other data types like, booleans, null, integers, doubles etc.
    let line = get_line(buf)?;
    if line.is_empty() {
        return Err(ParseFrameError::InvalidFormat);
    }
    // first byte of a line determines it's type
    let frame_type = line[0];
    // rest of the line can contain the length of the data (in case of strings and errors),
    // or the whole data in case of other data types
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
        // A line is CRLF terminated and we look for that
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
    // len is the length of encoded data
    let len = atoi::<usize>(line).ok_or(ParseFrameError::InvalidFormat)?;
    // add 2 to accommodate CRLF
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
    if len % 2 != 0 {
        return Err(ParseFrameError::InvalidFormat);
    }
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
    let double = str::from_utf8(line)
        .map_err(|_| ParseFrameError::InvalidFormat)?
        .parse::<f64>()
        .map_err(|_| ParseFrameError::InvalidFormat)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};
    use std::fs;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};

    fn get_cursor_from_bytes(bytes: &[u8]) -> Cursor<&[u8]> {
        Cursor::new(bytes)
    }

    fn read_file(path: PathBuf) -> Vec<u8> {
        fs::read(path).unwrap()
    }

    fn get_frame_from_file(data: &[u8], ident: u8) -> Bytes {
        let mut frame = BytesMut::new();
        frame.put_u8(ident);
        frame.put_slice(format!("{}", data.len()).as_bytes());
        frame.put_u8(b'\r');
        frame.put_u8(b'\n');
        frame.put(data);
        frame.put_u8(b'\r');
        frame.put_u8(b'\n');

        frame.copy_to_bytes(frame.len())
    }

    #[test]
    fn parse_given_empty_line_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_unknown_type_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"foo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_string_with_no_length_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"$\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_string_with_invalid_length_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"$abc\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_incomplete_string_with_zero_length_returns_incomplete_error() {
        let mut buf = get_cursor_from_bytes(b"$0\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::Incomplete))
    }

    #[test]
    fn parse_given_incomplete_string_with_non_zero_length_returns_incomplete_error() {
        let mut buf = get_cursor_from_bytes(b"$1\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::Incomplete))
    }

    #[test]
    fn parse_given_string_with_zero_length_returns_empty_string() {
        let mut buf = get_cursor_from_bytes(b"$0\r\n\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from(""))))
    }

    #[test]
    fn parse_given_string_with_length_less_than_length_of_data_returns_data_upto_given_length() {
        let mut buf = get_cursor_from_bytes(b"$1\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from("f"))))
    }

    #[test]
    fn parse_given_string_with_length_greater_than_length_of_data_returns_incomplete_error() {
        let mut buf = get_cursor_from_bytes(b"$100\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::Incomplete))
    }

    #[test]
    fn parse_given_string_returns_string() {
        let mut buf = get_cursor_from_bytes(b"$3\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from("foo"))))
    }

    #[test]
    fn parse_given_string_with_delimiter_in_data_returns_string() {
        let mut buf = get_cursor_from_bytes(b"$5\r\nfoo\r\n\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from("foo\r\n"))))
    }

    #[test]
    fn parse_given_string_with_pdf_data_returns_pdf_data() {
        let file_data = read_file(Path::new("test_data").join("test.pdf"));
        let frame = get_frame_from_file(file_data.as_slice(), STRING_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_string_with_png_data_returns_png_data() {
        let file_data = read_file(Path::new("test_data").join("test.png"));
        let frame = get_frame_from_file(file_data.as_slice(), STRING_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_string_with_jpg_data_returns_jpg_data() {
        let file_data = read_file(Path::new("test_data").join("test.jpg"));
        let frame = get_frame_from_file(file_data.as_slice(), STRING_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_string_with_html_data_returns_html_data() {
        let file_data = read_file(Path::new("test_data").join("test.html"));
        let frame = get_frame_from_file(file_data.as_slice(), STRING_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::String(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_invalid_integer_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"%abc\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_empty_integer_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"%\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_negative_integer_returns_given_integer() {
        let mut buf = get_cursor_from_bytes(b"%-1\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Integer(-1)))
    }

    #[test]
    fn parse_given_zero_returns_zero() {
        let mut buf = get_cursor_from_bytes(b"%0\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Integer(0)))
    }

    #[test]
    fn parse_given_positive_integer_returns_given_integer() {
        let mut buf = get_cursor_from_bytes(b"%1000\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Integer(1000)))
    }

    #[test]
    fn parse_given_out_of_range_integer_returns_format_error() {
        let mut buf = get_cursor_from_bytes(b"%9223372036854775808\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_false_returns_false() {
        let mut buf = get_cursor_from_bytes(b"^0\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Boolean(false)))
    }

    #[test]
    fn parse_given_true_returns_true() {
        let mut buf = get_cursor_from_bytes(b"^1\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Boolean(true)))
    }

    #[test]
    fn parse_given_invalid_boolean_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"^foo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_null_returns_null() {
        let mut buf = get_cursor_from_bytes(b"-\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Null))
    }

    #[test]
    fn parse_given_invalid_null_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"-foo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_double_with_invalid_decimal_part_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b".20.foo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_double_with_invalid_integer_part_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b".foo.90\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_double_with_zero_decimal_part_returns_double() {
        let mut buf = get_cursor_from_bytes(b".10.000\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Double(10.0)))
    }

    #[test]
    fn parse_given_double_with_trailing_zeroes_returns_double() {
        let mut buf = get_cursor_from_bytes(b".10.100\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Double(10.1)))
    }

    #[test]
    fn parse_given_double_with_leading_zeroes_returns_double() {
        let mut buf = get_cursor_from_bytes(b".000010.100\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Double(10.1)))
    }

    #[test]
    fn parse_given_invalid_double_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b".abc\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_error_with_no_length_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"!\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_error_with_invalid_length_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"!abc\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_incomplete_error_with_zero_length_returns_incomplete_error() {
        let mut buf = get_cursor_from_bytes(b"!0\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::Incomplete))
    }

    #[test]
    fn parse_given_incomplete_error_with_non_zero_length_returns_incomplete_error() {
        let mut buf = get_cursor_from_bytes(b"!1\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::Incomplete))
    }

    #[test]
    fn parse_given_error_with_zero_length_returns_empty_error_frame() {
        let mut buf = get_cursor_from_bytes(b"!0\r\n\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from(""))))
    }

    #[test]
    fn parse_given_error_with_length_less_than_length_of_data_returns_data_upto_given_length() {
        let mut buf = get_cursor_from_bytes(b"!1\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from("f"))))
    }

    #[test]
    fn parse_given_error_with_length_greater_than_length_of_data_returns_incomplete_error() {
        let mut buf = get_cursor_from_bytes(b"!100\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::Incomplete))
    }

    #[test]
    fn parse_given_error_returns_error_frame() {
        let mut buf = get_cursor_from_bytes(b"!3\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from("foo"))))
    }

    #[test]
    fn parse_given_error_with_delimiter_in_data_returns_error_frame() {
        let mut buf = get_cursor_from_bytes(b"!5\r\nfoo\r\n\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from("foo\r\n"))))
    }

    #[test]
    fn parse_given_error_with_pdf_data_returns_pdf_data() {
        let file_data = read_file(Path::new("test_data").join("test.pdf"));
        let frame = get_frame_from_file(file_data.as_slice(), ERROR_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_error_with_png_data_returns_png_data() {
        let file_data = read_file(Path::new("test_data").join("test.png"));
        let frame = get_frame_from_file(file_data.as_slice(), ERROR_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_error_with_jpg_data_returns_jpg_data() {
        let file_data = read_file(Path::new("test_data").join("test.jpg"));
        let frame = get_frame_from_file(file_data.as_slice(), ERROR_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_error_with_html_data_returns_html_data() {
        let file_data = read_file(Path::new("test_data").join("test.html"));
        let frame = get_frame_from_file(file_data.as_slice(), ERROR_IDENT);
        let mut buf = get_cursor_from_bytes(&frame);
        assert_eq!(parse(&mut buf), Ok(Frame::Error(Bytes::from(file_data))))
    }

    #[test]
    fn parse_given_array_with_no_length_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"*\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_array_with_zero_length_returns_empty_array() {
        let mut buf = get_cursor_from_bytes(b"*0\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Array(Vec::new())))
    }

    #[test]
    fn parse_given_array_with_invalid_length_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"*abc\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_map_with_no_length_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"#\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_map_with_zero_length_returns_empty_map() {
        let mut buf = get_cursor_from_bytes(b"#0\r\n");
        assert_eq!(parse(&mut buf), Ok(Frame::Map(Vec::new())))
    }

    #[test]
    fn parse_given_map_with_invalid_length_returns_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"#abc\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_map_with_odd_length_return_invalid_format_error() {
        let mut buf = get_cursor_from_bytes(b"#1\r\n$3\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::InvalidFormat))
    }

    #[test]
    fn parse_given_incomplete_map_return_incomplete_error() {
        let mut buf = get_cursor_from_bytes(b"#2\r\n$3\r\nfoo\r\n");
        assert_eq!(parse(&mut buf), Err(ParseFrameError::Incomplete))
    }
}
