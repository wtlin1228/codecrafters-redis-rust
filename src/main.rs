use redis_starter_rust::server;
use tokio::net::TcpListener;
use tokio::signal;

fn set_up_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_up_logging();

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
