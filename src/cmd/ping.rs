use crate::connection::Connection;
use crate::frame::Frame;
use crate::parse::{Parse, ParseError};
use bytes::Bytes;
use tracing::{debug, instrument};

#[derive(Debug, Default)]
pub struct Ping {
    message: Option<Bytes>,
}

impl Ping {
    pub fn new(message: Option<Bytes>) -> Self {
        Self { message }
    }

    pub fn parse_frames(parse: &mut Parse) -> anyhow::Result<Ping> {
        match parse.next_bytes() {
            Ok(msg) => Ok(Ping::new(Some(msg))),
            Err(ParseError::EndOfStreamError) => Ok(Ping::default()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(skip(self, dst))]
    pub async fn apply(self, dst: &mut Connection) -> anyhow::Result<()> {
        let response = match self.message {
            None => Frame::Simple("PONG".to_string()),
            Some(bytes) => Frame::Bulk(bytes),
        };
        debug!(?response);
        dst.write_frame(&response).await?;
        Ok(())
    }
}
