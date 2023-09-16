use std::sync::{Arc, Mutex};

use serialport::{SerialPort, SerialPortType};

pub fn get_available_usb() -> Vec<(String, serialport::UsbPortInfo)> {
    let mut res = Vec::new();
    match serialport::available_ports() {
        Ok(ports) => {
            for p in ports {
                if let SerialPortType::UsbPort(info) = p.port_type {
                    res.push((p.port_name, info))
                }
            }
            res
        }
        Err(_) => res,
    }
}

pub fn connect(port: &str, baud_rate: u32) -> serialport::Result<Box<dyn SerialPort>> {
    serialport::new(port, baud_rate).open()
}

pub fn reconnect(port: &Box<dyn SerialPort>) -> serialport::Result<Box<dyn SerialPort>> {
    let name = port.name().unwrap_or(String::new());
    let baud_rate = port.baud_rate().unwrap_or(9600);
    serialport::new(&name, baud_rate).open()
}

struct Connection {
    port_name: String,
    baud_rate: Option<u32>,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
}
