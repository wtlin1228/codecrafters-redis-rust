use crate::frame::Frame;
use bytes::Bytes;
use core::str;
use std::vec;
use thiserror::Error;

#[derive(Debug)]
pub struct Parse {
    /// Array frame iterator.
    parts: vec::IntoIter<Frame>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("end of stream error")]
    EndOfStreamError,

    #[error("protocal error; ${0}")]
    ProtocalError(String),
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

    fn next(&mut self) -> anyhow::Result<Frame, ParseError> {
        self.parts.next().ok_or(ParseError::EndOfStreamError)
    }

    pub fn next_string(&mut self) -> anyhow::Result<String, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(bytes) => str::from_utf8(&bytes[..])
                .map(|s| s.to_string())
                .map_err(|_| ParseError::ProtocalError("invalid string".to_string())),
            frame => Err(ParseError::ProtocalError(format!(
                "expected simple frame or bulk frame, got {:?}",
                frame
            ))),
        }
    }

    pub(crate) fn next_bytes(&mut self) -> anyhow::Result<Bytes, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Frame::Bulk(bytes) => Ok(bytes),
            frame => Err(ParseError::ProtocalError(format!(
                "expected simple frame or bulk frame, got {:?}",
                frame
            ))),
        }
    }
}
