mod ping;
use ping::Ping;

mod get;
use get::Get;

mod set;
use set::Set;

use crate::connection::Connection;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Get(Get),
    Set(Set),
}

impl Command {
    pub fn from_frame(frame: Frame) -> anyhow::Result<Self> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::parse_frames(&mut parse)?),
            "echo" => Command::Ping(Ping::parse_frames(&mut parse)?),
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            _ => unimplemented!(),
        };

        Ok(command)
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection) -> anyhow::Result<()> {
        match self {
            Command::Ping(cmd) => cmd.apply(dst).await,
            Command::Get(cmd) => cmd.apply(db, dst).await,
            Command::Set(cmd) => cmd.apply(db, dst).await,
        }
    }
}
