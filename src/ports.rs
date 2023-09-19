use std::time::Duration;

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
    serialport::new(port, baud_rate).timeout(Duration::from_millis(0)).open()
}
