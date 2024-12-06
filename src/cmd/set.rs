use crate::{connection::Connection, db::Db, frame::Frame, parse::Parse};
use bytes::Bytes;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
}

impl Set {
    pub fn new(key: impl ToString, value: Bytes) -> Self {
        Self {
            key: key.to_string(),
            value,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Bytes {
        &self.value
    }

    pub fn parse_frames(parse: &mut Parse) -> anyhow::Result<Set> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;

        Ok(Set { key, value })
    }

    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> anyhow::Result<()> {
        db.set(self.key, self.value);
        let response = Frame::Simple("OK".to_string());
        debug!(?response);
        dst.write_frame(&response).await?;
        Ok(())
    }
}
