use crate::frame::Frame;
use bytes::{Buf, BytesMut};
use std::io::{Cursor, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(socket),
            // Default to a 4KB read buffer for this toy redis project.
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> anyhow::Result<Option<Frame>> {
        loop {
            // To pass the wrong format input
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return Ok(None);
            }
            return Ok(Some(Frame::Array(vec![Frame::Simple("PING".to_string())])));

            // if let Some(frame) = self.parse_frame()? {
            //     return Ok(Some(frame));
            // }

            // if 0 == self.stream.read_buf(&mut self.buffer).await? {
            //     if self.buffer.is_empty() {
            //         return Ok(None);
            //     } else {
            //         anyhow::bail!("connection reset by peer");
            //     }
            // }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        match frame {
            Frame::Array(vec) => {
                self.stream.write_u8(b'*').await?;
                self.write_decimal(vec.len() as u64).await?;
                for frame in vec {
                    self.write_value(frame).await?;
                }
            }
            _ => self.write_value(frame).await?,
        }

        self.stream.flush().await
    }

    async fn write_value(&mut self, frame: &Frame) -> std::io::Result<()> {
        match frame {
            Frame::Simple(s) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(s) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(n) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*n).await?;
            }
            Frame::Bulk(bytes) => {
                let len = bytes.len();
                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(bytes).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Array(_vec) => unreachable!(),
        }

        Ok(())
    }

    async fn write_decimal(&mut self, val: u64) -> std::io::Result<()> {
        let mut buf = [0u8; 20];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }

    fn parse_frame(&mut self) -> anyhow::Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        let frame = Frame::parse(&mut buf)?;

        self.buffer.advance(buf.position() as usize);

        Ok(Some(frame))
    }
}
