use crate::frame::{
    self, Frame, ParseFrameError, ARRAY_IDENT, BOOLEAN_IDENT, DOUBLE_IDENT, ERROR_IDENT,
    INTEGER_IDENT, MAP_IDENT, STRING_IDENT,
};
use bytes::{Buf, Bytes, BytesMut};
use std::io::{self, Cursor};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
    buf: BytesMut,
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error(transparent)]
    TCPError(#[from] io::Error),

    #[error("connection reset by peer")]
    Reset,

    #[error(transparent)]
    FrameError(#[from] ParseFrameError),
}

impl Connection {
    pub fn new(stream: TcpStream, buf_size: usize) -> Self {
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

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), ConnectionError> {
        match frame {
            Frame::Array(array) => {
                self.stream.write_u8(ARRAY_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", array.len()).as_bytes())
                    .await?;
                for value in array {
                    self.write_value(value).await?;
                }
            }
            Frame::Map(map) => {
                self.stream.write_u8(MAP_IDENT).await?;
                self.stream
                    .write_all(format!("{}\r\n", map.len() / 2).as_bytes())
                    .await?;
                for value in map {
                    self.write_value(value).await?;
                }
            }
            _ => self.write_value(frame).await?,
        }

        self.stream.flush().await?;
        Ok(())
    }

    async fn write_value(&mut self, frame: &Frame) -> Result<(), ConnectionError> {
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
            _ => unreachable!(),
        }

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
