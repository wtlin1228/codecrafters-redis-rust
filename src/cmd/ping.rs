use bytes::Bytes;

#[derive(Debug)]
pub struct Ping {
    message: Option<Bytes>,
}

impl Ping {
    pub fn new(message: Option<Bytes>) -> Self {
        Self { message }
    }
}
