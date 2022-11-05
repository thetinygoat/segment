use crate::frame::{
    self, Frame, ParseFrameError, ARRAY_IDENT, BOOLEAN_IDENT, DOUBLE_IDENT, ERROR_IDENT,
    INTEGER_IDENT, MAP_IDENT, STRING_IDENT,
};
use async_recursion::async_recursion;
use bytes::{Buf, Bytes, BytesMut};
use std::io::{self, Cursor};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub struct Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    stream: T,
    buf: BytesMut,
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("connection reset by peer")]
    Reset,

    #[error(transparent)]
    Frame(#[from] ParseFrameError),

    #[error("malformed frame received for write")]
    MalformedFrameForWrite,
}

impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub fn new(stream: T, buf_size: usize) -> Self {
        Connection {
            stream,
            buf: BytesMut::with_capacity(buf_size),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, ConnectionError> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if self.stream.read_buf(&mut self.buf).await? == 0 {
                if self.buf.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ConnectionError::Reset);
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, ConnectionError> {
        let mut cursor = Cursor::new(&self.buf[..]);
        match frame::parse(&mut cursor) {
            Ok(frame) => {
                self.buf.advance(cursor.position() as usize);
                Ok(Some(frame))
            }
            Err(ParseFrameError::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[async_recursion]
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), ConnectionError> {
        match frame {
            Frame::String(data) => {
                let len = data.len();
                self.stream.write_u8(STRING_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", len).as_bytes())
                    .await?;
                self.stream.write_all(data).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(data) => {
                self.stream.write_u8(INTEGER_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", data).as_bytes())
                    .await?;
            }
            Frame::Boolean(data) => {
                self.stream.write_u8(BOOLEAN_IDENT).await?;
                if *data {
                    self.stream
                        .write_all(format!("{}\r\n", 1).as_bytes())
                        .await?;
                } else {
                    self.stream
                        .write_all(format!("{}\r\n", 0).as_bytes())
                        .await?;
                }
            }
            Frame::Null => {
                self.stream.write_all(b"-\r\n").await?;
            }
            Frame::Double(data) => {
                self.stream.write_u8(DOUBLE_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", data).as_bytes())
                    .await?;
            }
            Frame::Error(data) => {
                let len = data.len();
                self.stream.write_u8(ERROR_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", len).as_bytes())
                    .await?;
                self.stream.write_all(data).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(array) => {
                self.stream.write_u8(ARRAY_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", array.len()).as_bytes())
                    .await?;
                for value in array {
                    self.write_frame(value).await?;
                }
            }
            Frame::Map(map) => {
                if map.len() % 2 != 0 {
                    return Err(ConnectionError::MalformedFrameForWrite);
                }
                self.stream.write_u8(MAP_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", map.len() / 2).as_bytes())
                    .await?;
                for value in map {
                    self.write_frame(value).await?;
                }
            }
        }

        self.stream.flush().await?;
        Ok(())
    }

    pub async fn write_error(
        &mut self,
        error: impl std::error::Error,
    ) -> Result<(), ConnectionError> {
        self.write_frame(&Frame::Error(Bytes::from(error.to_string())))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tokio_test::io::Builder;

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

    #[tokio::test]
    async fn write_frame_given_string_writes_string_frame() {
        let mock = Builder::new().write(b"$3\r\nfoo\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::String(Bytes::from("foo")))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_string_with_crlf_writes_string_frame() {
        let mock = Builder::new().write(b"$5\r\nfoo\r\n\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::String(Bytes::from("foo\r\n")))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_string_with_pdf_data_writes_string_frame() {
        let file_data = read_file(Path::new("test_data").join("test.pdf"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), STRING_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::String(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_string_with_png_data_writes_string_frame() {
        let file_data = read_file(Path::new("test_data").join("test.png"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), STRING_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::String(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_string_with_jpg_data_writes_string_frame() {
        let file_data = read_file(Path::new("test_data").join("test.jpg"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), STRING_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::String(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_string_with_html_data_writes_string_frame() {
        let file_data = read_file(Path::new("test_data").join("test.html"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), STRING_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::String(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_positive_integer_writes_integer_frame() {
        let mock = Builder::new().write(b"%100\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Integer(100)).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_negative_integer_writes_integer_frame() {
        let mock = Builder::new().write(b"%-100\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Integer(-100)).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_zero_writes_integer_frame() {
        let mock = Builder::new().write(b"%0\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Integer(0)).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_false_writes_boolean_frame() {
        let mock = Builder::new().write(b"^0\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Boolean(false))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_true_writes_boolean_frame() {
        let mock = Builder::new().write(b"^1\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Boolean(true)).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_null_writes_null_frame() {
        let mock = Builder::new().write(b"-\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Null).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_positive_double_writes_double_frame() {
        let mock = Builder::new().write(b".100\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Double(100.0000))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_negative_double_writes_double_frame() {
        let mock = Builder::new().write(b".-100\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Double(-100.0))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_zero_writes_double_frame() {
        let mock = Builder::new().write(b".0\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Double(0.0)).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_positive_double_with_decimal_part_writes_double_frame() {
        let mock = Builder::new().write(b".100.00001\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Double(100.00001))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_negative_double_with_decimal_part_writes_double_frame() {
        let mock = Builder::new().write(b".-100.6789\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Double(-100.6789))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_zero_with_decimal_part_writes_double_frame() {
        let mock = Builder::new().write(b".0.1\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Double(0.10)).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_error_writes_error_frame() {
        let mock = Builder::new().write(b"!3\r\nfoo\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Error(Bytes::from("foo")))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_error_with_crlf_writes_error_frame() {
        let mock = Builder::new().write(b"!5\r\nfoo\r\n\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Error(Bytes::from("foo\r\n")))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_error_with_pdf_data_writes_error_frame() {
        let file_data = read_file(Path::new("test_data").join("test.pdf"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), ERROR_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Error(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_error_with_png_data_writes_error_frame() {
        let file_data = read_file(Path::new("test_data").join("test.png"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), ERROR_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Error(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_error_with_jpg_data_writes_error_frame() {
        let file_data = read_file(Path::new("test_data").join("test.jpg"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), ERROR_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Error(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_error_with_html_data_writes_error_frame() {
        let file_data = read_file(Path::new("test_data").join("test.html"));
        let mock = Builder::new()
            .write(&get_frame_from_file(file_data.as_slice(), ERROR_IDENT))
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Error(Bytes::from(file_data)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_empty_array_writes_array_frame() {
        let mock = Builder::new().write(b"*0\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Array(vec![])).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_array_with_one_element_writes_array_frame() {
        let mock = Builder::new().write(b"*1\r\n$3\r\nfoo\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Array(vec![Frame::String(Bytes::from("foo"))]))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_empty_map_writes_map_frame() {
        let mock = Builder::new().write(b"#0\r\n").build();
        let mut connection = Connection::new(mock, 1024);
        connection.write_frame(&Frame::Map(vec![])).await.unwrap();
    }

    #[tokio::test]
    async fn write_frame_given_malformed_map_returns_malformed_frame_for_write_error() {
        let mock = Builder::new().build();
        let mut connection = Connection::new(mock, 1024);
        match connection
            .write_frame(&Frame::Map(vec![Frame::String(Bytes::from("foo"))]))
            .await
        {
            Err(ConnectionError::MalformedFrameForWrite) => {}
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn write_frame_given_map_with_one_element_writes_map_frame() {
        let mock = Builder::new()
            .write(b"#1\r\n$3\r\nfoo\r\n$3\r\nbar\r\n")
            .build();
        let mut connection = Connection::new(mock, 1024);
        connection
            .write_frame(&Frame::Map(vec![
                Frame::String(Bytes::from("foo")),
                Frame::String(Bytes::from("bar")),
            ]))
            .await
            .unwrap();
    }
}
