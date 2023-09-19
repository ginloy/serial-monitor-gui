use tokio_serial::{self, Result, SerialPortBuilderExt, SerialPortType, SerialStream, UsbPortInfo};

pub fn get_available_usb() -> Vec<(String, UsbPortInfo)> {
    let mut res = Vec::new();
    match tokio_serial::available_ports() {
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

pub fn connect(port: &str, baud_rate: u32) -> Result<SerialStream> {
    tokio_serial::new(port, baud_rate).open_native_async()
}
