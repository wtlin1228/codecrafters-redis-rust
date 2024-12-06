use crate::cmd::Command;
use crate::connection::Connection;
use crate::db::Db;
use std::future::Future;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{debug, error, info, instrument};

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);

    let mut server = Listener {
        listener,
        notify_shutdown,
        db: Db::new(),
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = ?err, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }
}

#[derive(Debug)]
struct Shutdown {
    is_shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    pub fn new(notify: broadcast::Receiver<()>) -> Self {
        Self {
            is_shutdown: false,
            notify,
        }
    }

    async fn recv(&mut self) {
        if self.is_shutdown {
            return;
        }

        let _ = self.notify.recv().await;

        self.is_shutdown = true;
    }
}

#[derive(Debug)]
struct Handler {
    connection: Connection,
    shutdown: Shutdown,
    db: Db,
}

impl Handler {
    #[instrument(skip(self))]
    async fn run(&mut self) -> anyhow::Result<()> {
        while !self.shutdown.is_shutdown {
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };

            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };

            let cmd = Command::from_frame(frame)?;

            debug!(?cmd);

            cmd.apply(&self.db, &mut self.connection).await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    db: Db,
}

impl Listener {
    async fn run(&mut self) -> anyhow::Result<()> {
        info!("accepting inbound connections");

        loop {
            let (socket, _) = self.listener.accept().await?;

            let mut handler = Handler {
                connection: Connection::new(socket),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                db: self.db.clone(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
            });
        }
    }
}
