mod ping;
use crate::{frame::Frame, parse::Parse};
use ping::Ping;

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
}

impl Command {
    pub fn from_frame(frame: Frame) -> anyhow::Result<Self> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::new(None)),
            _ => unimplemented!(),
        };

        Ok(command)
    }
}
