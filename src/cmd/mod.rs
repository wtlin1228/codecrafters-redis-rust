mod ping;
use ping::Ping;

use crate::connection::Connection;
use crate::frame::Frame;
use crate::parse::Parse;

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
}

impl Command {
    pub fn from_frame(frame: Frame) -> anyhow::Result<Self> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::parse_frames(&mut parse)?),
            "echo" => Command::Ping(Ping::parse_frames(&mut parse)?),
            _ => unimplemented!(),
        };

        Ok(command)
    }

    pub async fn apply(self, dst: &mut Connection) -> anyhow::Result<()> {
        match self {
            Command::Ping(cmd) => cmd.apply(dst).await,
        }
    }
}
