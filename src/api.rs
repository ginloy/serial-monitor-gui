use std::{
    collections::VecDeque,
    io,
    sync::{Arc, Mutex, MutexGuard}, ops::Deref,
};

use dioxus::prelude::*;
use fermi::*;
use log::{error, info, warn};
use serialport::{SerialPort, UsbPortInfo};

use crate::ports;

static SCAN_FREQ: tokio::time::Duration = tokio::time::Duration::from_millis(20);

pub async fn scan_ports(buffer: Arc<Mutex<Vec<(String, UsbPortInfo)>>>) {
    let mut interval = tokio::time::interval(SCAN_FREQ);
    loop {
        interval.tick().await;
        let mut buffer = buffer.lock().unwrap();
        ports::get_available_usb().into_iter().for_each(|elem| buffer.push(elem));
    }
}

struct Connection {
    handle: Box<dyn SerialPort>,
    is_connected: bool,
}

impl Connection {
    fn open(port: &str, baud_rate: u32) -> Option<Self> {
        match ports::connect(&port, baud_rate) {
            Ok(port) => Some(Self {
                handle: port,
                is_connected: true,
            }),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    fn close(&mut self) {
        self.is_connected = false;
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }

    fn write(&mut self, data: &str) {
        let data: Vec<u8> = data.chars().map(|c| c as u8).collect();
        match self.handle.write_all(&data) {
            Ok(_) => (),
            Err(e) => {
                warn!("{:?}", e);
                self.is_connected = false;
            }
        }
    }

    fn read(&mut self) -> String {
        let mut buf = String::new();
        match self.handle.read_to_string(&mut buf) {
            Ok(_) => buf,
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => buf,
            Err(e) => {
                warn!("{:?}", e);
                self.is_connected = false;
                buf
            }
        }
    }

    async fn scan(&mut self, buffer: Arc<Mutex<String>>) {
        let mut interval = tokio::time::interval(SCAN_FREQ);
        while self.is_connected(){
            interval.tick().await;
            buffer.lock().unwrap().push_str(&self.read());
        }
    }
}

pub struct AppState {
    available_ports: Arc<Mutex<Vec<(String, UsbPortInfo)>>>,
    handle: Option<Arc<Mutex<Connection>>>,
    input_text: Arc<Mutex<String>>,
    output_text: Arc<Mutex<String>>
}

impl AppState {
    pub fn new() -> Self {
        Self {
            available_ports: Arc::new(Mutex::new(Vec::new())),
            handle: None,
            input_text: Arc::new(Mutex::new(String::new())),
            output_text: Arc::new(Mutex::new(String::new()))
        }
    }

    async fn connect(&mut self, port: &str, baud_rate: u32) {
        if let Some(ref handle) = self.handle {
            handle.lock().unwrap().close();
        }
        self.handle = loop {
            match Connection::open(port, baud_rate) {
                Some(port) => break Some(Arc::new(Mutex::new(port))),
                None => ()
            }
        };
        let handle = self.handle.clone();
        // tokio::spawn(async move {
        //     let mut interval = tokio::time::interval(SCAN_FREQ);
        //     loop {
        //         interval.tick().await;
        //     }
        // })
    }

    fn disconnect(&mut self) {
        if let Some(ref handle) = self.handle {
            handle.lock().unwrap().close();
        }
    }

    pub fn get_input_text(&self) -> Inner<String> {
        Inner(self.input_text.lock().unwrap())
    }

    pub fn get_output_text(&self) -> Inner<String> {
        Inner(self.output_text.lock().unwrap())
    }

    pub fn append_input(&mut self, string: &str) {
        self.input_text.lock().unwrap().push_str(string);
    }

    pub fn clear_output(&mut self) {
        self.output_text.lock().unwrap().clear();
    }

    pub fn clear_input(&mut self) {
        self.input_text.lock().unwrap().clear();
    }
}

pub enum Action {

}


pub async fn start_service(mut rx: UnboundedReceiver<Action>, atoms: AtomRoot) {
    
}

pub struct Inner<'a, T>(MutexGuard<'a, T>);

impl<'a, T> Deref for Inner<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
} 
