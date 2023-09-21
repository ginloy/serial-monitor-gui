use std::{
    io::{Error, ErrorKind::NotConnected},
    path::PathBuf,
};

use dioxus::prelude::*;
use dirs::download_dir;
use log::*;
use rfd::{AsyncFileDialog, AsyncMessageDialog, MessageButtons, MessageLevel};
use tokio::time::{interval, Duration};
use tokio_serial::UsbPortInfo;

use crate::{
    handle::{self, Handle},
    ports,
};

pub const SCAN_FREQ: Duration = Duration::from_millis(500);
pub const READ_FREQ: Duration = Duration::from_millis(20);
pub const DEFAULT_BR: u32 = 9600;

pub async fn scan_ports(buffer: UseState<Vec<(String, UsbPortInfo)>>) {
    let mut interval = interval(SCAN_FREQ);
    loop {
        interval.tick().await;
        buffer.set(ports::get_available_usb())
    }
}

pub async fn read(connection: UseRef<Connection>, buffer: UseRef<String>) {
    let mut interval = interval(READ_FREQ);
    info!("Reading from {:?}", connection.read().handle);
    while connection.with(|c| c.has_handle()) {
        interval.tick().await;
        let data = connection.write().read();
        match data {
            Ok(x) => {
                if !x.is_empty() {
                    buffer.with_mut(|b| b.push_str(&x));
                }
            }
            Err(e) => {
                warn!("{:?}", e);
                // break;
            }
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
    handle: Option<Handle>,
    name: Option<String>,
    baud_rate: u32,
}

impl Connection {
    pub fn new(baud_rate: u32) -> Self {
        Self {
            handle: None,
            name: None,
            baud_rate,
        }
    }

    #[must_use]
    pub fn open(&mut self, port: &str) -> handle::Result<()> {
        self.handle = Some(Handle::open(port, self.baud_rate)?);
        self.name = Some(port.to_string());
        Ok(())
    }

    pub fn close(&mut self) {
        self.handle = None;
        self.name = None;
    }

    pub fn get_baud_rate(&self) -> u32 {
        self.baud_rate
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref().map(|s| s.as_str()).unwrap_or("none")
    }

    #[must_use]
    pub fn set_baud_rate(&mut self, rate: u32) -> handle::Result<()> {
        if self.handle.is_none() {
            self.baud_rate = rate;
            return Ok(());
        }
        self.baud_rate = rate;
        debug!("{}", self.baud_rate);
        let name = self.name.as_ref().unwrap();
        self.handle.as_mut().unwrap().reconnect(name, rate)?;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        match self.handle {
            None => false,
            Some(ref handle) => handle.is_connected(),
        }
    }

    pub fn has_handle(&self) -> bool {
        self.handle.is_some()
    }

    pub fn write(&mut self, data: &str) -> handle::Result<()> {
        match self.handle {
            None => Err(Error::new(NotConnected, "Not connected")),
            Some(ref mut handle) => handle.write(data),
        }
    }

    pub fn read(&mut self) -> handle::Result<String> {
        match self.handle {
            None => Err(Error::new(NotConnected, "Not connected")),
            Some(ref mut handle) => handle.read(),
        }
    }
}

fn process_data(titles: Vec<String>, content: Vec<String>) -> Vec<Vec<String>> {
    let mut res = Vec::new();
    res.push(titles);
    let mat: Vec<Vec<_>> = content
        .into_iter()
        .map(|s| s.lines().map(|s| s.to_string()).collect())
        .collect();
    let num_cols = mat.iter().map(|r| r.len()).max().unwrap();
    let mut row_iters: Vec<_> = mat.into_iter().map(|r| r.into_iter()).collect();
    (0..num_cols)
        .map(|_| {
            row_iters
                .iter_mut()
                .map(|it| it.next().unwrap_or("".to_string()))
                .collect()
        })
        .for_each(|r| res.push(r));
    res
}

fn download_csv(data: Vec<Vec<String>>, path: PathBuf) -> csv::Result<()> {
    let mut wtr = csv::WriterBuilder::new().from_path(&path)?;
    data.into_iter().map(|v| wtr.write_record(&v)).collect()
}

fn start_process_data(
    titles: Vec<String>,
    content: Vec<String>,
) -> std::thread::JoinHandle<Vec<Vec<String>>> {
    std::thread::spawn(move || process_data(titles, content))
}

fn start_download_csv(
    data: Vec<Vec<String>>,
    path: PathBuf,
) -> std::thread::JoinHandle<csv::Result<()>> {
    std::thread::spawn(move || download_csv(data, path))
}

pub async fn download(titles: Vec<String>, content: Vec<String>) {
    let handle = start_process_data(titles, content);
    let mut check_interval = interval(SCAN_FREQ);
    while !handle.is_finished() {
        check_interval.tick().await;
    }
    let data = handle.join().unwrap();
    match get_download_path().await {
        None => (),
        Some(path) => {
            let handle = start_download_csv(data, path.clone());
            while !handle.is_finished() {
                check_interval.tick().await;
            }
            let res = handle.join().unwrap();
            match res {
                Ok(_) => {
                    info!("Download successful");
                }
                Err(e) => {
                    show_download_error_dialog(format!("{:?}", e).as_str()).await;
                    if tokio::fs::remove_file(path).await.is_err() {
                        warn!("Failed to remove download");
                    } else {
                        warn!("Download removed due to error downloading");
                    }
                }
            }
        }
    }
}

async fn get_download_path() -> Option<PathBuf> {
    let dir = download_dir()?;
    AsyncFileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_directory(&dir)
        .set_file_name("record.csv")
        .save_file()
        .await
        .map(|p| p.path().to_owned())
}

async fn show_download_error_dialog(msg: &str) {
    AsyncMessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title("Download failed")
        .set_description(msg)
        .set_buttons(MessageButtons::Ok)
        .show()
        .await;
}
