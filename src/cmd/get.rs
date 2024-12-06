use crate::{connection::Connection, db::Db, frame::Frame, parse::Parse};
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn new(key: impl ToString) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn parse_frames(parse: &mut Parse) -> anyhow::Result<Get> {
        let key = parse.next_string()?;

        Ok(Get { key })
    }

    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> anyhow::Result<()> {
        let response = match db.get(&self.key) {
            None => Frame::Null,
            Some(bytes) => Frame::Bulk(bytes),
        };
        debug!(?response);
        dst.write_frame(&response).await?;
        Ok(())
    }
}
