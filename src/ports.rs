use log::*;
use tokio_serial::{self, SerialPortType};

pub fn get_available_usb() -> Vec<PortInfo> {
    let mut res = Vec::new();
    match tokio_serial::available_ports() {
        Ok(ports) => {
            for p in ports {
                debug!("{:?}", p);
                // if let SerialPortType::UsbPort(info) = p.port_type {
                //     res.push((p.port_name, info))
                // }
                let name = p.port_name;
                let (manufacturer, product) = {
                    if let SerialPortType::UsbPort(info) = p.port_type {
                        (info.manufacturer, info.product)
                    } else {
                        (None, None)
                    }
                };
                res.push(PortInfo::new(name, manufacturer, product))
            }
            res
        }
        Err(_) => res,
    }
}

pub struct PortInfo {
    name: String,
    manufacturer: Option<String>,
    product: Option<String>,
}

impl PortInfo {
    fn new(name: String, manufacturer: Option<String>, product: Option<String>) -> Self {
        Self {
            name,
            manufacturer,
            product,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn manufacturer(&self) -> &str {
        self.manufacturer
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Unknown")
    }

    pub fn product(&self) -> &str {
        self.product
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Unknown")
    }
}
