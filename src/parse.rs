use anyhow::Context;

use crate::frame::Frame;
use core::str;
use std::vec;

#[derive(Debug)]
pub struct Parse {
    /// Array frame iterator.
    parts: vec::IntoIter<Frame>,
}

impl Parse {
    pub fn new(frame: Frame) -> anyhow::Result<Self> {
        match frame {
            Frame::Array(vec) => Ok(Self {
                parts: vec.into_iter(),
            }),
            _ => anyhow::bail!("protocol error; expected array, got {:?}", frame),
        }
    }

    fn next(&mut self) -> anyhow::Result<Frame> {
        self.parts.next().context("end of stream")
    }

    pub fn next_string(&mut self) -> anyhow::Result<String> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(bytes) => str::from_utf8(&bytes[..])
                .map(|s| s.to_string())
                .context("protocol error; invalid string"),
            frame => anyhow::bail!(
                "protocal error; expected simple frame or bulk frame, got {:?}",
                frame
            ),
        }
    }
}
