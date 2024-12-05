use anyhow::Context;
use atoi::atoi;
use bytes::{Buf, Bytes};
use std::io::Cursor;

#[derive(Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    pub fn parse(src: &mut Cursor<&[u8]>) -> anyhow::Result<Frame> {
        match get_u8(src)? {
            b'$' => {
                let len = get_decimal(src)?.try_into()?;
                let n = len + 2;

                if src.remaining() < n {
                    anyhow::bail!("incomplete")
                }

                let data = Bytes::copy_from_slice(&src.chunk()[..len]);

                skip(src, n)?;

                Ok(Frame::Bulk(data))
            }
            b'*' => {
                let len = get_decimal(src)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(out))
            }
            _ => unimplemented!(),
        }
    }
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> anyhow::Result<()> {
    if !src.remaining() < n {
        anyhow::bail!("incomplete")
    }

    src.advance(n);
    Ok(())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> anyhow::Result<u8> {
    if !src.has_remaining() {
        anyhow::bail!("incomplete")
    }

    Ok(src.get_u8())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> anyhow::Result<&'a [u8]> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    anyhow::bail!("incomplete")
}

fn get_decimal(src: &mut Cursor<&[u8]>) -> anyhow::Result<u64> {
    let line = get_line(src)?;
    atoi::<u64>(line).context("protocol error; invalid frame format")
}
