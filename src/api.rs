use std::path::PathBuf;

use dioxus::prelude::*;
use dirs::download_dir;
use log::*;
use rfd::{AsyncFileDialog, AsyncMessageDialog, MessageButtons, MessageLevel};
use tokio::{
    io::AsyncWriteExt,
    time::{interval, Duration},
};
use tokio_serial::{SerialPort, SerialStream, UsbPortInfo};

use crate::ports;

pub static SCAN_FREQ: Duration = Duration::from_millis(20);

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
        .map(|_| row_iters.iter_mut().map(|it| it.next().unwrap_or("".to_string())).collect())
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
                },
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
        .add_filter(".csv", &["csv"])
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
