use std::io::{Error, ErrorKind::BrokenPipe};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Handle {
    write_channel: UnboundedSender<String>,
    read_channel: UnboundedReceiver<String>,
}

impl Handle {
    pub fn open(port: &str, br: u32) -> Result<Self> {
        let handle = tokio_serial::new(port, br).open_native_async()?;
        let (read_half, write_half) = tokio::io::split(handle);
        let (tx_write, rx_write) = unbounded_channel();
        let (tx_read, rx_read) = unbounded_channel();
        tokio::spawn(async move { read_task(tx_read, read_half).await });
        tokio::spawn(async move { write_task(rx_write, write_half).await });
        Ok(Self {
            write_channel: tx_write,
            read_channel: rx_read,
        })
    }

    pub fn read(&mut self) -> Option<String> {
        match self.read_channel.try_recv() {
            Ok(x) -> Some(x),
            Some
        }   
    }
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
    mut handle: WriteHalf<SerialStream>,
) -> Result<()> {
    while let Some(msg) = channel.recv().await {
        handle.write_all(msg.as_bytes()).await?;
    }
    Ok(())
}
