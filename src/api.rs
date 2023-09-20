use std::path::PathBuf;

use dioxus::prelude::*;
use dirs::download_dir;
use log::*;
use rfd::{FileDialog, AsyncFileDialog};
use tokio::{
    io::AsyncWriteExt,
    time::{interval, Duration},
};
use tokio_serial::{SerialPort, SerialStream, UsbPortInfo};

use crate::ports;

pub static SCAN_FREQ: Duration = Duration::from_millis(20);
pub use Download::*;

pub async fn scan_ports(buffer: UseState<Vec<(String, UsbPortInfo)>>) {
    let mut interval = interval(SCAN_FREQ);
    loop {
        interval.tick().await;
        buffer.set(ports::get_available_usb())
    }
}

pub async fn read(connection: UseRef<Connection>, buffer: UseRef<String>) {
    let mut interval = interval(SCAN_FREQ);
    info!("Reading from {:?}", connection.read().handle);
    while connection.with(|c| c.is_connected()) {
        interval.tick().await;
        let data = connection.write().read();
        if !data.is_empty() {
            buffer.with_mut(|b| b.push_str(&data));
        }
    }
    info!(
        "Reading from {:?} stopped due to loss in connection",
        connection.read()
    );
}

pub async fn connect(connection: UseRef<Connection>, port: &str) {
    let mut interval = interval(SCAN_FREQ);
    loop {
        interval.tick().await;
        match connection.write().open(port) {
            Err(e) => {
                error!("{:?}", e);
            }
            Ok(_) => break,
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    handle: Option<SerialStream>,
    baud_rate: u32,
}

impl Connection {
    pub fn new(baud_rate: u32) -> Self {
        Self {
            handle: None,
            baud_rate,
        }
    }
    pub fn open(&mut self, port: &str) -> tokio_serial::Result<()> {
        match ports::connect(port, self.baud_rate) {
            Ok(port) => {
                self.handle = Some(port);
                Ok(())
            }
            Err(e) => {
                self.handle = None;
                Err(e)
            }
        }
    }

    pub fn close(&mut self) {
        self.handle = None;
    }

    pub fn get_baud_rate(&self) -> u32 {
        self.baud_rate
    }

    pub fn set_baud_rate(&mut self, rate: u32) -> tokio_serial::Result<()> {
        if let Some(ref mut handle) = self.handle {
            handle.set_baud_rate(rate)?;
        }
        self.baud_rate = rate;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.handle.is_some()
    }

    pub async fn write(&mut self, data: &str) {
        let data = data.as_bytes();
        let res = match self.handle {
            None => {
                warn!("Attempted to write to unconnected port");
                Ok(())
            }
            Some(ref mut handle) => handle.write_all(data).await,
        };
        match res {
            Ok(()) => (),
            Err(e) => {
                warn!("{:?}", e);
                self.handle = None;
            }
        }
    }

    pub fn read(&mut self) -> String {
        let mut buf = [0u8; 64];
        match self.handle {
            None => {
                warn!("Attempted to read from unconnected port");
                String::new()
            }
            Some(ref mut handle) => match handle.try_read(&mut buf) {
                Ok(x) => std::str::from_utf8(&buf[..x]).unwrap().to_string(),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => String::new(),
                Err(e) => {
                    warn!("{:?}", e);
                    self.handle = None;
                    String::new()
                }
            },
        }
    }
}

mod Download {
    use std::{fs::File, io::Write, path::{Path, PathBuf}, thread, thread::JoinHandle};

    fn download(titles: Vec<String>, content: Vec<String>, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let mut temp = String::new();
        titles
            .into_iter()
            .map(|s| format!("\"{}\"", s))
            .enumerate()
            .map(|(i, s)| if i != 0 { format!(",{s}") } else { s })
            .for_each(|s| temp.push_str(&s));
        temp.push('\n');
        let content = content
            .into_iter()
            .map(|s| s.lines().map(|s| s.to_string()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let longest = content.iter().map(|v| v.len()).max().unwrap();
        for i in 0..longest {
            for j in 0..content.len() {
                if j != 0 {
                    temp.push(',');
                }
                temp.push_str(format!("\"{}\"", content[j].get(i).unwrap_or(&"".to_string())).as_str());
            }
            temp.push('\n');
        }
        file.write_all(temp.as_bytes())?;
        Ok(())
    }

    pub fn start_download(
        titles: Vec<String>,
        content: Vec<String>,
        path: PathBuf,
    ) -> JoinHandle<std::io::Result<()>> {
        thread::spawn(move || { download(titles, content, &path) })
    }
}

pub async fn get_download_path() -> Option<PathBuf> {
    let dir = download_dir()?;
    AsyncFileDialog::new()
        .add_filter("csv", &["csv"])
        .set_directory(&dir)
        .save_file().await.map(|f| f.inner().to_owned())
}
