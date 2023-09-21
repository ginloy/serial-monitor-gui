use log::*;
use std::io::{Error, ErrorKind::BrokenPipe, ErrorKind::InvalidData};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::mpsc::{error::TryRecvError, unbounded_channel, UnboundedReceiver, UnboundedSender}, task::JoinHandle,
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Handle {
    write_channel: UnboundedSender<String>,
    read_channel: UnboundedReceiver<String>,
    task_handles: Vec<JoinHandle<()>>
}

impl Handle {
    pub fn open(port: &str, br: u32) -> Result<Self> {
        let handle = tokio_serial::new(port, br).open_native_async()?;
        let (read_half, write_half) = tokio::io::split(handle);
        let (tx_write, rx_write) = unbounded_channel();
        let (tx_read, rx_read) = unbounded_channel();
        let h1 = tokio::spawn(async move {
            if let Err(e) = read_task(tx_read, read_half).await {
                warn!("{:?}", e)
            }
        });
        let h2 = tokio::spawn(async move {
            if let Err(e) = write_task(rx_write, write_half).await {
                warn!("{:?}", e)
            }
        });
        Ok(Self {
            write_channel: tx_write,
            read_channel: rx_read,
            task_handles: vec![h1, h2]
        })
    }

    pub fn read(&mut self) -> Result<String> {
        match self.read_channel.try_recv() {
            Ok(x) => Ok(x),
            Err(TryRecvError::Empty) => Ok(String::new()),
            Err(TryRecvError::Disconnected) => Err(Error::new(BrokenPipe, "Handle disconnected")),
        }
    }

    #[must_use]
    pub fn write(&self, content: &str) -> Result<()> {
        match self.write_channel.send(content.to_string()) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(BrokenPipe, "Handle disconnected")),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.task_handles.iter().all(|h| !h.is_finished())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.task_handles.iter().for_each(|h| h.abort());
    }
}

#[must_use]
async fn read_task(
    channel: UnboundedSender<String>,
    mut handle: ReadHalf<SerialStream>,
) -> Result<()> {
    let mut buf = [0u8; 64];
    while !channel.is_closed() {
        debug!("reading");
        let n_bytes = handle.read(&mut buf).await?;
        match std::str::from_utf8(&buf[..n_bytes]) {
            Ok(str) => {
                channel
                    .send(str.to_string())
                    .map_err(|e| Error::new(BrokenPipe, e))?;
            }
            Err(e) => {
                Err(Error::new(InvalidData, e))?;
            }
        }
    }
    info!("Read task ended");
    Ok(())
}

#[must_use]
async fn write_task(
    mut channel: UnboundedReceiver<String>,
    mut handle: WriteHalf<SerialStream>,
) -> Result<()> {
    while let Some(msg) = channel.recv().await {
        handle.write_all(msg.as_bytes()).await?;
    }
    info!("Write task ended");
    Ok(())
}
