use std::io::{Error, ErrorKind::BrokenPipe};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_serial::SerialStream;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Handle {
    write_channel: UnboundedSender<String>,
    read_channel: UnboundedReceiver<String>,
}

async fn read_task(
    channel: UnboundedSender<String>,
    mut handle: ReadHalf<SerialStream>,
) -> Result<()> {
    while !channel.is_closed() {
        let mut buf = String::new();
        handle.read_to_string(&mut buf).await?;
        if !buf.is_empty() {
            match channel.send(buf) {
                Ok(_) => (),
                Err(e) => return Err(Error::new(BrokenPipe, e)),
            }
        }
    }
    Ok(())
}

async fn write_task(
    mut channel: UnboundedReceiver<String>,
    mut handle: WriteHalf<SerialStream>
) -> Result<()> {
    while let Some(msg) = channel.recv().await {
        handle.write_all(msg.as_bytes()).await?;
    }
    Ok(())
}
